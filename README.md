<div style="display: flex; flex-direction: column; justify-content: center; align-items: center;" align="center">
    <h1><code>Video processing app</code></h1>
    <h4>Built with <a href="https://rust-lang.org/">🦀</a><a href="https://react.dev">⚛️</a></h4>
</div>


[![main](https://github.com/arthurhovhannisyan31/video_processing_app/actions/workflows/code-validation.yml/badge.svg?branch=main)](https://github.com/arthurhovhannisyan31/video_processing_app/actions/workflows/code-validation.yml)
[![main](https://github.com/arthurhovhannisyan31/video_processing_app/actions/workflows/packages-validation.yml/badge.svg?branch=main)](https://github.com/arthurhovhannisyan31/video_processing_app/actions/workflows/packages-validation.yml)

## Overview

A full-stack app for inspecting and processing video files: upload a video, get its metadata (via `ffprobe`), run an
`ffmpeg` operation on it, and watch the job progress live over a WebSocket. It's a monorepo with a Rust API server and a
Next.js web client.

- **[`modules/backend`](modules/backend/README.md)** — Rust/Axum API: auth, video inspect/process endpoints, WebSocket
  progress updates, PostgreSQL storage. See its [README](modules/backend/README.md) for details.
- **[`modules/frontend`](modules/frontend/README.md)** — Next.js/React web client: upload UI, auth, live job progress.
  See its [README](modules/frontend/README.md) for details.

## Tech stack

|          |                                                                                    |
|----------|------------------------------------------------------------------------------------|
| Backend  | Rust, Axum, SQLx/PostgreSQL, JWT auth, `ffmpeg`/`ffprobe`                          |
| Frontend | Next.js 16, React 19, TypeScript, Tailwind CSS, TanStack Query, Jotai, better-auth |
| Infra    | Docker Compose, nginx, GitHub Actions CI/CD                                        |

## Repository layout

```
.
├── modules/
│   ├── backend/     # Rust API server (see modules/backend/README.md)
│   └── frontend/    # Next.js web client (see modules/frontend/README.md)
├── configs/         # shared scripts, docker files, nginx configs, git hooks
├── .github/         # CI/CD workflows and composite actions
├── docker-compose.dev.yml
├── docker-compose.prod.yml
└── Makefile         # convenience commands spanning both modules
```

## Prerequisites

- Rust (stable, edition 2024) — see [`modules/backend/README.md`](modules/backend/README.md)
- Node.js `^24` and Yarn 4 (Berry) — see [`modules/frontend/README.md`](modules/frontend/README.md)
- PostgreSQL
- `ffmpeg` / `ffprobe` on `PATH`
- Docker + Docker Compose (optional, for containerized dev/prod)

## Getting started

### 1. Manual setup

1.1 Copy the env template and fill in the values:

```sh
cp .env.template .env.container
```

1.2 One-time setup (git hooks, DB query cache, API client generation):

```sh
make prepare
```

Or Run each module in its own terminal:

```sh
make backend    # cd modules/backend && cargo run
make frontend   # cd modules/frontend && yarn dev
```

### 2. Docker Compose:

You can run backend services with docker compose:

```sh
docker compose -f docker-compose.dev.yml --env-file .env.container up --build
```

This starts Postgres, the backend, and an nginx reverse proxy (dev config in [`configs/nginx`](configs/nginx)).

## Common commands (Makefile)

Run from the repo root; each target delegates into the relevant module.

| Command                             | Description                                                                     |
|-------------------------------------|---------------------------------------------------------------------------------|
| `make prepare`                      | install git hooks, prepare SQLx offline cache, generate the frontend API client |
| `make backend` / `make frontend`    | run the backend / frontend dev servers                                          |
| `make check`                        | lint both modules (`cargo clippy`, `yarn check`)                                |
| `make format` / `make format-check` | format both modules / verify formatting in CI                                   |
| `make test`                         | run backend tests (`cargo nextest run`)                                         |
| `make audit`                        | dependency vulnerability audits for both modules                                |
| `make generate-openapi`             | regenerate `openapi.json` from the backend                                      |
| `make prepare-frontend-local`       | regenerate the frontend's typed API client from the local backend schema        |

## CI/CD

GitHub Actions workflows in [`.github/workflows`](.github/workflows):

- **code-validation** — spins up Postgres, runs migrations, generates the OpenAPI client, then lints/formats/tests both
  modules on every push
- **packages-validation** — `cargo audit` and `yarn npm audit`
- **cleanup-caches** — clears GitHub Actions caches for closed branches
- **server-build-and-deploy** — builds/pushes the backend Docker image and deploys it
- **release** — automated versioned releases from `develop` to `main`

## License

Dual-licensed under [MIT](LICENSE_MIT) or [Apache-2.0](LICENSE_APACHE), at your option.