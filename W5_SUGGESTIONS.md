# Backend suggestions (not yet applied)

## General improvements

### 1. `#[allow(dead_code)]` in `auth/middleware.rs` hides unused code

`features/auth/middleware.rs:63` has:

```rust
#[allow(dead_code)]
pub fn get_auth_cookie(token: &str, is_secure: bool) -> Cookie<'static> {
```

Silencing the warning hides that `get_auth_cookie` builds a `Set-Cookie`-style `Authorization` cookie but nothing in the codebase calls it — auth currently only reads the `Authorization` header, never sets a cookie.

**Fix:** either wire this function into the login/register response path (if cookie-based auth is planned), or delete it instead of suppressing the warning.

---

### 2. `DomainError::PostNotFound` looks like leftover template code

`core/error.rs` defines:

```rust
#[error("Post not found: {0}")]
PostNotFound(u64),
```

There is no "post" concept anywhere in this video-processing app (no `Post` model, route, or repository) — `PostNotFound` is dead code, likely copied from a different (blog/CMS-style) template project, and it's matched in the `From<DomainError> for ApplicationError` impl even though nothing ever constructs it.

**Fix:** remove the variant and its match arm:

```rust
#[error("Post not found: {0}")]
PostNotFound(u64),
```

---

### 3. `features/video/model.rs` is an empty file

`features/video/model.rs` contains nothing (just a blank line), unlike `features/auth/model.rs` which holds the actual `User`/`UserId` types. Every other feature module that declares a `model` submodule uses it; this one is dead weight.

---

### 4. `PostgresUserRepository` error handling has avoidable boilerplate

`features/auth/repository.rs::find_by_email` and `::find_by_id` both do:

```rust
let row = sqlx::query_as!(...)
  .fetch_optional(&self.pool)
  .await
  .map_err(|e| {
    error!("Failed to find user by id {}: {}", id, e);
    DomainError::Internal(format!("database error: {}", e))
  })?;

Ok(row)
```

Also, `find_by_id` passes `id.0` to bind the query param, unwrapping the `UserId` newtype manually even though `UserId` derives `#[sqlx(transparent)]`, which already implements `Encode`/`Decode` for the wrapper — `id` binds directly, no `.0` needed.

**Fix:** add a `#[from] sqlx::Error` variant to `DomainError`, and use `.inspect_err()` to keep the log without a full `map_err` closure:

```rust
#[error("database error: {0}")]
Database(#[from] sqlx::Error),
```

```rust
async fn find_by_id(&self, id: UserId) -> Result<Option<User>, DomainError> {
  Ok(
    sqlx::query_as!(
      User,
      r#"SELECT users.id as "id: UserId", users.username, users.email, users.password_hash, users.created_at FROM users WHERE users.id = $1"#,
      id
    )
    .fetch_optional(&self.pool)
    .await
    .inspect_err(|err| error!("Failed to find user by id {id}: {err}"))?
  )
}
```

`create()` keeps its full `map_err` since it branches on the constraint violation to return `UserAlreadyExists` — that logic can't be expressed through a plain `#[from]` conversion.

---

### 5. Same `#[from]` opportunity for JWT / password-hash errors on `ApplicationError`

`core/jwt.rs::generate_token` returns `Result<String, jsonwebtoken::errors::Error>` and `hash_password`/`verify_password` return `Result<_, argon2::password_hash::Error>`. All three call sites convert them the same way:

- `features/auth/service.rs:46` — `hash_password(...).map_err(|err| ApplicationError::Internal(err.to_string()))?`
- `features/auth/service.rs:78` — `.generate_token(...).map_err(|err| ApplicationError::Internal(err.to_string()))`
- `features/auth/routes.rs:83` — `.generate_token(...).map_err(|err| ApplicationError::Internal(err.to_string()))?`

Same fix as #4: add dedicated `#[from]` variants to `ApplicationError`:

```rust
#[error("JWT error: {0}")]
Jwt(#[from] jsonwebtoken::errors::Error),
#[error("password hash error: {0}")]
PasswordHash(#[from] argon2::password_hash::Error),
```

Then all three call sites collapse to plain `?`, no closure needed.

`core/database.rs::create_pool` also re-implements a conversion `ServerError` already has: `.connect(database_url).await.map_err(|e| ServerError::SqlxError(format!("Failed connecting to Postgres: {e}")))?` duplicates the existing `impl From<sqlx::Error> for ServerError` (`error.rs:68-72`) — `.connect(database_url).await?` would already work via that impl (only losing the `"Failed connecting to Postgres: "` prefix, recoverable with `.inspect_err()` if wanted). `run_migrations`'s `sqlx::migrate!().run(pool).await` is a different error type (`sqlx::migrate::MigrateError`, not `sqlx::Error`), so it still needs its own `#[from]` variant added to benefit from the same pattern.

---

### 6. `form_data_reader.rs` is duplicated between `inspect/` and `process/`

`inspect/form_data_reader.rs`:

```rust
pub async fn read(
  mut media_data: Multipart,
  temp_dir: &Path,
) -> Result<String, ApplicationError> {
  let mut file_path = String::new();

  while let Some(mut field) = media_data.next_field().await? {
    let field_name = field
      .name()
      .ok_or(ApplicationError::BadRequest(
        "Missing field name".to_string(),
      ))?
      .to_string();

    if field_name == "video" {
      file_path = read_video_to_file(&mut field, temp_dir).await?;
      break;
    }
  }

  Ok(file_path)
}
```

`process/form_data_reader.rs`:

```rust
pub async fn read(
  mut media_data: Multipart,
  temp_dir: &Path,
) -> Result<ProcessVideoMeta, ApplicationError> {
  let mut meta = ProcessVideoMeta::default();

  while let Some(mut field) = media_data.next_field().await? {
    let field_name = field
      .name()
      .ok_or(ApplicationError::BadRequest(
        "Missing field name".to_string(),
      ))?
      .to_string();

    match field_name.as_str() {
      "video" => {
        meta.file_path = read_video_to_file(&mut field, temp_dir).await?;
      }
      "operation" => {
        meta.command = field.text().await?;
      }
      _ => {}
    }
  }

  Ok(meta)
}
```

Both implement the same multipart-walking loop (`next_field`, match on field name, error on missing name). The `process` version just adds handling for an extra `"operation"` field.

**Fix:** factor the shared multipart-walk loop into `features/video/helpers.rs` (which already hosts `read_video_to_file`), and have both `read()` functions call into it, passing a per-field-name callback or match arm for the extra fields each one needs.

---

### 7. No `clippy.toml` in `modules/backend`

A `clippy.toml` can ban untyped `sqlx::query`/`sqlx::query_as` in favor of `query_as!`/`query_scalar!` — enforcing type safety at compile time instead of relying on convention alone. `video_processing_app` has no `clippy.toml` at all.

**Fix:** add `modules/backend/clippy.toml`:

```toml
allow-unwrap-in-tests = true
allow-expect-in-tests = true
allow-panic-in-tests = true

disallowed-macros = [
    { path = "sqlx::query", reason = "Use query_as! or query_scalar! for compile-time type safety" },
]
disallowed-methods = [
    { path = "sqlx::query_as", reason = "Use query_as! macro for compile-time type safety" },
    { path = "sqlx::query",    reason = "Use query_as! or query_scalar! macro for compile-time type safety" },
]
```

---

### 8. Improve `.rustfmt.toml`

`modules/backend/.rustfmt.toml` currently has:

```toml
tab_spaces = 2
max_width = 80
```

A more standard pattern would instead use:

```toml
edition = "2024"
max_width = 100
imports_granularity = "Module"
group_imports = "StdExternalCrate"
```

`tab_spaces = 2` is non-standard for Rust (rustfmt default and community convention is 4), which is why files like `features/auth/constants.rs` and the `ffprobe_runner`/`ffmpeg_runner` modules read awkwardly compared to typical Rust code. There's also no `imports_granularity` / `group_imports` setting, so imports aren't sorted into std / external / crate groups (visible in `ffprobe_runner.rs`: `core::error`, then `serde_json`, then `tokio`, in file order rather than grouped).

**Fix:** change `tab_spaces` to `4`, and add `imports_granularity = "Module"` + `group_imports = "StdExternalCrate"`. Then run `cargo fmt` once across the backend to apply consistently (large mechanical diff, no logic change).

---

### 9. `features/auth/constants.rs` wraps a single constant group in an unnecessary `pub mod`

```rust
pub mod db_constraints {
  pub const USERS_EMAIL: &str = "users_email_key";
  pub const USERS_USERNAME: &str = "users_username_key";
}
```

Only one category of constants exists in the file, so the `db_constraints` submodule adds a layer of indirection without benefit — callers write `constants::db_constraints::USERS_EMAIL` instead of `constants::USERS_EMAIL`.

**Fix:** flatten to top-level consts unless/until a second, distinct category is added:

```rust
pub const USERS_EMAIL: &str = "users_email_key";
pub const USERS_USERNAME: &str = "users_username_key";
```

---

### 10. `users.id` column name doesn't signal it's a UUID

`migrations/20260714000001_create_schema_users.sql`:

```sql
id            UUID PRIMARY KEY     DEFAULT gen_random_uuid(),
```

Naming it `id` reads like a plain auto-increment integer PK, which this isn't — it's a random UUID. That mismatch can mislead someone skimming the schema or writing a join/DTO without checking the type.

**Fix:**

```sql
uuid          UUID PRIMARY KEY     DEFAULT gen_random_uuid(),
```

---

### 11. Video routes are missing from the OpenAPI spec

`core/openapi.rs`:

```rust
use crate::features::auth::routes::{__path_login, __path_register};
use crate::features::system::routes::{__path_health, __path_openapi};

use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
  info(title = "Video processing API specification", version = "1.0.0"),
  paths(login, register, health, openapi)
)]
pub struct OpenApiSpec;
```

`login`/`register`/`health`/`openapi` are documented, but `inspect_video` and `process_video` (`features/video/routes.rs`) have no `#[utoipa::path(...)]` attribute at all, unlike every handler in `auth/routes.rs`. The two main video-processing endpoints — the actual point of this API — are invisible in the generated spec.

**Fix:** add `#[utoipa::path(...)]` to `inspect_video` and `process_video`, then register them:

```rust
use crate::features::video::routes::{__path_inspect_video, __path_process_video};

#[openapi(
  info(title = "Video processing API specification", version = "1.0.0"),
  paths(login, register, health, openapi, inspect_video, process_video)
)]
pub struct OpenApiSpec;
```

---

### 12. CORS origin parsing silently drops invalid entries

`core/cors.rs`:

```rust
let origin_values: Vec<HeaderValue> = app_config
  .cors_origins
  .iter()
  .filter_map(|el| el.parse().ok())
  .collect();
```

If a `cors_origins` entry can't parse as a `HeaderValue` (typo, stray whitespace, malformed URL), `filter_map` drops it silently — no log, no error. Combined with `.allow_credentials(true)`, a misconfigured origin just quietly stops working with no signal of why.

**Fix:** log parse failures instead of swallowing them:

```rust
let origin_values: Vec<HeaderValue> = app_config
  .cors_origins
  .iter()
  .filter_map(|el| {
    el.parse()
      .inspect_err(|err| tracing::warn!("Invalid CORS origin {el}: {err}"))
      .ok()
  })
  .collect();
```

---

### 13. `login` has no brute-force protection

`features/auth/routes.rs::login`:

```rust
pub async fn login(
  State(auth_state): State<Arc<AuthState>>,
  Json(payload): Json<AuthRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
  payload.validate()?;

  let token = auth_state
    .auth_service
    .login(&payload.email, &payload.password)
    .await?;
  let user = auth_state.auth_service.get_by_email(&payload.email).await?;
  build_auth_response(StatusCode::OK, token.clone(), user)
}
```

There's no rate limiting, no lockout, and no per-IP/per-account failure tracking anywhere in the auth flow — an attacker can hit `/auth/login` as fast as argon2 verification allows, with no backoff.

**Fix:** track failed attempts per IP (or per email) in-memory and reject once a threshold is hit within a time window:

```rust
pub struct LoginLimiter {
  ip_failures: Mutex<HashMap<String, (u32, Instant)>>,
}

impl LoginLimiter {
  pub fn is_blocked(&self, ip: &str) -> bool { /* ... */ }
  pub fn record_failure(&self, ip: &str) -> bool { /* ... */ }
  pub fn record_success(&self, ip: &str) { /* ... */ }
}
```

Wire it into `login` via `ConnectInfo<SocketAddr>` (or an `X-Forwarded-For` extractor behind a proxy), checking `is_blocked` before calling `auth_service.login`, and calling `record_failure`/`record_success` based on the result.

---

### 14. No test coverage on service/repository/route logic

Only `features/auth/middleware.rs` and `features/auth/dto.rs` have any `#[test]`/`#[cfg(test)]` code. `AuthService::register`/`login` (`features/auth/service.rs`), `PostgresUserRepository` (`features/auth/repository.rs`), and every route handler (`login`, `register`, `inspect_video`, `process_video`) have zero tests — including the handlers carrying the bugs described below (#15, #16).

**Fix:** add unit tests for `AuthService` (mock the repository trait), and integration tests per route handler that exercise the happy path plus the validation/error branches (missing multipart fields, wrong credentials, duplicate email).

---

## Bugs

### 15. `process_video` deletes its own output before returning it

`features/video/routes.rs::process_video`:

```rust
pub async fn process_video(
  media_data: Multipart,
) -> Result<impl IntoResponse, ApplicationError> {
  let temp_dir = TempDir::new().map_err(|err| {
    ApplicationError::Internal(format!(
      "Failed to create temp directory: {err}"
    ))
  })?;
  let ProcessVideoMeta { command, file_path } =
    process::form_data_reader::read(media_data, temp_dir.path()).await?;
  let output_path = append_path_suffix(&file_path, OUTPUT_PATH_SUFFIX)?;
  let preset = get_preset_by_name(&command)?;

  ffmpeg_runner(&file_path, &output_path, preset).await?;

  Ok("Success")
}
```

`temp_dir` writes the uploaded video into itself, `ffmpeg_runner` produces `output_path` next to it (via `append_path_suffix`, inside the same temp dir), then the function returns the plain string `"Success"` — `output_path` is never read or returned.

`TempDir` deletes its directory (and everything in it) on `Drop`. Since `temp_dir` is a local variable in `process_video`, it drops at the end of the function — after `ffmpeg_runner` has finished but before/as the response is sent. The processed file is deleted immediately and is never returned to the caller. As it stands, the endpoint does real work (transcodes the video) and then discards the only copy of the result.

**Fix:** either stream `output_path`'s bytes back in the response body (e.g. `axum::body::Body::from_stream` reading the file, or read it fully into a `Vec<u8>` for smaller outputs) before `temp_dir` is dropped, or move the output file to persistent/object storage and return a reference to it instead of `"Success"`.

---

### 16. `ffmpeg_runner` never checks the process exit status

`modules/backend/src/features/video/process/ffmpeg_runner.rs`:

```rust
pub async fn ffmpeg_runner(
  input: &str,
  output: &str,
  preset: Vec<&str>,
) -> Result<(), ApplicationError> {
  let mut args: Vec<&str> = vec!["-i", input];
  args.extend(preset);
  args.extend([output]);

  let output =
    Command::new("ffmpeg")
      .args(args)
      .output()
      .await
      .map_err(|err| {
        ApplicationError::Internal(format!(
          "Failed executing 'ffmpeg' binary: {err}"
        ))
      })?;

  Ok(())
}
```

`output.status.success()` and `output.stderr` are never checked. If ffmpeg fails (bad codec, corrupt input, missing preset arg), the function still returns `Ok(())` as if the conversion succeeded.

**Fix:** mirror the pattern already used in `ffprobe_runner.rs` (`inspect/ffprobe_runner.rs`), which does check `probe_output.status.success()` and returns the stderr in the error.

---

### 17. `inspect/form_data_reader.rs` silently succeeds with an empty file path

```rust
pub async fn read(
  mut media_data: Multipart,
  temp_dir: &Path,
) -> Result<String, ApplicationError> {
  let mut file_path = String::new();

  while let Some(mut field) = media_data.next_field().await? {
    let field_name = field
      .name()
      .ok_or(ApplicationError::BadRequest(
        "Missing field name".to_string(),
      ))?
      .to_string();

    if field_name == "video" {
      file_path = read_video_to_file(&mut field, temp_dir).await?;
      break;
    }
  }

  Ok(file_path)
}
```

If the multipart payload has no `"video"` field, the `while let` loop finishes without ever assigning `file_path`, and the function returns `Ok(String::new())` instead of an error. The caller then proceeds with an empty path.

**Fix:** compare with `process/form_data_reader.rs` — same missing-field problem there. Return an explicit `ApplicationError::BadRequest("Missing 'video' field")` when the field was never found, instead of relying on the caller to notice an empty string.
