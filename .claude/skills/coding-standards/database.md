# Database standards (Postgres + SQLx)

## Migrations (`modules/backend/migrations/`)

- File name: `YYYYMMDDNNNNNN_<verb>_<object>.sql` (e.g. `20261014000001_create_password_reset_tokens.sql`).
- Forward-only: never edit a migration that has been applied; add a new one.
- Make them idempotent with `IF NOT EXISTS` (tables, columns, indexes).
- Conventions from existing tables:
  - `id UUID PRIMARY KEY DEFAULT gen_random_uuid()`
  - timestamps as `timestamptz`, `created_at ... NOT NULL DEFAULT NOW()`
  - named `CHECK` constraints for text length limits (`chk_<column>_len`)
  - foreign keys with `ON DELETE CASCADE` where child rows belong to the parent
  - indexes on lookup/foreign key columns (`idx_<table>_<column>`)
- Migrations run on startup (`src/core/database.rs`) and in CI with `cargo sqlx migrate run`.

## Queries

- Only checked macros: `sqlx::query_as!`, `sqlx::query_scalar!` (plain `sqlx::query` / `query_as` are banned in
  `clippy.toml`).
- Use type overrides for newtypes, e.g. `users.id as "id: UserId"`.
- Do multi-step writes atomically (single statement or transaction), e.g.
  `consume_reset_token_and_set_password` in `features/auth/repository.rs`.

## Repositories

- Data access goes through a trait (`#[async_trait]`, `Send + Sync`) plus a `Postgres*` implementation holding a
  `PgPool` (pattern: `UserRepository` / `PostgresUserRepository`).
- Return `DomainError`. Map unique violations by constraint name constants (`features/auth/constants.rs`) to domain
  variants such as `UserAlreadyExists`; everything else to `DomainError::Internal` and log it.

## SQLx offline cache

- CI and Docker build with `SQLX_OFFLINE=true`. After any query or schema change run `make prepare-backend`
  (needs a migrated DB at `DATABASE_URL`) and commit `modules/backend/.sqlx/`.
- Update test fixtures in `modules/backend/tests/fixtures/*.sql` when the schema change affects them.
