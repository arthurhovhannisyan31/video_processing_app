# Backend suggestions (not yet applied)

Review of `modules/backend` changes pushed over the last 2 weeks (`3fe9f8e^..HEAD`, 54 files).

## Bugs

### 1. Path traversal / arbitrary file write in video upload

`features/video/helpers.rs::read_video_to_file`:

```rust
pub async fn read_video_to_file(
  field: &mut Field<'_>,
  temp_dir: &Path,
) -> Result<String, ServerError> {
  let file_name = field
    .file_name()
    .ok_or(ServerError::DataError("Missing file_name".to_string()))?;
  let path = temp_dir.join(file_name);
```

The uploaded video's client-supplied multipart filename is joined to `temp_dir` via `Path::join` with no sanitization. An absolute path (e.g. `/root/.ssh/authorized_keys`) makes `Path::join` discard the base entirely; a relative path with `../../..` escapes `temp_dir` the same way. Affects both `/api/video/inspect` and `/api/video/jobs` — any authenticated user (just needs a valid JWT) can write a file anywhere the process has write permission.

**Fix:** strip to the last path segment before joining — `Path::file_name()` discards any directory components the client sent:

```rust
pub async fn read_video_to_file(
  field: &mut Field<'_>,
  temp_dir: &Path,
) -> Result<String, ServerError> {
  let file_name = field
    .file_name()
    .ok_or(ServerError::DataError("Missing file_name".to_string()))?;
  let safe_file_name = Path::new(file_name)
    .file_name()
    .ok_or(ServerError::DataError("Invalid file_name".to_string()))?;
  let path = temp_dir.join(safe_file_name);
```

---

### 2. Login timing side-channel leaks which emails are registered

`features/auth/service.rs::login`:

```rust
pub async fn login(&self, email: &str, password: &str) -> Result<String, ApplicationError> {
    let user = match self.get_by_email(email).await {
      Ok(user) => user,
      Err(ApplicationError::NotFound(_)) => {
        return Err(ApplicationError::Unauthorized);
      }
      Err(err) => return Err(err),
    };

    let password_valid = match verify_password(password, &user.password_hash) {
```

`login()` returns `Unauthorized` before calling `verify_password` when the email doesn't exist. Unknown email responds fast (no argon2 hash); known email + wrong password costs the full hash (tens of ms) — lets an attacker enumerate registered emails by measuring response latency.

**Fix:** always run the hash comparison, against a fixed dummy hash when no user was found, so both paths cost the same:

```rust
// A hash with no matching plaintext password — computed once, spends the same
// argon2 time as a real user so "email not found" and "wrong password" are
// indistinguishable by response latency.
const DUMMY_PASSWORD_HASH: &str =
  "$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHQAAAAAAAAAAA$XLZuXVUXAvNRolPtu/DDS1QRr50p9j37F9/AYT5Zk7c";

pub async fn login(&self, email: &str, password: &str) -> Result<String, ApplicationError> {
    let user = self.get_by_email(email).await.ok();

    let password_hash = user
      .as_ref()
      .map(|u| u.password_hash.as_str())
      .unwrap_or(DUMMY_PASSWORD_HASH);

    let password_valid = match verify_password(password, password_hash) {
      Ok(valid) => valid,
      Err(err) => return Err(ApplicationError::Internal(err.to_string())),
    };

    if user.is_none() || !password_valid {
      return Err(ApplicationError::Unauthorized);
    }
    let user = user.unwrap();
```

---

### 3. `ffmpeg`/`ffprobe` child process has no timeout

`features/video/process/ffmpeg_runner.rs`:

```rust
  let status = child_process.wait().await?;
  let _: Result<(), ServerError> = log_task.await?;
```

A crafted/corrupted video can make the ffmpeg child process stall forever. `child_process.wait().await` never resolves, the handler blocks, the connection and temp dir stay open — repeated requests from a single authenticated user exhaust the workers (DoS). Same pattern in `inspect/ffprobe_runner.rs`.

**Fix:** bound the wait with `tokio::time::timeout`, relying on `kill_on_drop(true)` (already set on the `Command`) to clean up the process when the future is dropped on timeout:

```rust
  use std::time::Duration;
  use tokio::time::timeout;

  const FFMPEG_TIMEOUT: Duration = Duration::from_secs(300);

  let status = match timeout(FFMPEG_TIMEOUT, child_process.wait()).await {
    Ok(result) => result?,
    Err(_) => return Err(ServerError::Processing("ffmpeg timed out".to_string())),
  };
  let _: Result<(), ServerError> = log_task.await?;
```

### 4. Empty-upload check tests the wrong value, so it never fires

`features/video/helpers.rs::read_video_to_file`:

```rust
  // Stream chunks directly from the request network buffer into the file
  while let Some(chunk) = field.chunk().await? {
    created_file.write_all(&chunk).await?;
  }

  // Ensure all data chunks are flushed to file
  created_file.flush().await?;

  if file_path.is_empty() {
    return Err(ServerError::DataError("Missing 'video' field".to_string()));
  }

  Ok(file_path)
```

`file_path` is built from `temp_dir.join(file_name).to_string_lossy()` — it is a filesystem path string, never empty regardless of how many bytes were actually streamed from the field. The intent (reject a 0-byte upload) never triggers: a multipart request with a `video` field carrying zero chunks silently creates an empty file on disk and gets passed on to `ffprobe`/`ffmpeg`, which then fail with a confusing codec/format error instead of a clear "empty upload" error. Same dead check repeated in `process/form_data_reader.rs::read` (`meta.file_path.is_empty()`).

**Fix:** count bytes actually written and check that instead:

```rust
pub async fn read_video_to_file(
  field: &mut Field<'_>,
  temp_dir: &Path,
) -> Result<String, ServerError> {
  let file_name = field
    .file_name()
    .ok_or(ServerError::DataError("Missing file_name".to_string()))?;
  let path = temp_dir.join(file_name);

  let mut created_file = File::create(&path).await?;
  let file_path = path.to_string_lossy().to_string();

  let mut written_bytes: u64 = 0;
  while let Some(chunk) = field.chunk().await? {
    written_bytes += chunk.len() as u64;
    created_file.write_all(&chunk).await?;
  }

  created_file.flush().await?;

  if written_bytes == 0 {
    return Err(ServerError::DataError("Uploaded video file is empty".to_string()));
  }

  Ok(file_path)
}
```

---

## General improvements

### 5. `println!` instead of `tracing` in ffmpeg logging

`features/video/process/ffmpeg_runner.rs`:

```rust
          "progress" => {
            println!("DB update -> Frame: {current_frame}, FPS: {current_fps}, Status: {value}");
          }
          _ => {}
        }
      } else if !line.is_empty() {
        println!("FFmpeg Log/Error: {}", line);
      }
```

ffmpeg progress/error lines are logged via `println!` instead of `tracing`, used everywhere else (`auth/repository.rs`, `auth/routes.rs`). In prod this bypasses the configured tracing subscriber/formatter/level filtering — either floods stdout unfiltered or is invisible to the log pipeline.

**Fix:**

```rust
use tracing::{debug, warn};
// ...
          "progress" => {
            debug!(frame = %current_frame, fps = %current_fps, status = %value, "ffmpeg progress");
          }
          _ => {}
        }
      } else if !line.is_empty() {
        warn!(%line, "ffmpeg log/error");
      }
```

---

### 6. Redundant DB round-trip on login

`features/auth/routes.rs::login`:

```rust
  let token = auth_state
    .auth_service
    .login(&payload.email, &payload.password)
    .await?;
  let user = auth_state.auth_service.get_by_email(&payload.email).await?;
  Ok(build_auth_response(StatusCode::OK, token.clone(), user)?)
```

`AuthService::login` already fetches the user via `get_by_email` internally, then the handler calls `get_by_email` again to build the response. Two SELECTs instead of one on the most-hit auth endpoint, no functional reason for it.

**Fix:** change `AuthService::login` to return `(String, User)` instead of just the token — it already has `user` in scope:

```rust
// service.rs
pub async fn login(&self, email: &str, password: &str) -> Result<(String, User), ApplicationError> {
    // ... unchanged validation ...
    let token = self
      .jwt_service
      .generate_token(user.id, user.username.clone())
      .map_err(|err| ApplicationError::Internal(err.to_string()))?;
    Ok((token, user))
}
```

```rust
// routes.rs
  let (token, user) = auth_state
    .auth_service
    .login(&payload.email, &payload.password)
    .await?;
  Ok(build_auth_response(StatusCode::OK, token, user)?)
```

---

### 7. Dead/confusing `#[from]` attribute on `DomainError`

`core/error.rs`:

```rust
#[derive(Debug, Error)]
#[from(sqlx::Error)]
pub enum DomainError {
  #[error("Access is forbidden")]
  Forbidden,
  // ...
  #[error("Sqlx error")]
  SqlxError(#[from] sqlx::Error),
}
```

The enum-level `#[from(sqlx::Error)]` attribute duplicates/obscures the field-level `#[from]` already on the `SqlxError` variant — that's not how `thiserror` actually derives `From` (the variant-level attribute is what does the work).

**Fix:** remove the enum-level attribute, keep only the variant-level one:

```rust
#[derive(Debug, Error)]
pub enum DomainError {
  #[error("Access is forbidden")]
  Forbidden,
  // ...
  #[error("Sqlx error")]
  SqlxError(#[from] sqlx::Error),
}
```

---

### 8. Redundant JWT expiration check

`features/auth/middleware.rs::authenticate_user`:

```rust
  let claims = jwt_service.verify_token(token).ok()?;
  let exp = chrono::DateTime::from_timestamp(claims.exp as i64, 0)?;

  if Utc::now().gt(&exp) {
    return None;
  }

  let user = auth_service.get(claims.user_id).await.ok()?;
```

`jwt_service.verify_token` already validates `exp` at decode time via `jsonwebtoken`'s `Validation::default()`. The manual `chrono` check that follows is redundant — not a bug, but dead logic that suggests the decode-time check isn't trusted, confusing about which check is authoritative.

**Fix:**

```rust
  let claims = jwt_service.verify_token(token).ok()?;
  let user = auth_service.get(claims.user_id).await.ok()?;
```
