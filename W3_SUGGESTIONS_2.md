# Backend Architecture Suggestions (Rust-Focused)

The current frontend is a good foundation.  
The main part that should evolve is the backend: keep the current level of rigor, but reorganize it in a more idiomatic Rust shape.

## Why change the backend architecture

The current backend follows a very layered structure:

```text
application/
data/
domain/
infrastructure/
presentation/
```

This is not the idiomatic shape in Rust.

DDD-style global layering is not especially common in Rust projects, especially when it creates a strict split between `application`, `domain`, `data`, `infrastructure`, and `presentation`.

The reason is mostly pragmatic:

- Rust already has meaningful complexity
  ownership, traits, async, typed errors, SQLx macros
- adding multiple architectural layers on top of that increases indirection quickly
- it slows navigation because one feature is spread across too many folders
- it often introduces abstractions before there is a real need for them

In many Rust backends, the more common direction is:

- concrete types over excessive abstraction
- feature-based organization over global DDD layers
- shared technical plumbing only where it is truly mutual
- explicit dependencies with less architectural ceremony

So the goal is not to "simplify" the backend.  
The goal is to make it more direct, more readable, and more idiomatic for a real Rust project.

## Proposed direction

Keep:

- `axum`
- `sqlx::query_as!`
- `tracing`
- `utoipa`
- auth/JWT
- migrations
- service / repository separation if we want to preserve that level of rigor

Change mainly:

- the global layered split
- the way code is navigated
- where responsibilities live

## Proposed backend architecture

```text
src/
├── main.rs
├── lib.rs
├── app_state.rs
├── router.rs
├── shared/
│   ├── config.rs
│   ├── cors.rs
│   ├── database.rs
│   ├── error.rs
│   ├── jwt.rs
│   ├── logging.rs
│   └── openapi.rs
└── features/
    ├── auth/
    │   ├── mod.rs
    │   ├── dto.rs
    │   ├── handlers.rs
    │   ├── middleware.rs
    │   ├── model.rs
    │   ├── protected.rs
    │   ├── repository.rs
    │   └── service.rs
    └── system/
        ├── mod.rs
        └── handlers.rs
```

## Why this shape is more idiomatic in Rust

This is closer to how many Rust backends are actually organized in practice.

The main point is not that DDD is impossible in Rust.  
The point is that strict global DDD layering is not especially common in Rust codebases, while feature-first organization is much more natural to maintain.

### 1. You read the product before you read the layers

With `features/auth` and `features/system`, you can immediately find everything related to one feature.

That is simpler than splitting one feature across:

- controller code in `presentation`
- logic in `application`
- entities in `domain`
- database access in `data`
- shared helpers in `infrastructure`

### 2. Dependencies stay explicit

Rust works well when dependencies are concrete and easy to trace.

Here:

- `handlers` call `service`
- `service` uses `repository`
- `shared` contains common plumbing

There is still structure, but less architectural ceremony.

### 3. `shared/` avoids duplication without polluting features

`shared/` should not contain business logic.  
It should only contain truly mutual pieces such as:

- config
- database
- JWT
- errors
- logging
- OpenAPI
- CORS

That keeps features clean, while shared cross-cutting pieces are not hidden inside an arbitrary feature.

### 4. Growth into video features becomes more natural

Even with only the current modules, this structure already gives a clean path for future growth.

For now, the important point is simpler:

- `auth` contains auth-specific logic
- `system` contains infrastructure-facing endpoints such as `health` and `openapi`
- `shared` contains only common backend plumbing

## Why this is not just a style preference

The gain is concrete:

- less navigation friction
- less indirection
- easier onboarding
- simpler refactors
- closer to how many modern Rust backends are actually organized

So yes, there is a real pragmatic reason behind it, not just a naming preference:

- better readability
- better time-to-build
- lower maintenance cost

## What should not be lost

This refactor should not reduce the current level of rigor.

These parts should be preserved:

- `sqlx::query_as!`
- clear error handling
- `AppState`
- auth middleware
- JWT service
- migrations
- `utoipa`
- structured logging

The requested change is mostly a change in **shape**, not a drop in quality.

## Recommended position

### Frontend

The frontend is already a good base.  
No major architectural rewrite is needed there for now.

### Backend

The backend should be reorganized toward:

- `shared/` for mutual concerns
- `features/` for business logic
- a central `router.rs`
- a clear `app_state.rs`

In short:

- keep the rigor of the template
- keep the solid technical choices
- make the architecture more Rust-first
- prepare the codebase for future video-processing features
