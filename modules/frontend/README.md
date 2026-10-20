# Frontend

Next.js web client for the [Video processing app](../../README.md). Lets a user sign up/sign in (with forgot/reset password), drag-and-drop a video file, inspect its metadata, run an `ffmpeg` operation on it via the [backend](../backend/README.md), and watch job progress live over a WebSocket.

## Tech stack

- **Framework:** [Next.js](https://nextjs.org) 16 (App Router), React 19, TypeScript
- **Styling:** Tailwind CSS 4, [shadcn](https://ui.shadcn.com/)-based UI components (`src/components/ui`)
- **State:** [Jotai](https://jotai.org/) for client state, [TanStack Query](https://tanstack.com/query) for server state, [TanStack Table](https://tanstack.com/table) for tabular data
- **Forms/validation:** react-hook-form + zod
- **Auth:** [better-auth](https://www.better-auth.com/)
- **API client:** generated from the backend's OpenAPI spec via [`@hey-api/openapi-ts`](https://heyapi.dev/) (`@hey-api/client-axios`), output in `src/generated/client`
- **Drag & drop / sorting:** `@dnd-kit/*`
- **Charts:** [Recharts](https://recharts.org/)
- **WebSocket:** [`@github/stable-socket`](https://github.com/github/stable-socket) with reconnect policy
- **Error tracking:** [Sentry](https://sentry.io) (`@sentry/nextjs`)
- **Lint/format:** [Biome](https://biomejs.dev/)
- **Package manager:** Yarn 4 (Berry), Node `^24`

## Project layout

```
src/
├── app/
│   ├── (app)/video/          # main authenticated video page
│   ├── (auth)/               # signin, signup, forgot-password, reset-password pages
│   └── api/auth/[...all]/    # better-auth route handler
├── components/
│   ├── modules/auth/         # sign-in / sign-up / forgot-password / reset-password forms
│   ├── modules/video/        # drop zone, file list/card, controls bar, inspect result, hooks (useWebSocket)
│   ├── ui/                    # shadcn-based primitives
│   └── theme-provider, theme-mode-toggle
├── configs/                    # env-derived constants (API URLs, flags), route path constants
├── proxy.ts                    # Next middleware: redirects unauthenticated users to /signin
├── generated/client/           # generated OpenAPI client — do not hand-edit
├── helpers/api/                 # thin wrappers around generated client calls (inspect/process video, download)
├── hooks/, lib/, store/           # shared hooks (incl. useBufferedWebSocket), auth/axios/query-client setup, jotai atoms
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
| `NEXT_PUBLIC_API_DOMAIN` | backend host + path **without protocol** (e.g. `localhost:8080/api`); `http`/`ws` or `https`/`wss` is picked from `NEXT_PUBLIC_IS_PROD` |
| `NEXT_PUBLIC_MAX_BODY_SIZE` | max upload size, mirrors the backend's `BACKEND_VIDEO_MAX_BODY_SIZE` for client-side validation |
| `NEXT_PUBLIC_PROXY_AUTH_CHECK_ENABLED` | `true` enables the sign-in redirect in `src/proxy.ts` |
| `NEXT_PUBLIC_IS_PROD` | production flag for client code (protocols, secure cookie name) |
| `BETTER_AUTH_SECRET` / `BETTER_AUTH_URL` | better-auth server config |
| `NEXT_PUBLIC_SENTRY_DSN` | Sentry DSN |
| `NEXT_PUBLIC_LOCAL_ENV` | `true` disables Sentry for local production builds |
| `SENTRY_AUTH_TOKEN`, `SENTRY_ORG_NAME`, `SENTRY_PROJECT_NAME` | Sentry source map upload at build time |

## Getting started

```sh
yarn install
yarn generate-openapi-local   # or: yarn generate-openapi (fetches the remote schema)
yarn dev
```

Open [http://localhost:3000](http://localhost:3000) — the root path redirects to `/video`.

## Auth flow

1. The sign-in/sign-up forms call the backend (`/auth/login`, `/auth/register`) through the generated client.
2. The returned user + JWT are posted to a custom better-auth endpoint (`/custom-auth` in `src/lib/auth.ts`), which
   creates a better-auth session and sets its cookie (JWT cookie cache, 7 days).
3. `src/proxy.ts` checks that cookie on every non-public page and redirects to `/signin?callback-url=...` when it is
   missing or expired. Public pages: `signin`, `signup`, `forgot-password`, `reset-password`.
4. The axios client (`src/lib/api-client.ts`) sends requests `withCredentials` and redirects to sign-in on `401`.

Forgot/reset password forms call `/auth/password/forgot` and `/auth/password/reset`; the reset link in the email
points to `/reset-password` with the token.

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
- Video processing progress is streamed over a WebSocket (`useWebSocket` hook, `/api/video/ws` on the backend) and reflected in the `videoStore` jotai atom. The backend ties the socket to the session cookie, so only one tab per session can receive progress.
- `yarn build` regenerates the API client from the **remote** schema before building.
