# Frontend auth integration suggestions (not yet applied)

## Bugs

### 1. Auth client and auth relay currently need different base URLs, but the code only models one

The current frontend auth flow has two distinct hops:

- the browser-side Better Auth client must call the Next.js auth route
- the Next.js auth route must call the Rust backend

Those are not the same target:

- browser -> `http://localhost:3000/api/auth`
- server route -> `http://127.0.0.1:8080/api`

Right now the code treats them as if they were driven by one shared `API_BASE_URL`, which creates a structural conflict:

- if `API_BASE_URL` points to the Rust backend, the browser-side Better Auth client tries to call Better Auth endpoints on the Rust server, which do not exist there
- if `API_BASE_URL` points to the Next.js app, the Next.js relay route loses the correct backend target

This is why an env-only fix is not enough with the current layout. The problem is not just a wrong value; it is that one variable is being asked to represent two different systems.

**Fix:** model the two roles explicitly:

```ts
NEXT_PUBLIC_APP_BASE_URL=http://localhost:3000
API_BASE_URL=http://127.0.0.1:8080/api
```

Then use:

- `NEXT_PUBLIC_APP_BASE_URL` for the browser Better Auth client
- `API_BASE_URL` for the server-side relay route

---

### 2. Browser-side Better Auth client cannot point directly to the Rust backend

`authClient.signIn.email(...)` and `authClient.signUp.email(...)` speak Better Auth's own HTTP contract. The Rust backend does not expose that protocol; it exposes custom endpoints:

- `POST /api/auth/login`
- `POST /api/auth/register`

So if the browser auth client points at the Rust backend directly, sign-in fails even when the backend credentials are valid, because the client is talking to the wrong API shape.

**Fix:** always route browser auth calls through the local Next.js Better Auth endpoint:

```ts
http://localhost:3000/api/auth
```

That route can then translate Better Auth requests into the Rust backend's login/register endpoints.

---

### 3. Relative Better Auth `baseURL` is rejected at runtime

Using:

```ts
baseURL: "/api/auth"
```

looks attractive because it avoids hardcoding the origin, but Better Auth expects a valid absolute base URL here and throws at runtime.

This means the "minimal local fix" still needs an explicit app origin in configuration.

**Fix:** build the auth client base URL from a public absolute origin:

```ts
baseURL: `${NEXT_PUBLIC_APP_BASE_URL}/api/auth`
```

---

## Structural feedback

### 4. The current setup mixes transport concerns across layers

The frontend currently combines:

- Better Auth session/cookie handling
- a custom Next.js relay route
- a Rust backend with its own auth contract

This can work, but only if responsibilities are sharply separated. Right now the main failure mode comes from transport ambiguity:

- which code talks to Better Auth endpoints?
- which code talks to Rust auth endpoints?
- which env var identifies which hop?

When those boundaries are implicit, small config changes break login in non-obvious ways.

**Fix:** keep the layering explicit:

- browser client -> Better Auth route in Next.js
- Next.js route -> Rust auth API
- Rust backend -> user validation + JWT issuance

That keeps Better Auth isolated to the frontend app boundary instead of leaking its transport assumptions into backend configuration.

---

## Easy wins

### 5. Name frontend and backend auth URLs by role, not by generic "API"

`API_BASE_URL` is too generic for a flow that already has two different HTTP targets. That naming invites the exact confusion that caused this issue.

**Fix:** prefer role-based names such as:

```ts
NEXT_PUBLIC_APP_BASE_URL
BACKEND_API_BASE_URL
```

or equivalent.

The main point is not the exact naming; it is making it impossible to confuse the browser auth target with the backend relay target.

---

### 6. Document the auth request path once in the repo

This auth flow is slightly non-obvious even when the code is correct. A short note in the repo would save time for the next person touching auth.

**Suggested note:**

1. Browser submits sign-in to Next.js Better Auth route.
2. Next.js auth route forwards credentials to Rust backend.
3. Rust backend returns `{ user, token }`.
4. Better Auth converts that response into the frontend session/cookie state.

That makes future auth debugging much faster and reduces the chance of someone "simplifying" the config back into the broken single-base-URL model.
