# Frontend

Next.js web client for the [Video processing app](../../README.md). Lets a user sign up/sign in, drag-and-drop a video file, inspect its metadata, run an `ffmpeg` operation on it via the [backend](../backend/README.md), and watch job progress live over a WebSocket.

## Tech stack

- **Framework:** [Next.js](https://nextjs.org) 16 (App Router), React 19, TypeScript
- **Styling:** Tailwind CSS 4, [shadcn](https://ui.shadcn.com/)-based UI components (`src/components/ui`)
- **State:** [Jotai](https://jotai.org/) for client state, [TanStack Query](https://tanstack.com/query) for server state, [TanStack Table](https://tanstack.com/table) for tabular data
- **Forms/validation:** react-hook-form + zod
- **Auth:** [better-auth](https://www.better-auth.com/)
- **API client:** generated from the backend's OpenAPI spec via [`@hey-api/openapi-ts`](https://heyapi.dev/) (`@hey-api/client-axios`), output in `src/generated/client`
- **Drag & drop / sorting:** `@dnd-kit/*`
- **Charts:** [Recharts](https://recharts.org/)
- **Lint/format:** [Biome](https://biomejs.dev/)
- **Package manager:** Yarn 4 (Berry), Node `^24`

## Project layout

```
src/
├── app/
│   ├── (app)/video/        # main authenticated video page
│   ├── (auth)/_signin, _signup/  # auth pages
│   └── api/auth/[...all]/   # better-auth route handler
├── components/
│   ├── modules/auth/        # sign-in / sign-up forms
│   ├── modules/video/        # drop zone, file list/card, controls bar, inspect result, hooks (useWebSocket)
│   ├── ui/                    # shadcn-based primitives
│   └── theme-provider, theme-mode-toggle
├── configs/routes/            # route path constants
├── generated/client/           # generated OpenAPI client — do not hand-edit
├── helpers/api/                 # thin wrappers around generated client calls (inspect/process video, download)
├── hooks/, lib/, store/           # shared hooks, auth/query-client setup, jotai atoms (video/user)
└── typings/                       # shared TS types
```

## Prerequisites

- Node.js `^24`
- Yarn 4 (Berry) — enable via `corepack enable`
- The [backend](../backend/README.md) running (locally or reachable at `NEXT_PUBLIC_API_DOMAIN`) if you need a live OpenAPI schema or working API calls

## Configuration

Copy [`.env.template`](.env.template) to `.env` and fill in the values:

| Variable | Purpose |
|---|---|
| `NEXT_PUBLIC_API_DOMAIN` | base URL of the backend API |
| `NEXT_PUBLIC_MAX_BODY_SIZE` | max upload size, mirrors the backend's limit for client-side validation |
| `NEXT_PUBLIC_PROXY_AUTH_CHECK_ENABLED` | toggles the auth check proxy behavior |
| `NEXT_PUBLIC_IS_PROD` | production flag for client code |
| `BETTER_AUTH_SECRET` / `BETTER_AUTH_URL` | better-auth server config |

## Getting started

```sh
yarn install
yarn generate-openapi-local   # or: yarn generate-openapi (fetches the remote schema)
yarn dev
```

Open [http://localhost:3000](http://localhost:3000) — the root path redirects to `/video`.

From the repo root, the equivalent is:

```sh
make prepare-frontend-local   # generate API client from the local backend's openapi.json
make frontend                 # yarn dev
```

## Generating the API client

The typed API client under `src/generated/client` is generated from the backend's OpenAPI spec and should never be hand-edited.

```sh
yarn generate-openapi         # pulls schema from the deployed API (https://api.videoprocessing.app/api/openapi)
yarn generate-openapi-local   # pulls schema from ../backend/openapi.json (run `make generate-openapi` in the backend first)
```

## Development commands

```sh
yarn dev      # start dev server (Turbopack)
yarn build    # generate API client + next build (configs/scripts/build-fe.sh)
yarn start    # start production server
yarn lint     # biome check
yarn format   # biome check --write .
yarn types    # tsc --noEmit
yarn check    # types + lint
```

## Notes

- `next.config.ts` enables the React Compiler and redirects `/` to `/video`.
- Video processing progress is streamed over a WebSocket (`useWebSocket` hook, `/api/video/ws/{user_id}` on the backend) and reflected in the `videoStore` jotai atom.
