# Backend standards (`modules/backend`, Rust/Axum)

## Structure

- Features live in `src/features/<name>/` with `routes.rs`, `dto.rs`, `service.rs`, `repository.rs`, `model.rs`,
  `state.rs`, `constants.rs` (only the ones needed). Register the module in `src/features/mod.rs`.
- Route path constants go in `src/router.rs` (`routes::*`); feature routers are nested under `/api` there.
- New feature state: add a field to `AppState` (`src/core/app_state.rs`) and wire it in `AppState::new` **and** in
  `tests/common/utils.rs::setup_router_with_email` (it builds `AppState` by hand). Extract it in handlers with
  `State<Arc<FeatureState>>` via `FromRef`.
- Shared infrastructure (config, DB, JWT, email, sessions, errors, openapi) lives in `src/core/`.

## Errors

Read `docs/error-handling.md` before adding or changing errors. Summary:

- Handlers return `Result<_, ApplicationError>`; let `?` convert `ServerError` / `DomainError`.
- Client-caused problems → `InputError` (or `ApplicationError::BadRequest` / `Validation`). Everything else maps
  to `Internal`, which logs the message and hides it from the client.
- New third-party error → `#[from]` variant on `ServerError` + arm in `From<ServerError> for ApplicationError`.
- Persistence/business errors → `DomainError`.
- Errors that can't be returned (spawned tasks, WebSocket loop) are logged with `tracing::error!` / `warn!`.

## API contract (OpenAPI)

- DTOs derive `Serialize`/`Deserialize`, `utoipa::ToSchema` and, for input, `validator::Validate`; call
  `payload.validate()?` in the handler. Document constraints with `#[schema(...)]` (see `features/auth/dto.rs`).
- Every handler has `#[utoipa::path]` with all responses, errors as `body = ErrorBody`, and is registered in
  `src/core/openapi.rs`.
- After any route/DTO change: `make generate-openapi && make prepare-frontend-local`, then fix the frontend.

## Patterns

- Rate limiting only when `app_config.is_production`; push the limiter into `system_state.rate_limiters` so it is
  cleaned up (pattern: `features/video/routes.rs`, `core/governor.rs`).
- Config comes from env through `AppConfig::from_env` (`src/core/app_config.rs`); missing vars are
  `StartupError::VarError`. Document new env vars in `modules/backend/README.md`.
- Video: uploads go to a `TempDir`; `ffmpeg`/`ffprobe` run as child processes with timeouts; the session id links
  inspect, job and WebSocket; inspect metadata is cached in `mini_moka`.
- Logging with `tracing` (structured fields, e.g. `info!(user_id = %id, "user created")`). Dates with `chrono`.

## Style

- `cargo fmt` (`.rustfmt.toml`): 2-space indent, max width 100, imports grouped std / external / crate,
  module granularity.
- `cargo clippy --all-features --all-targets` must be clean. `unwrap` / `expect` / `panic` only in tests
  (`clippy.toml`).
- No plain `sqlx::query` / `query_as` — see `database.md`.
