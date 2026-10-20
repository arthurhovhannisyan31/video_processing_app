# Backend

Rust HTTP server for the [Video processing app](../../README.md), built with [Axum](https://github.com/tokio-rs/axum)
and PostgreSQL. It exposes endpoints for user auth (incl. email-based password reset), video inspection/processing via
`ffprobe`/`ffmpeg`, and streams job progress over a WebSocket.

## Tech stack

- **Runtime / framework:** Rust (edition 2024), [Tokio](https://tokio.rs/), [Axum](https://github.com/tokio-rs/axum)
  (multipart, ws)
- **Database:** PostgreSQL via [SQLx](https://github.com/launchbadge/sqlx) (compile-time checked queries, offline mode)
- **Auth:** JWT ([jsonwebtoken](https://crates.io/crates/jsonwebtoken)), password hashing
  with [argon2](https://crates.io/crates/argon2), cookie sessions stored in Postgres
  ([axum_session](https://crates.io/crates/axum_session))
- **Email:** [lettre](https://crates.io/crates/lettre) SMTP client for password reset emails
- **Video processing:** shells out to `ffprobe` (inspection) and `ffmpeg` (processing) as external processes
- **API docs:** [utoipa](https://github.com/juhaku/utoipa) generates an OpenAPI 3 spec, served at `/api/openapi` and
  also exported to `openapi.json` for the frontend client generator
- **Rate limiting:** [tower_governor](https://crates.io/crates/tower_governor) (enabled in production only)
- **Caching:** [mini-moka](https://crates.io/crates/mini-moka) in-memory cache for inspection results
- **Error tracking:** [Sentry](https://sentry.io)
- **Testing:** [axum-test](https://crates.io/crates/axum-test), [proptest](https://crates.io/crates/proptest)

## Project layout

```
src/
├── core/                     # cross-cutting concerns: config, app state, db pool, sessions, JWT, email, CORS,
│                             # rate limiter config, logging, OpenAPI spec, error types
├── features/
│   ├── auth/                 # register/login, password forgot/reset, user model & repository, JWT middleware
│   ├── system/               # health, readiness, openapi endpoints
│   └── video/                # video feature
│       ├── inspect/          # ffprobe integration: read upload, run ffprobe, map output
│       ├── process/          # ffmpeg integration: build args, run ffmpeg, report progress, stream response
│       ├── cache.rs          # in-memory inspection cache (1h TTL, 10min idle)
│       ├── service.rs        # inspect/process orchestration + WebSocket connections map
│       └── routes.rs         # /video/inspect, /video/jobs, /video/ws (WebSocket)
├── bin/openapi_generator.rs  # standalone bin that writes openapi.json
├── router.rs                 # route path constants + router assembly
├── http.rs                   # server bootstrap (bind + serve)
├── main.rs / lib.rs
migrations/                   # SQLx migrations (Postgres): users, password_reset_tokens
.sqlx/                        # SQLx offline query cache (commit it)
http-client/                  # .http request files for manual API testing
tests/                        # integration tests + fixtures
```

Each feature under `src/features/*` is self-contained: routes, DTOs, service/business logic, repository (DB access), and
per-feature state live together and are wired into `AppState` (`src/core/app_state.rs`). Handlers extract their
feature state via `FromRef` (e.g. `State<Arc<VideoState>>`). Errors from services (`DomainError` / `ServerError`) are
converted into `ApplicationError`, which maps them to an HTTP status and a `{ "message": ... }` body.

## API overview

All routes are nested under `/api`.

| Method | Path                    | Description                                                                 |
|--------|-------------------------|-----------------------------------------------------------------------------|
| GET    | `/health`               | Liveness check                                                              |
| GET    | `/ready`                | Readiness check (verifies DB connectivity)                                  |
| GET    | `/openapi`              | OpenAPI 3 spec (JSON)                                                       |
| POST   | `/auth/login`           | Authenticate, returns user + JWT                                            |
| POST   | `/auth/register`        | Create a user account, returns user + JWT (`201`)                           |
| POST   | `/auth/password/forgot` | Email a password reset link if the user exists (always `202`)               |
| POST   | `/auth/password/reset`  | Set a new password with a reset token (`204`)                               |
| POST   | `/video/inspect`        | Upload a video (`video` field), inspect it with `ffprobe`, cache metadata   |
| POST   | `/video/jobs`           | Upload a video (`video` + `operation` fields), run `ffmpeg`, return the file |
| GET/WS | `/video/ws`             | WebSocket for live job progress (`VideoStateProgress` messages)             |

Notes:

- Video routes are tied to the **session cookie**, not to a JWT: the session id links an inspection, a processing job
  and the WebSocket that receives its progress. Only one WebSocket per session is allowed. The Bearer-token middleware
  in `features/auth/middleware.rs` exists but is not applied to any route yet.
- `/video/jobs` reuses cached inspection metadata (keyed by file name + session id) to compute progress, and runs
  `ffprobe` itself on a cache miss.
- Supported input: `.mp4` files. Supported operation: `compress` (libx264 CRF 23 + AAC 128k).
- Password reset tokens are stored hashed, expire after `BACKEND_PASSWORD_RESET_TTL_MIN` minutes (default 30), and can
  be requested at most once per minute per user. A successful reset invalidates JWTs issued before it.
- Video routes enforce a max upload body size. In production, auth and video routes are rate limited per IP via
  `tower_governor`.

## Prerequisites

- Rust (stable, edition 2024 toolchain — see `configs/docker/backend.dockerfile` for the pinned version)
- PostgreSQL
- `ffmpeg` / `ffprobe` available on `PATH`
- [`cargo-nextest`](https://nexte.st/) and [`sqlx-cli`](https://github.com/launchbadge/sqlx/tree/main/sqlx-cli) for
  local dev (`cargo install cargo-nextest sqlx-cli`)

## Configuration

The server reads its config from environment variables (see [`.env.template`](../../.env.template) at the repo root).
Locally it loads a `.env` file via `dotenvy` unless `CONTAINER=true` is set (containers get env vars injected directly).

Key variables:

| Variable                                                                | Purpose                                                                 |
|-------------------------------------------------------------------------|-------------------------------------------------------------------------|
| `BACKEND_HOST`, `BACKEND_HTTP_PORT`                                     | bind address/port (default `localhost:8080`)                            |
| `DATABASE_URL`                                                          | Postgres connection string                                              |
| `BACKEND_DB_MAX_CONNECTIONS`                                            | DB pool size (default 10)                                               |
| `BACKEND_JWT_SECRET`                                                    | JWT signing secret                                                      |
| `BACKEND_CORS_ORIGINS`                                                  | comma-separated list of allowed origins                                 |
| `BACKEND_VIDEO_MAX_BODY_SIZE`                                           | max multipart upload size in bytes (default 100 MB)                     |
| `BACKEND_VIDEO_RATE_LIMIT_PERIOD_SEC` / `BACKEND_VIDEO_RATE_LIMIT_SIZE` | video rate limiter period/burst, production only (default 60 / 100)     |
| `BACKEND_VIDEO_INSPECT_TIMEOUT` / `BACKEND_VIDEO_PROCESS_TIMEOUT`       | `ffprobe`/`ffmpeg` timeouts in seconds (default 20 / 300)               |
| `BACKEND_VIDEO_MOCK_PASSWORD_HASH`                                      | dummy hash used to defend login against user-enumeration timing attacks |
| `BACKEND_FRONTEND_URL`                                                  | frontend base URL used in reset links (default `http://localhost:3000`) |
| `BACKEND_PASSWORD_RESET_TTL_MIN`                                        | reset token lifetime in minutes (default 30)                            |
| `BACKEND_SMTP_HOST`, `BACKEND_SMTP_PORT`, `BACKEND_SMTP_TLS`            | SMTP server; empty host falls back to Mailpit (`mailpit:1025`, no TLS)  |
| `BACKEND_SMTP_USERNAME`, `BACKEND_SMTP_PASSWORD`, `BACKEND_SMTP_FROM`   | SMTP credentials and sender address                                     |
| `SENTRY_DSN`                                                            | Sentry DSN (required)                                                   |
| `IS_PRODUCTION`                                                         | toggles production-only behavior (rate limiting, secure cookies)        |

`DATABASE_URL`, `BACKEND_JWT_SECRET`, `BACKEND_CORS_ORIGINS`, `BACKEND_VIDEO_MOCK_PASSWORD_HASH` and `SENTRY_DSN` are
required; the rest have defaults.

## Running locally

From the repo root (see the [root README](../../README.md) for the full Makefile-driven workflow):

```sh
# from repo root
cp .env.template .env   # fill in the values
make backend            # cd modules/backend && cargo run
```

Or directly from this directory:

```sh
cargo run
```

The server runs pending SQLx migrations automatically on startup (`run_migrations` in `main.rs`).

Password reset emails go to [Mailpit](https://mailpit.axllent.org/) when `BACKEND_SMTP_HOST` is empty. The dev compose
file starts it; its web UI is at http://localhost:8025. When running the server outside Docker, set
`BACKEND_SMTP_HOST=localhost` (the default host `mailpit` only resolves inside the compose network).

## Development commands

Run from the repo root via `make`, or `cd modules/backend` and use `cargo` directly:

```sh
make backend           # cargo run
make check              # cargo clippy --all-features --all-targets
make format              # cargo fmt
make format-check        # CI-style fmt check (configs/scripts/cargo-fmt.sh)
make test                 # cargo nextest run
make audit                 # cargo audit
make cargo-update           # cargo update
make generate-openapi        # cargo run --bin openapi-generator -> openapi.json
make prepare-backend          # cargo sqlx prepare (regenerate .sqlx offline query cache)
```

`cargo sqlx prepare` must be re-run (and the `.sqlx/` output committed) whenever a query changes, since the server
builds with `SQLX_OFFLINE=true` in CI/containers. Use the checked macros (`query_as!`, `query_scalar!`): plain
`sqlx::query` / `sqlx::query_as` are disallowed in `clippy.toml`.

After changing a route or DTO, run `make generate-openapi` and `make prepare-frontend-local` so the frontend client
stays in sync.

## Testing

```sh
cargo nextest run                        # all tests
cargo nextest run --test video_process   # one integration test file
cargo nextest run -E 'test(login)'       # tests whose name matches
```

Tests need a running Postgres reachable via `DATABASE_URL` and `ffmpeg`/`ffprobe` on `PATH`. Integration tests in
`tests/` use `#[sqlx::test(fixtures("create_user"))]`, which creates a fresh database per test; fixtures (SQL and sample
media) live in `tests/fixtures/`. `tests/common/utils.rs` builds the router with a mock email sender. Property-based test
regressions are tracked under `proptest-regressions/`.

## Docker

The production image is built from [`configs/docker/backend.dockerfile`](../../configs/docker/backend.dockerfile): a
multi-stage build that compiles the release binary against `rust:1.97` and ships it on `debian:trixie-slim` with
`ffmpeg` installed. See the root [`docker-compose.dev.yml`](../../docker-compose.dev.yml) / [
`docker-compose.prod.yml`](../../docker-compose.prod.yml) for how it's composed with Postgres and nginx.

## Notes

- `ffprobe` JSON output format reference: https://raw.githubusercontent.com/FFmpeg/FFmpeg/master/doc/ffprobe.xsd
