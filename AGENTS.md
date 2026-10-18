# AGENTS.md

Monorepo for a video inspect/process app: Rust/Axum API in `modules/backend`, Next.js 16 client in `modules/frontend`.
Module READMEs (`modules/*/README.md`) cover env vars, API routes and tech stack in detail.

## Commands

Run from the repo root (each `make` target `cd`s into the module):

```sh
make prepare            # install git hooks, cargo sqlx prepare, generate FE API client (remote schema)
make backend            # cargo run (runs pending migrations on startup)
make frontend           # yarn dev
make check              # cargo clippy --all-features --all-targets + yarn check (tsc + biome)
make format             # cargo fmt + biome --write
make format-check       # cargo fmt -- --check (backend only)
make test               # cargo nextest run (backend only; frontend has no tests)
make generate-openapi   # cargo run --bin openapi-generator -> modules/backend/openapi.json
make prepare-frontend-local  # regenerate FE client from modules/backend/openapi.json
```

Single backend test (from `modules/backend`):

```sh
cargo nextest run --test video_process                 # one integration test file
cargo nextest run -E 'test(name_substring)'            # filter by test name
```

Backend tests need a running Postgres reachable via `DATABASE_URL` plus `ffmpeg`/`ffprobe` on `PATH`; integration tests
use `#[sqlx::test(fixtures("create_user"))]`, which creates a throwaway DB per test.

Git hooks (`configs/git/hooks`): pre-commit runs `make check` + `make format`; pre-push runs openapi generation,
check, format-check, test and audit. CI (`.github/workflows/code-validation.yml`) mirrors pre-push.

## Cross-module contracts

- **OpenAPI is the API contract.** Backend handlers are annotated with `#[utoipa::path]` and registered in
  `src/core/openapi.rs`. The frontend client in `modules/frontend/src/generated/client` is generated from that spec
  (`configs/scripts/generate-openapi-client.sh`) and is gitignored — never hand-edit it. After changing a route/DTO:
  `make generate-openapi && make prepare-frontend-local`.
- **SQLx offline cache.** Queries must use the checked macros (`query_as!`, `query_scalar!`; plain `sqlx::query`/
  `query_as` are banned in `clippy.toml`). Builds in CI/Docker use `SQLX_OFFLINE=true`, so after changing any query run
  `make prepare-backend` and commit `modules/backend/.sqlx/`. Migrations live in `modules/backend/migrations/`.
- `NEXT_PUBLIC_MAX_BODY_SIZE` (frontend) mirrors `BACKEND_VIDEO_MAX_BODY_SIZE` (backend).

## Backend architecture (`modules/backend`)

- `main.rs` → config from env (`AppConfig::from_env`, loads `.env` unless `CONTAINER=true`) → DB pool → session store
  → migrations → `AppState` → `http.rs` serve. Crate is also a lib (`lib.rs`) so `tests/` can build the router.
- `router.rs` holds all route path constants (`routes::*`) and nests every feature router under `/api`, layering
  sessions (`axum_session` in Postgres), tracing, compression (skipped for `video/*`), CORS, Sentry.
- `src/features/{auth,system,video}` each contain routes / dto / service / repository / state. Per-feature state is a
  field of `AppState` (`src/core/app_state.rs`) and is extracted in handlers via `FromRef` (e.g.
  `State<Arc<VideoState>>`). When adding a feature, wire its state into `AppState::new` **and** into
  `tests/common/utils.rs::setup_router_with_email`, which builds `AppState` by hand (with `MockEmailSender`).
- Errors: `DomainError` / `ServerError` convert into `ApplicationError`, which is the only `IntoResponse` mapping to HTTP
  status + `ErrorBody { message }` (`src/core/error.rs`). Handlers return `Result<_, ApplicationError>`.
- Auth: login returns a JWT; `features/auth/middleware.rs` validates `Authorization: Bearer` tokens and rejects tokens
  issued before `password_changed_at`. Password reset emails go via SMTP (`core/email.rs`; falls back to Mailpit at
  `mailpit:1025` when `BACKEND_SMTP_HOST` is empty — set `localhost` when running outside Docker).
- Video flow (`features/video/service.rs`): uploads are written to a `TempDir`, `ffprobe`/`ffmpeg` run as child
  processes with timeouts. `/video/inspect` caches metadata in an in-memory `mini_moka` cache keyed by
  hash(file name + session id); `/video/jobs` reuses that cache for the duration used in progress reporting (runs
  `ffprobe` itself on a miss), streams the processed file back, and pushes progress to the session's WebSocket (`/video/ws`) via an
  `mpsc` channel stored in `connections_map`. The session id is what links inspect, job and WebSocket.
- The Bearer-token middleware is not applied to any route yet; video routes are scoped by session cookie only.
- Rate limiting (`tower_governor`, auth + video routes) is only applied when `IS_PRODUCTION=true`.
- Style: 2-space indent, max width 100, imports grouped std/external/crate (`.rustfmt.toml`). Unwrap/expect/panic are
  allowed only in tests.

## Frontend architecture (`modules/frontend`)

- App Router with route groups `(app)` (authenticated, `/video`) and `(auth)` (signin/signup/forgot/reset password);
  `/` redirects to `/video` (`next.config.ts`, React Compiler enabled).
- Auth: better-auth (`src/lib/auth.ts`) with a custom `/custom-auth` endpoint that stores the backend JWT in a
  better-auth session cookie; `src/proxy.ts` (Next middleware) redirects to sign-in when that cookie is missing/expired
  (toggle `NEXT_PUBLIC_PROXY_AUTH_CHECK_ENABLED`). The axios client (`src/lib/api-client.ts`) uses
  `withCredentials` and redirects to sign-in on 401.
- API calls go through thin wrappers in `src/helpers/api/` around the generated client; server state via TanStack
  Query, client state via Jotai atoms in `src/store/`. Job progress arrives over WebSocket
  (`components/modules/video/hooks/useWebSocket.ts`, `hooks/useBufferedWebSocket.ts`).
- `src/components/ui/` are shadcn primitives. Imports are absolute from `src/` (e.g. `configs/constants`). Biome is the
  linter/formatter.

## Repo notes

- Work on `develop`; releases go `develop` → `main` via the release workflow.
- `notes`, `temp_docs/` and `.env*` are gitignored local files (personal notes, past code reviews/PRD, secrets) — don't
  commit them or copy their contents into tracked files.
