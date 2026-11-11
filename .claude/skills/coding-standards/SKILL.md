---
name: coding-standards
description: Project contribution rules for this repo. Use before writing or changing code in modules/backend (Rust/Axum), modules/frontend (Next.js), SQL migrations/queries, or tests.
---

Load only the file(s) that match the change:

| Change                                                   | Read                            |
|----------------------------------------------------------|---------------------------------|
| Rust handlers, services, DTOs, errors, config, routers   | [backend.md](./backend.md)      |
| Next.js pages, components, API wrappers, state, forms    | [frontend.md](./frontend.md)    |
| Migrations, SQLx queries, repositories, `.sqlx/` cache   | [database.md](./database.md)    |
| Integration/unit tests, fixtures                         | [testing.md](./testing.md)      |
| New/changed route or DTO (OpenAPI contract)              | `backend.md` + `frontend.md`    |
| New/changed query or schema                              | `database.md` + `testing.md`    |

## Rules for every change

- Work on `develop`; releases go `develop` → `main` via the release workflow.
- Before committing: `make check` and `make format` (the pre-commit hook runs both).
- Pre-push (and CI) runs `make generate-openapi`, `make prepare-frontend-local`, `make check`,
  `make format-check`, `make test`, `make audit` — run the relevant ones locally first.
- Never commit `notes`, `temp_docs/` or `.env*`, and don't copy their contents into tracked files.
- When behaviour, env vars or routes change, update docs with the `docs-maintenance` skill
  (`README.md`, `docs/*`, `modules/*/README.md`).
