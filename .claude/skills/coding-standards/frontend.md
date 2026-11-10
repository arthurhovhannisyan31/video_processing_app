# Frontend standards (`modules/frontend`, Next.js 16)

Read `modules/frontend/AGENTS.md` first: this Next.js version has breaking changes, so check
`node_modules/next/dist/docs/` before using a Next API.

## Structure

- App Router groups: `src/app/(app)/` (authenticated, `/video`) and `src/app/(auth)/` (signin, signup,
  forgot/reset password). Pages stay thin and render feature components.
- Feature components: `src/components/modules/<feature>/<component>/index.tsx` with a sibling `constants.ts` for
  zod schemas and their inferred types (e.g. `components/modules/auth/signin-form/constants.ts`).
- `src/components/ui/` holds shadcn primitives — add new ones with the shadcn CLI, compose them instead of
  restyling.
- Shared hooks in `src/hooks/`, feature hooks in `components/modules/<feature>/hooks/`, constants/types/routes in
  `src/configs/`, helpers in `src/helpers/`.

## Data and state

- `src/generated/client` is generated from the backend OpenAPI spec and gitignored — never edit it. Regenerate
  with `make prepare-frontend-local` (local spec) or `make prepare-frontend` (remote).
- Call the generated client through thin wrappers in `src/helpers/api/`. Report errors with `getErrorMessage`
  (`helpers/utils`) and a `sonner` toast.
- Server state: TanStack Query. Client state: Jotai atoms in `src/store/<domain>/index.ts`.
- HTTP goes through `src/lib/api-client.ts` (axios, `withCredentials`, redirects to sign-in on 401).
- Job progress arrives over WebSocket (`components/modules/video/hooks/useWebSocket.ts`,
  `hooks/use-buffered-websocket.ts`) — reuse them.
- Auth uses better-auth (`src/lib/auth.ts`, `src/lib/auth-client.ts`); route protection is `src/proxy.ts`.

## Forms

- react-hook-form + `zodResolver` with the schema from the component's `constants.ts`.
- Use the `components/ui/field` primitives for labels and errors.

## Style

- Imports are absolute from `src/` (`components/...`, `configs/...`, `lib/...`).
- Biome is the formatter/linter; import order: types, react, packages, aliases/paths, styles.
- `useExhaustiveDependencies` and `useHookAtTopLevel` are errors — don't silence them.
- React Compiler is on: don't add `useMemo` / `useCallback` / `memo` by default.
- `"use client"` only on components that need client features.
- `NEXT_PUBLIC_MAX_BODY_SIZE` must match backend `BACKEND_VIDEO_MAX_BODY_SIZE`.

## Verify

- `yarn check` (tsc + biome) and `yarn format`. If `yarn` is not found, use the `yarn-not-found` skill.
- There are no frontend tests; check UI changes by running the app (`make frontend`).
