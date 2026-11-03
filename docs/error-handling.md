# Error handling (backend)

All error models live in `modules/backend/src/core/error.rs` and are built with `thiserror`.

## Overview

Errors are layered. Lower-level errors convert into higher-level ones with `?` (via `From`), and only
the top layer is turned into an HTTP response:

```text
InputError   ──► ServerError ──┐
StartupError ──► ServerError   ├──► ApplicationError ──► HTTP status + ErrorBody
DomainError  ──────────────────┘
```

| Type               | Layer             | Responsibility                                                   |
|--------------------|-------------------|------------------------------------------------------------------|
| `ErrorBody`        | HTTP contract     | JSON body of error responses                                     |
| `ApplicationError` | HTTP boundary     | Maps every failure to a status code and response body            |
| `DomainError`      | Business / data   | Auth feature and user repository errors                          |
| `ServerError`      | Infrastructure    | Catch-all for IO, processes, libraries, startup and input errors |
| `InputError`       | Client input      | Invalid or missing data sent by the client (video feature)       |
| `StartupError`     | Boot              | Config, `.env`, migrations and session store failures            |

## `ErrorBody`

```json
{ "message": "..." }
```

- The only error body shape the API returns.
- Derives `ToSchema`, so it is part of the OpenAPI spec (`body = ErrorBody` in `#[utoipa::path]`
  responses, e.g. `features/video/routes.rs`) and of the generated frontend client.
- Integration tests deserialize it to assert error messages (`tests/video_inspect.rs`,
  `tests/video_process.rs`, `tests/video_body_limit.rs`).

## `ApplicationError`

The HTTP boundary. Route handlers return `Result<_, ApplicationError>`, and it is the type that owns
the `IntoResponse` mapping. Also returned directly by `AuthService` (`features/auth/service.rs`) and
the system routes, and used as the result type in some integration tests.

| Variant                     | Status | Body                                                     |
|-----------------------------|--------|----------------------------------------------------------|
| `BadRequest(msg)`           | 400    | `ErrorBody { msg }`                                      |
| `Validation(ValidationErrors)` | 400 | `ErrorBody` with the `validator` message |
| `Unauthorized`              | 401    | empty                                                    |
| `Forbidden`                 | 403    | empty                                                    |
| `NotFound(msg)`             | 404    | `ErrorBody { msg }`                                      |
| `Conflict(msg)`             | 409    | `ErrorBody { msg }`                                      |
| `Internal(msg)`             | 500    | `ErrorBody { "Internal server error" }`; `msg` is logged |
| `ServiceUnavailable(msg)`   | 503    | empty; `msg` is logged                                   |

Notes:

- `Internal` never leaks its message to the client — it is only written to logs with `error!`.
- `Validation` comes from `payload.validate()?` on DTOs deriving `validator::Validate` (auth routes).
- `ServiceUnavailable` is used by the readiness probe (`features/system/routes.rs`) when the DB check
  fails.

## `DomainError`

Business and persistence errors of the auth feature. Returned by the `UserRepository` trait and its
Postgres implementation (`features/auth/repository.rs`), e.g. a unique constraint violation on
username/email becomes `UserAlreadyExists`.

| Variant              | Maps to                              |
|----------------------|--------------------------------------|
| `Forbidden`          | `ApplicationError::Forbidden`        |
| `InvalidCredentials` | `ApplicationError::Unauthorized`     |
| `UserAlreadyExists`  | `ApplicationError::Conflict`         |
| `UserNotFound(id)`   | `ApplicationError::NotFound`         |
| `Internal(msg)`      | `ApplicationError::Internal`         |
| `SqlxError(err)`     | `ApplicationError::Internal`         |

`Forbidden`, `InvalidCredentials` and `UserNotFound` are not constructed anywhere yet; the service
currently returns the matching `ApplicationError` variants directly.

## `ServerError`

Infrastructure and internal errors for code that is not an HTTP handler: video service, `ffmpeg` /
`ffprobe` runners and helpers, JWT and password hashing (`core/jwt.rs`), logging, session store,
router and rate limiter (`core/governor.rs`) setup, `main.rs` / `http.rs`.

It wraps third-party errors via `#[from]` (`io`, `sqlx`, `lettre`, `jsonwebtoken`, `argon2`,
`serde_json`, int/float parsing, tokio `JoinError`, axum WebSocket errors, tracing filter, `anyhow`)
and nests `InputError` and `StartupError`. Own variants:

- `Processing(String)` — `ffmpeg` / `ffprobe` failed to spawn, timed out or exited with an error.
- `StaleCache(String)` — cached inspection data is outdated (currently unused).

Conversion to `ApplicationError`:

| Variant                                  | Maps to                        | Message exposed |
|------------------------------------------|--------------------------------|-----------------|
| `InputError`, `StaleCache`, `OtherError` | `ApplicationError::BadRequest` | yes             |
| everything else                          | `ApplicationError::Internal`   | no (logged)     |

`ServerError` also implements `IntoResponse` by delegating to `ApplicationError`, so it can be
returned from a handler or response builder directly (e.g. `build_auth_response` in
`features/auth/routes.rs`).

`OtherError(anyhow::Error)` maps to 400, so use it only for client-caused problems; it is currently
used for invalid rate limiter config.

## `InputError`

Invalid client input in the video feature. Always ends up as a 400 with its message shown to the
user.

- `DataError(String)` — missing or invalid form fields, bad file name/extension, empty upload, zero
  duration, unparseable `ffprobe` output.
- `MissingMediaData(String)` — `ffprobe` output has no `format` or `streams`.
- `Multipart(MultipartError)` — failed reading the multipart body.

Raised in `features/video/{inspect,process}/form_data_reader.rs`, `features/video/helpers.rs`,
`features/video/inspect/{types,utils}.rs` and `features/video/process/types.rs`. It travels as
`ServerError::InputError` (`#[error(transparent)]`, so the message is unchanged).

## `StartupError`

Boot-time failures. Converted into `ServerError::StartupError` and abort `main`; it does not reach
clients.

- `VarError(String)` — missing or invalid env variable (`core/app_config.rs`, `core/email.rs`).
- `Dotenv(dotenvy::Error)` — failed loading `.env` (only when `CONTAINER` is not `true`).
- `MigrationError(MigrateError)` — failed running migrations (`core/database.rs`).
- `SessionError(SessionError)` — failed creating the session store (`core/session.rs`).

## Guidelines for new code

- Handlers return `Result<_, ApplicationError>`; let `?` convert `ServerError` / `DomainError`.
- Client-caused problems go to `InputError` (or `ApplicationError::BadRequest` / `Validation`);
  everything else maps to `Internal`, which keeps details in logs and out of responses.
- New third-party error: add a `#[from]` variant to `ServerError` and an arm in
  `From<ServerError> for ApplicationError`. The match is exhaustive, so the compiler enforces this.
- Persistence/business errors of a feature belong in `DomainError` and are mapped in
  `From<DomainError> for ApplicationError`.
- Document error statuses in `#[utoipa::path]` `responses(...)` with `body = ErrorBody`, then
  regenerate the OpenAPI spec and frontend client.
- Errors that cannot be returned (spawned tasks such as `forgot_password`, the WebSocket loop in
  `features/video/service.rs`) are logged with `tracing::error!` / `warn!`.
