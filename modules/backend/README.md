# Backend

Rust HTTP server for the [Video processing app](../../README.md), built with [Axum](https://github.com/tokio-rs/axum)
and PostgreSQL. It exposes endpoints for user auth, video inspection/processing via `ffprobe`/`ffmpeg`, and streams job
progress over a WebSocket.

## Tech stack

- **Runtime / framework:** Rust (edition 2024), [Tokio](https://tokio.rs/), [Axum](https://github.com/tokio-rs/axum)
  (multipart, ws)
- **Database:** PostgreSQL via [SQLx](https://github.com/launchbadge/sqlx) (compile-time checked queries, offline mode)
- **Auth:** JWT ([jsonwebtoken](https://crates.io/crates/jsonwebtoken)), password hashing
  with [argon2](https://crates.io/crates/argon2)
- **Video processing:** shells out to `ffprobe` (inspection) and `ffmpeg` (processing) as external processes
- **API docs:** [utoipa](https://github.com/juhaku/utoipa) generates an OpenAPI 3 spec, served at `/api/openapi` and
  also exported to `openapi.json` for the frontend client generator
- **Rate limiting:** [tower_governor](https://crates.io/crates/tower_governor) (enabled in production only)
- **Testing:** [axum-test](https://crates.io/crates/axum-test), [proptest](https://crates.io/crates/proptest)

## Project layout

```
src/
├── core/                 # cross-cutting concerns: config, app state, db pool, JWT, CORS, logging, error types
├── features/
│   ├── auth/             # register/login, user model & repository, JWT middleware
│   ├── system/            # health, readiness, openapi endpoints
│   └── video/            # video feature
│       ├── inspect/       # ffprobe integration: read upload, run ffprobe, map output, cache result
│       ├── process/       # ffmpeg integration: build args, run ffmpeg, stream response
│       └── routes.rs       # /video/inspect, /video/jobs, /video/ws/{user_id} (WebSocket)
├── bin/openapi_generator.rs  # standalone bin that writes openapi.json
├── router.rs              # route table + router assembly
├── http.rs                # server bootstrap (bind + serve)
├── main.rs / lib.rs
migrations/                # SQLx migrations (Postgres)
tests/                     # integration tests + fixtures
```

Each feature under `src/features/*` is self-contained: routes, DTOs, service/business logic, repository (DB access), and
per-feature state live together and are wired into `AppState` (`src/core/app_state.rs`).

## API overview

All routes are nested under `/api`.

| Method | Path                  | Description                                               |
|--------|-----------------------|-----------------------------------------------------------|
| GET    | `/health`             | Liveness check                                            |
| GET    | `/ready`              | Readiness check (verifies DB connectivity)                |
| GET    | `/openapi`            | OpenAPI 3 spec (JSON)                                     |
| POST   | `/auth/login`         | Authenticate, returns user + JWT                          |
| POST   | `/auth/register`      | Create a user account                                     |
| POST   | `/video/inspect`      | Upload a video, inspect it with `ffprobe`, cache metadata |
| POST   | `/video/jobs`         | Upload a video and run an `ffmpeg` operation on it        |
| GET/WS | `/video/ws/{user_id}` | WebSocket for live job progress updates                   |

Video routes enforce a max upload body size and (in production) per-IP rate limiting via `tower_governor`.

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
| `BACKEND_HOST`, `BACKEND_HTTP_PORT`                                     | bind address/port                                                       |
| `DATABASE_URL`                                                          | Postgres connection string                                              |
| `BACKEND_DB_MAX_CONNECTIONS`                                            | DB pool size                                                            |
| `BACKEND_JWT_SECRET`                                                    | JWT signing secret                                                      |
| `BACKEND_CORS_ORIGINS`                                                  | comma-separated list of allowed origins                                 |
| `BACKEND_VIDEO_MAX_BODY_SIZE`                                           | max multipart upload size (bytes)                                       |
| `BACKEND_VIDEO_RATE_LIMIT_PERIOD_SEC` / `BACKEND_VIDEO_RATE_LIMIT_SIZE` | rate limiter period/burst (production only)                             |
| `BACKEND_VIDEO_INSPECT_TIMEOUT` / `BACKEND_VIDEO_PROCESS_TIMEOUT`       | timeouts (seconds) for `ffprobe`/`ffmpeg` invocations                   |
| `BACKEND_VIDEO_MOCK_PASSWORD_HASH`                                      | dummy hash used to defend login against user-enumeration timing attacks |
| `IS_PRODUCTION`                                                         | toggles production-only behavior (rate limiting, etc.)                  |

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
builds with `SQLX_OFFLINE=true` in CI/containers.

## Testing

```sh
cargo nextest run
```

Integration tests live in `tests/` with fixtures in `tests/fixtures/`; property-based test regressions are tracked under
`proptest-regressions/`.

## Docker

The production image is built from [`configs/docker/backend.dockerfile`](../../configs/docker/backend.dockerfile): a
multi-stage build that compiles the release binary against `rust:1.97` and ships it on `debian:trixie-slim` with
`ffmpeg` installed. See the root [`docker-compose.dev.yml`](../../docker-compose.dev.yml) / [
`docker-compose.prod.yml`](../../docker-compose.prod.yml) for how it's composed with Postgres and nginx.

## Notes

- `ffprobe` JSON output format reference: https://raw.githubusercontent.com/FFmpeg/FFmpeg/master/doc/ffprobe.xsd
