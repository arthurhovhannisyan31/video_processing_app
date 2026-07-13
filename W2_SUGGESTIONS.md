# Backend idiomatic-review suggestions (not yet applied)

## Bugs

### 1. `UserAlreadyExists` reports id `0` instead of the actual user

`data/user_repository.rs:56` — on unique-constraint violation:

```rust
DomainError::UserAlreadyExists(user.id)
```

`user.id` is always `0` at this point: `User::new()` (`domain/user.rs:14`) hardcodes `id: 0` because the real id only exists after the DB insert. So every conflict error reads "user 0 already exists" no matter who registered.

**Fix:** report something you actually have before insert — email or username:

```rust
DomainError::UserAlreadyExists(user.email.clone())
```
(and change the variant to carry a `String` instead of `i64`).

Ties into #20/#23: switching `id` to `UUID`/`UserId` doesn't fix this on its own — a random/nil UUID generated before insert is just as meaningless in the error as `0` is today. Same fix applies regardless: report email/username, not the id.

---

### 2. Error responses lose their JSON body

`presentation/auth.rs` handlers are typed:

```rust
async fn login(...) -> Result<impl IntoResponse, StatusCode> {
    let token = auth_state.auth_service.login(&payload.email, &payload.password).await?;
    ...
```

`login()` returns `Result<_, ApplicationError>`. Using `?` against a `Result<_, StatusCode>` function forces a conversion through:

```rust
// application/error.rs:79
impl From<ApplicationError> for StatusCode { ... }
```

But `ApplicationError` already has a proper `IntoResponse` impl (`error.rs:28`) that returns a JSON body like `{"message": "..."}`. Going through `StatusCode` throws that body away — client gets a bare status code, no message.

**Fix:**
```rust
async fn login(...) -> Result<impl IntoResponse, ApplicationError> {
```
on every handler, then delete the now-unused `From<ApplicationError> for StatusCode` impl in `error.rs`.

Same body-loss family as #5 below — `Validation`'s own `IntoResponse` arm drops its message too, even without going through `StatusCode` at all.

---

### 3. DB errors get mislabeled as "Unauthorized"

`application/auth_service.rs:57`:

```rust
let Ok(user) = self.get_by_email(&email.to_lowercase()).await else {
    return Err(ApplicationError::Unauthorized);
};
```

`get_by_email` fails for two different reasons: user genuinely not found (should be 401), or the DB call itself errored (should be 500). This code treats both the same.

**Why it matters:** a real outage/DB error silently looks like "wrong password" to the client and to you in the logs — masks incidents.

**Fix:**
```rust
let user = match self.get_by_email(&email.to_lowercase()).await {
    Ok(user) => user,
    Err(ApplicationError::NotFound(_)) => return Err(ApplicationError::Unauthorized),
    Err(err) => return Err(err),
};
```

Same pattern a few lines down, `auth_service.rs:61-62`:
```rust
let password_valid = verify_password(password, &user.password_hash)
    .map_err(|_| ApplicationError::Unauthorized)?;
```
`verify_password` only realistically errors on a corrupt/unparseable hash in DB, not "wrong password" — that case is also collapsed into a plain 401, hiding a data-integrity problem behind what looks like a login mistake. Same fix idea: distinguish "hash didn't match" from "hash itself is broken."

---

### 4. `username` has no uniqueness constraint

`migrations/20260101000001_create_schema_users.sql` only indexes username:

```sql
CREATE INDEX IF NOT EXISTS idx_users_username ON users (username);
```

No `UNIQUE`. Meanwhile `data/user_repository.rs` already checks for a `USERS_USERNAME` constraint violation on insert — that code path can never fire because the constraint doesn't exist. Two people can register the same username right now.

**Fix:**
```sql
username TEXT NOT NULL UNIQUE,
```

---

### 5. `ApplicationError::Validation` drops its own message

`application/error.rs:52-54`:

```rust
ApplicationError::Validation(_) => {
    StatusCode::BAD_REQUEST.into_response()
}
```

Every other variant with a `String` payload (`BadRequest`, `Conflict`, `Internal`, `NotFound`) builds `{"message": msg}` into the response body. `Validation` alone captures the message and throws it away (`_`) — client gets a bare 400 with no indication of what failed validation.

**Fix:**
```rust
ApplicationError::Validation(msg) => {
    (StatusCode::BAD_REQUEST, json!({"message": msg}).to_string()).into_response()
}
```

---

### 6. Config error message references the wrong env var name

`infrastructure/config.rs:26-31`:

```rust
let http_port = env::var("BACKEND_HTTP_PORT")
    .unwrap_or_else(|_| "8080".into())
    .parse()
    .map_err(|e| {
        ServerError::VarError(format!("Invalid SERVER_HTTP_PORT variable: {e}"))
    })?;
```

Var actually read is `BACKEND_HTTP_PORT`, but the error message says `SERVER_HTTP_PORT`. Anyone debugging a bad port value gets pointed at a variable name that doesn't exist.

**Fix:** `format!("Invalid BACKEND_HTTP_PORT variable: {e}")`.

---

### 7. Postgres pool sized at 10000 connections

`infrastructure/database.rs:8`:

```rust
let pool = PgPoolOptions::new()
    .max_connections(10000)
    .min_connections(2)
```

Default Postgres `max_connections` server-side is 100. A pool configured for 10000 will happily try to open way more connections than the DB allows the moment load increases, and will hit `too many clients already` errors.

**Fix:** pick something sane and make it configurable:
```rust
.max_connections(app_config.db_max_connections) // e.g. default 10-20
```

---

### 8. Full request logged on every call — leaks the auth token

`presentation/init.rs:26-30`:

```rust
.layer(TraceLayer::new_for_http().on_request(
    |request: &axum::extract::Request<_>, _span: &tracing::Span| {
        info!("{:?}", request);
    },
))
```

`Debug` on an `axum`/`http` `Request` includes all headers — including `Authorization: Bearer <jwt>` and the `Cookie` header set by `get_auth_cookie`. Every authenticated request writes the caller's live JWT straight into the logs at `info` level.

**Why it matters:** anyone with log access (or a leaked log file) can lift a valid token and impersonate that user until it expires.

**Fix:** log method/path/status instead of the whole request, or explicitly redact sensitive headers:
```rust
info!(method = %request.method(), path = %request.uri().path(), "request");
```

---

### 9. Cookie hardcoded `Secure` — breaks local HTTP dev

`presentation/utils.rs`:

```rust
cookie.set_secure(true);
```

Browsers refuse to send a `Secure` cookie over plain `http://`. Fine in production (behind TLS), but local dev over `http://localhost` silently never gets the cookie sent back.

**Fix:** derive from config/environment instead of hardcoding:
```rust
cookie.set_secure(!app_config.run_locally); // ties into #11
```

---

### 10. Error detail silently dropped in `ServerError` messages

`infrastructure/error.rs`:

```rust
#[error("Sqlx error")]
SqlxError(String),
#[error("Failed to read env variable")]
VarError(String),
```

Both variants carry a `String` with the actual detail (e.g. `format!("Failed connecting to Postgres: {e}")` in `database.rs`), but the `#[error(...)]` message never references `{0}` — so `.to_string()`/`Display` on these just prints the generic label, dropping the useful part. (Still visible via `{:?}` / `Debug`, which is how `main`'s `Result` gets printed on exit — but any other place using `Display` loses it.)

**Fix:**
```rust
#[error("Sqlx error: {0}")]
SqlxError(String),
#[error("Failed to read env variable: {0}")]
VarError(String),
```

---

## Style

### 11. Misleading boolean name in config

`infrastructure/config.rs:17-19`:

```rust
let docker_container = env::var("DOCKER_CONTAINER")
    .unwrap_or("false".to_owned())
    .eq("false");
if docker_container {
    dotenvy::dotenv()?;
}
```

`docker_container` is `true` exactly when the app is **not** running in Docker (`DOCKER_CONTAINER == "false"`). The name says the opposite of what the value means — next person reading this will misread it.

**Fix:** rename to what it actually represents:
```rust
let run_locally = env::var("DOCKER_CONTAINER")
    .unwrap_or("false".to_owned())
    .eq("false");
if run_locally {
    dotenvy::dotenv()?;
}
```

---

### 12. Manual header split instead of `split_once`

`presentation/middleware.rs:28-41`:

```rust
let values = auth_header.split(" ").collect::<Vec<&str>>();
if values.len() != 2 {
    return None;
}
let token = values.get(1)?;
Some(token)
```

Allocates a `Vec` and does a length check just to grab the part after the first space. `str::split_once` does this directly, no allocation:

```rust
fn get_token(headers: &HeaderMap) -> Option<&str> {
    let auth_header = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    let (scheme, token) = auth_header.split_once(' ')?;
    (scheme.eq_ignore_ascii_case("bearer")).then_some(token)
}
```
(bonus: this version also actually checks the scheme is `Bearer`, which the current code doesn't).

---

### 13. Email lowercased twice

`application/auth_service.rs`:

```rust
pub async fn login(&self, email: &str, password: &str) -> Result<String, ApplicationError> {
    let Ok(user) = self.get_by_email(&email.to_lowercase()).await else { ... };
```

and inside `get_by_email` itself:

```rust
pub async fn get_by_email(&self, email: &str) -> Result<User, ApplicationError> {
    self.repo.find_by_email(&email.to_lowercase()).await?...
```

`get_by_email` already normalizes the email, so callers don't need to. Harmless today, but it's a sign the invariant ("emails are always normalized before reaching the repo") isn't clearly owned by one place — next call site added might forget it, or double-lowercase like this one for no reason.

**Fix:** drop the `.to_lowercase()` in `login`, keep it only inside `get_by_email`.

---

### 14. Duplicated response-building code

`presentation/auth.rs` — `login` and `register` both end with the identical block:

```rust
Ok(
    Response::builder()
        .status(StatusCode::CREATED)
        .header("Access-Control-Allow-Credentials", "true")
        .header(SET_COOKIE, get_auth_cookie(&token).to_string())
        .body(Body::from(response))
        .map_err(|err| ApplicationError::Internal(err.to_string()))?,
)
```

**Fix:** extract once:
```rust
fn build_auth_response(user: AuthenticatedUser, token: String) -> Result<Response<Body>, ApplicationError> {
    let body = json!(AuthResponse { user, token: token.clone() }).to_string();
    Response::builder()
        .status(StatusCode::CREATED)
        .header(SET_COOKIE, get_auth_cookie(&token).to_string())
        .body(Body::from(body))
        .map_err(|err| ApplicationError::Internal(err.to_string()))
}
```
then both handlers just call `build_auth_response(authenticated_user, token)`.

---

### 15. CORS header hardcoded in handlers

Same block above also sets:
```rust
.header("Access-Control-Allow-Credentials", "true")
```
by hand, per handler, as a raw string. Confirmed 100% redundant: `infrastructure/cors.rs` already does `.allow_credentials(true)` on the global `CorsLayer` — the manual header in `auth.rs` duplicates something already set for every route.

**Fix:** configure `.allow_credentials(true)` once on the `CorsLayer` in `infrastructure/cors.rs`, remove the per-handler header entirely.

---

### 16. Silent `.unwrap()` on time arithmetic

`infrastructure/jwt.rs:38-41`:

```rust
exp: chrono::Utc::now()
    .checked_add_signed(chrono::Duration::hours(TOKEN_EXPIRATION_HOURS))
    .unwrap() as usize,
```

`checked_add_signed` returns `None` only on overflow, which won't realistically happen with a sane `TOKEN_EXPIRATION_HOURS` — but a bare `.unwrap()` gives future readers no clue why it's safe, and panics with an unhelpful message if that assumption ever breaks (e.g. someone sets `TOKEN_EXPIRATION_HOURS` to a huge value).

**Fix:** same behavior, but self-documenting:
```rust
.expect("token expiration duration does not overflow")
```

---

### 17. `register` redundantly re-runs `login` after already creating the user

`presentation/auth.rs:71-74`:

```rust
let user = app_state.auth_service.register(...).await?;
...
let token = app_state.auth_service.login(&payload.email, &payload.password).await?;
```

`register()` already hashes the password and inserts the user (has `user.id` in hand). The handler then calls `login()` with the plaintext password, which does a *second* `get_by_email` DB round-trip and a full `argon2::verify_password` against the hash just created — pure repeated work on every single signup.

**Fix:** generate the token directly from the `user` returned by `register`, no second `login` call:
```rust
let token = app_state.auth_service.jwt_service
    .generate_token(user.id, user.username.clone())
    .map_err(|err| ApplicationError::Internal(err.to_string()))?;
```
(may need to expose `jwt_service` or add a small `AuthService` helper that just mints a token for a known `User`).

---

### 18. Auth cookie is set but never read anywhere

`login`/`register` set a `SET_COOKIE` header via `get_auth_cookie`, but `presentation/middleware.rs get_token()` only ever reads the `Authorization` header — nothing in the codebase parses the cookie back out. The cookie is dead weight: written on every login, never consumed.

**Fix:** either read the cookie as a fallback in `get_token` (real cookie-based auth), or drop `get_auth_cookie`/`SET_COOKIE` entirely and rely on the JSON `token` field the client already gets back.

Side note if you go the "keep cookie-based auth" route: the cookie is set to live 7 days (`utils.rs` — `Duration::days(7)`) while the JWT inside it expires in 24h (`infrastructure/constants.rs` — `TOKEN_EXPIRATION_HOURS = 24`). Cookie would sit there for 6 more days holding a dead token. Align the two durations once the cookie is actually consumed.

---

## Migration SQL

### 19. Column definitions not aligned

Current:
```sql
CREATE TABLE IF NOT EXISTS users
(
    id  BIGSERIAL PRIMARY KEY,
    username TEXT NOT NULL,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at timestamptz NOT NULL DEFAULT NOW(),
    ...
```

`id` has 2 spaces after it, everything else 1, types don't line up in a column — harder to scan visually.

**Fix** (also folds in #4, the missing `UNIQUE` on username):
```sql
CREATE TABLE IF NOT EXISTS users
(
    id            BIGSERIAL   PRIMARY KEY,
    username      TEXT        NOT NULL UNIQUE,
    email         TEXT        NOT NULL UNIQUE,
    password_hash TEXT        NOT NULL,
    created_at    timestamptz NOT NULL DEFAULT NOW(),
    CONSTRAINT chk_name_len          CHECK (length(username) <= 255),
    CONSTRAINT chk_email_len         CHECK (length(email) <= 255),
    CONSTRAINT chk_password_hash_len CHECK (length(password_hash) <= 255)
);
```

---

### 20. `id` is BIGSERIAL but gets exposed to clients

`id` ends up as `user_id` in JWT claims (`infrastructure/jwt.rs`) and in API responses (`AuthenticatedUser`) — so it's client-visible. BIGSERIAL is sequential: user 1, 2, 3... anyone can guess/enumerate other users' ids just by trying numbers.

**Why it matters:** enumeration is a real (low-severity but free-to-fix) info-leak — attacker can count your users, probe `/users/{id}` endpoints sequentially if any get added later.

**Fix:** `sqlx`'s `uuid` feature is already enabled in `Cargo.toml`, just unused:
```sql
id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
```
`gen_random_uuid()` is built-in since Postgres 13; on older versions it needs the `pgcrypto` extension (`CREATE EXTENSION IF NOT EXISTS pgcrypto;`) — check the target PG version.

(BIGSERIAL is still fine for tables that are never exposed outside the backend.)

---

## Cargo.toml

### 21. Inconsistent version pin: `cookie = "0.18.1"`

Every other dependency pins `"major.minor"` only:
```toml
chrono = { version = "0.4", ... }
tower = { version = "0.5", ... }
```
`cookie` alone adds a patch digit. Not wrong, just inconsistent style in the same file.

**Fix:** `cookie = "0.18"`.

---

### 22. Dependency list isn't alphabetized

```toml
anyhow = "1.0"
axum = ...
axum-extra = ...
axum-test = ...
argon2 = "0.5"   # <- should be before axum
async-trait = ...
```

Not a bug, just makes it slightly harder to scan for a dep or spot a duplicate at a glance.

**Fix:** sort alphabetically (or run `cargo-sort` if you want it automated/enforced in CI).

---

## Easy wins

### 23. Typed id instead of raw `i64`

Right now `User::id: i64` is just a plain integer — nothing stops passing a `PostId` (once that entity exists) where a `UserId` is expected; the compiler can't catch it, only a runtime bug or a test would.

**Fix:** a small newtype, ties in with #20 (id becoming a UUID):
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, sqlx::Type, Serialize, Deserialize)]
#[sqlx(transparent)]
pub struct UserId(pub Uuid);
```
Now `fn get(&self, id: UserId)` can never accidentally be called with a `PostId` — it won't compile.

---

### 24. Pure helper functions with zero test coverage

`get_token()` (`presentation/middleware.rs`) and the env parsing in `AppConfig::from_env()` are both plain, deterministic functions (no I/O beyond reading env vars) — cheap to unit test, currently untested.

**Example for `get_token`:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{HeaderMap, HeaderValue, header};

    #[test]
    fn extracts_bearer_token() {
        let mut headers = HeaderMap::new();
        headers.insert(header::AUTHORIZATION, HeaderValue::from_static("Bearer abc123"));
        assert_eq!(get_token(&headers), Some("abc123"));
    }

    #[test]
    fn rejects_missing_header() {
        assert_eq!(get_token(&HeaderMap::new()), None);
    }

    #[test]
    fn rejects_malformed_header() {
        let mut headers = HeaderMap::new();
        headers.insert(header::AUTHORIZATION, HeaderValue::from_static("abc123"));
        assert_eq!(get_token(&headers), None);
    }
}
```

**Why it matters:** this is exactly the kind of function a refactor (like #12 above) can silently break — a couple of cheap tests catch that immediately instead of at runtime in prod.
