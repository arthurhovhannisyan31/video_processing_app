# Backend suggestions (not yet applied)

Review of `modules/backend` changes pushed since the W5 review (`8a61741..HEAD` on `main`, ~21 backend files). Several W5 items were applied in the meantime (path traversal strip, login timing hardening, ffmpeg/ffprobe timeouts, redundant `get_by_email`, dead `#[from]`, redundant JWT `exp` check). The notes below are new issues, most of them leftovers from those partly-finished refactors.

## Bugs

### 1. `ffprobe` timeout returns the wrong tool name

`features/video/inspect/ffprobe_runner.rs`:

```rust
  let output = match timeout(
    VIDEO_API_INSPECT_TIMEOUT,
    ffprobe_process.wait_with_output(),
  )
  .await
  {
    Ok(output) => output?,
    Err(_) => return Err(ServerError::Processing("ffmpeg timed out".to_string())),
  };
```

Copy-paste from `ffmpeg_runner.rs`. An `ffprobe` timeout on `/api/video/inspect` reports `ffmpeg timed out`, which points debugging at the wrong subsystem.

**Fix:**

```rust
    Err(_) => return Err(ServerError::Processing("ffprobe timed out".to_string())),
```

---

### 2. Dead `file_path.is_empty()` check left behind in `read_video_to_file`

`features/video/helpers.rs::read_video_to_file`:

```rust
  if written_bytes == 0 {
    return Err(ServerError::DataError(
      "Uploaded video file is empty".to_string(),
    ));
  }

  if file_path.is_empty() {
    return Err(ServerError::DataError("Missing 'video' field".to_string()));
  }

  Ok(file_path)
```

The `written_bytes == 0` guard is the W5 #4 fix and it is correct. The `file_path.is_empty()` line right below it is the old broken check that #4 asked to remove — `file_path` is `temp_dir.join(...).to_string_lossy()`, never empty. It is pure dead code now, and the `"Missing 'video' field"` message is also wrong for this function (the "no video field at all" case is handled one level up in `form_data_reader::read`, where the same check *is* meaningful).

**Fix:** delete the `file_path.is_empty()` block from `read_video_to_file`. Keep the identical checks in `inspect/form_data_reader.rs` and `process/form_data_reader.rs` — there `file_path` starts as `String::new()` and stays empty when no `video` field is sent, so they do their job.

---

### 3. `login` refactor left an unreachable branch and a wasted hash on DB errors

`features/auth/service.rs::login`:

```rust
    let password_valid = match verify_password(password, password_hash) {
      Ok(true) => true,
      Ok(false) => return Err(ApplicationError::Unauthorized),
      Err(err) => return Err(ApplicationError::Internal(err.to_string())),
    };

    if !password_valid {
      return Err(ApplicationError::Unauthorized);
    }

    let user = match user_res {
      Ok(user) => user,
      Err(ApplicationError::NotFound(_)) => {
        return Err(ApplicationError::Unauthorized);
      }
      Err(err) => return Err(err),
    };
```

Two leftovers from the timing-hardening rewrite:

- `password_valid` can only ever be `true` at the point it is read (`Ok(false)` already returned). The `if !password_valid` block is dead.
- When `get_by_email` fails with a real error (DB down, pool exhausted), the code still runs a full argon2 verification against `DUMMY_PASSWORD_HASH` first, then falls through to `Err(err) => return Err(err)`. Every login during a DB outage pays the hash cost for nothing.

**Fix:** model "not found" as `Option`, not `Err(NotFound)` — a dedicated `get_by_email_opt` (or `fetch_optional` at the repo) keeps real DB errors on the `?` path and collapses the rest into one tuple match, no dead branch:

```rust
    let user: Option<User> = self.repo.find_by_email(&email.to_lowercase()).await?;

    let password_hash = user
      .as_ref()
      .map(|u| u.password_hash.as_str())
      .unwrap_or(DUMMY_PASSWORD_HASH);

    // constant-time regardless of whether the email exists
    let password_ok = match verify_password(password, password_hash) {
      Ok(ok) => ok,
      Err(err) => return Err(ApplicationError::Internal(err.to_string())),
    };

    let user = match (user, password_ok) {
      (Some(user), true) => user,
      _ => return Err(ApplicationError::Unauthorized),
    };

    let token = self
      .jwt_service
      .generate_token(user.id.clone(), user.username.clone())
      .map_err(|err| ApplicationError::Internal(err.to_string()))?;

    Ok((user, token))
```

---

### 4. ffmpeg progress logging routed through the `log` shim, not `tracing` — W5 #5 half-reverted

`features/video/process/ffmpeg_runner.rs`:

```rust
use tracing::log::info;
use tracing::warn;
// ...
          "progress" if value == "continue" || value == "end" => {
            info!("Progress Update -> Frame: {current_frame}, FPS: {current_fps}");
          }
```

`tracing::log::info` is the `log`-crate compatibility macro re-exported by `tracing`, not `tracing::info`. It emits a `log` record, which only reaches the tracing subscriber if a `tracing-log` bridge is installed — otherwise it is dropped or goes to a separate `log` backend. This is the same class of problem W5 #5 flagged with `println!`. Also `warn` is now imported but unused (the `else if !line.is_empty()` branch that used it was removed), so `cargo clippy` warns on every build.

**Fix:**

```rust
use tracing::info;
// ...
            info!(frame = %current_frame, fps = %current_fps, "ffmpeg progress");
```

and drop the `use tracing::warn;` line (or reinstate a `warn!(%line, "ffmpeg log")` branch for non-progress stderr lines — losing all ffmpeg stderr makes failed conversions hard to diagnose).

---

### 5. `log_task` result is discarded, and on timeout the task is never awaited

`features/video/process/ffmpeg_runner.rs`:

```rust
  let status = match timeout(VIDEO_API_PROCESS_TIMEOUT, ffmpeg_process.wait()).await {
    Ok(res) => res?,
    Err(_) => return Err(ServerError::Processing("ffmpeg timed out".to_string())),
  };

  let _: Result<(), ServerError> = log_task.await?;
```

- On the timeout branch the function returns immediately; `log_task` is never awaited or aborted. `kill_on_drop` closes the child's stderr so the task will end on its own, but until then it is a detached task holding a `Lines` reader.
- On the success branch `log_task.await?` propagates a `JoinError` but then binds the inner `Result` to `_`, so any `io::Error` from reading stderr is silently swallowed.

**Fix:** keep a handle and abort on timeout, and surface the task result:

```rust
  let status = match timeout(VIDEO_API_PROCESS_TIMEOUT, ffmpeg_process.wait()).await {
    Ok(res) => res?,
    Err(_) => {
      log_task.abort();
      return Err(ServerError::Processing("ffmpeg timed out".to_string()));
    }
  };

  if let Err(err) = log_task.await? {
    warn!(%err, "ffmpeg stderr reader failed");
  }
```

---

### 6. `ApplicationError::Internal` returns the raw internal error string to the client

`core/error.rs`:

```rust
      ApplicationError::Internal(msg) => (
        StatusCode::INTERNAL_SERVER_ERROR,
        json!({"message": msg}).to_string(),
      )
        .into_response(),
```

Every `ApplicationError::Internal(err.to_string())` site — `verify_password` failure, `generate_token` failure, `TempDir::new` failure, `ffprobe_mapper` errors — puts the underlying error text straight into the HTTP response body. That leaks internal detail (crate names, file paths, DB driver messages) to any caller.

**Fix:** log the detail server-side, return a fixed message to the client:

```rust
      ApplicationError::Internal(msg) => {
        tracing::error!(error = %msg, "internal error");
        (
          StatusCode::INTERNAL_SERVER_ERROR,
          json!({"message": "Internal server error"}).to_string(),
        )
          .into_response()
      }
```

Same treatment for any `ServerError` variant that reaches the client with a `Display` that isn't already generic.

---

## General improvements

### 7. Rate-limiter setup copy-pasted between `auth/routes.rs` and `video/routes.rs`

Same block in both files:

```rust
  if app_state.app_config.is_production {
    let rate_limiter = GovernorConfigBuilder::default()
      // ...
      .key_extractor(SmartIpKeyExtractor)
      .finish()
      .ok_or(ServerError::OtherError(anyhow!(
        "Wrong tower_governor configuration"
      )))?;
    router = router.layer(GovernorLayer::new(rate_limiter));
  }
```

**Fix:** one helper, e.g. `core::rate_limit::governor_layer(period_secs: u64, burst: u32) -> Result<GovernorLayer<...>, ServerError>`, called from both routers. Keeps the `is_production` gate and the error message in one place.

---

### 8. `GovernorConfigBuilder::period` is a per-cell refill interval, not a window

`video/routes.rs`:

```rust
    .period(Duration::from_secs(app_state.app_config.video_rate_limit_period)) // 3600
    .burst_size(app_state.app_config.video_rate_limit_size)                    // 100
```

`tower_governor` is a GCRA limiter: `burst_size` is the bucket capacity and `period` is the time to replenish **one** cell. With `period = 3600s` the steady-state rate after the initial burst is 1 request per hour, not "100 requests per hour". If the intent is ~100/hour sustained, `period` should be `3600 / 100 = 36s`.

**Fix:** derive `period` from the rate — store `video_rate_limit_size` (max burst) and a separate `video_rate_limit_per_sec` or compute `period = window / size`. Add a comment stating the resulting sustained rate so the next reader doesn't have to reverse-engineer GCRA.

---

### 9. Config surface is half-env, half-constant

`features/video/constants.rs`:

```rust
pub const VIDEO_MAX_BODY_SIZE: usize = 100 * 1024 * 1024;      // env-overridable
pub const VIDEO_API_INSPECT_TIMEOUT: Duration = ...secs(20);   // hard-coded
pub const VIDEO_API_PROCESS_TIMEOUT: Duration = ...secs(300);  // hard-coded
pub const VIDEO_RATE_LIMIT_PERIOD: u16 = 3600;                 // env-overridable
pub const VIDEO_RATE_LIMIT_SIZE: u16 = 100;                    // env-overridable
```

Body size and rate limits read from `BACKEND_VIDEO_*` env vars; the two ffmpeg/ffprobe timeouts don't, even though they are the values most likely to need tuning per deployment (slow disks, large files). Also the rate-limit constants are typed `u16` while the matching `AppConfig` fields are `u64`/`u32`, bridged only by `.to_string()` in `from_env` — a real type mismatch waiting to bite a future refactor.

**Fix:** move both timeouts into `AppConfig` alongside the others (`BACKEND_VIDEO_INSPECT_TIMEOUT_SEC`, `BACKEND_VIDEO_PROCESS_TIMEOUT_SEC`), and make the constant types match the config field types (`u64` for period-seconds, `u32` for burst).

---

### 10. `DUMMY_PASSWORD_HASH` is a hard-coded literal silently coupled to `Argon2::default()`

`features/auth/constants.rs`:

```rust
pub const DUMMY_PASSWORD_HASH: &str = "$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHQ...";
```

The params (`m=19456, t=2, p=1`) currently match `argon2::Argon2::default()`, so the timing defence in `login` works. The day someone tunes `hash_password` to `Argon2::new(Algorithm::Argon2id, Version::V0x13, Params::new(...))`, this literal stops matching and the "unknown email" path becomes measurably faster again — with no test failing.

**Fix:** compute the dummy hash once at startup with the *same* hasher used for real passwords, so it can never drift:

```rust
use std::sync::OnceLock;

/// Valid Argon2 hash no real password matches. Burns the same CPU as a real
/// verify when the email is unknown, so login latency can't enumerate accounts.
pub fn dummy_password_hash() -> &'static str {
  static DUMMY: OnceLock<String> = OnceLock::new();
  DUMMY.get_or_init(|| hash_password("dummy-password-for-timing-safety").expect("dummy hash"))
}
```

and call `dummy_password_hash()` in `login` instead of the const.

---

### 11. Inconsistent `ServerError` variants for the same condition

`inspect/form_data_reader.rs` vs `process/form_data_reader.rs`, identical "missing field name" case:

```rust
// inspect
.ok_or(ServerError::OtherError(anyhow!("Missing field name".to_string())))?
// process
.ok_or(ServerError::DataError("Missing field name".to_string()))?
```

Same for the `TempDir::new()` failure in `video/routes.rs`: `inspect_video` maps it to `ApplicationError::Internal(format!(...))`, `process_video` maps it to `ServerError::IO`. Same fault, two error types → two different HTTP responses / log shapes.

**Fix:** pick one variant per condition (`DataError` for bad client input, `IO` for `TempDir`) and use it in both handlers. `anyhow!("literal".to_string())` is also redundant — `anyhow!("literal")` is enough.

---

### 12. Only `/health` — no readiness probe

`features/system/routes.rs` exposes `GET /health`, which returns static JSON regardless of whether the process can actually serve traffic (DB pool up, migrations applied). With the container + nginx deploy now in place, there is no endpoint for the reverse proxy / compose healthcheck to gate on.

**Fix:** add `GET /ready` that does a cheap liveness check on the pool (`SELECT 1` / `pool.acquire()` with a short timeout) and returns 503 when it fails. Point the `docker-compose.prod.yml` healthcheck and the nginx upstream check at `/ready`; keep `/health` as the bare liveness probe.
