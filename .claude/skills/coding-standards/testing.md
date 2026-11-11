# Testing standards

Only the backend has tests. Frontend verification is `yarn check`.

## Running

- Needs Postgres reachable via `DATABASE_URL` and `ffmpeg` / `ffprobe` on `PATH`.
- `make test` from the repo root, or from `modules/backend`:
  - `cargo nextest run --test video_process` — one integration test file
  - `cargo nextest run -E 'test(name_substring)'` — filter by test name

## Integration tests (`modules/backend/tests/`)

- One file per feature area: `<feature>_<area>.rs` (e.g. `auth_api.rs`, `video_inspect.rs`).
- File layout:
  ```rust
  mod common;
  use common::utils;

  #[cfg(test)]
  mod test_<name> {
    // imports, helpers, tests
  }
  ```
- Each test uses `#[sqlx::test]` (fresh throwaway DB per test) or `#[sqlx::test(fixtures("create_user"))]` to seed
  data from `tests/fixtures/<name>.sql`, takes `pool: PgPool` and returns `Result<(), ServerError>`.
- Build the app with `utils::setup_router(pool)` / `utils::setup_server(pool)`; use
  `utils::setup_router_with_email(pool)` to assert on sent emails via `MockEmailSender`.
- Build paths with `utils::with_base_route(routes::X)` — no hard-coded URLs. Shared values in
  `tests/common/constants.rs`.
- Send `X-Forwarded-For` on routes behind the rate limiter's IP extractor.
- Assert status codes and, for errors, deserialize `ErrorBody` and check `message`.
- Cover the success path and every error status documented in the route's `#[utoipa::path]`.
- Media files live in `tests/fixtures/media/`; reuse the existing edge cases (audio only, truncated, dual audio,
  vertical without audio) before adding new files, and keep new ones small.

## Unit tests

- In the same file under `#[cfg(test)] mod tests` (e.g. `features/video/helpers.rs`,
  `features/auth/middleware.rs`).
- Use `proptest` for validators and parsers (`features/auth/dto.rs`); commit files in `proptest-regressions/`.

## Rules

- `unwrap` / `expect` / `panic` are allowed in tests only.
- New feature state must also be wired into `tests/common/utils.rs::setup_router_with_email`.
- Tests must pass with `cargo clippy --all-targets` clean, since pre-push and CI run both.
