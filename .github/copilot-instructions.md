# Copilot Instructions - Architecture

Project intent: vertical slices by bounded context. Inside each bounded context, follow Clean Architecture. `contracts` is the shared kernel for cross-context contracts. `api_types` is only for API <-> UI DTOs (do not use it inside domain usecases).

## Workspace layout
- `contracts/`: shared kernel (cross-context contracts). Keep it minimal and stable.
- `auth/`: bounded context (domain + usecases + ports + infra + composition).
- `api/`: delivery layer (Actix web) that wires bounded contexts and exposes HTTP.
- `console/`: delivery layer (CLI demo).
- `api_types/`: UI/API DTOs only.

## Dependency rules
- Bounded contexts must not depend on each other directly.
- `contracts` can be used by any crate, but must not depend on infra, frameworks, or delivery concerns.
- `api_types` is for API/UI only. Domain/usecases should not depend on it.
- `api` can depend on bounded contexts and `contracts`.
- `console` can depend on bounded contexts and `contracts`.

## Clean Architecture inside a bounded context (example: `auth`)
Layers and allowed dependencies (inward only):
1) `entities/`: pure domain types and constants. No framework or IO types.
2) `usecases/`: application logic; depends on `entities/` and `ports/` only.
3) `ports/`: traits that abstract infra needs (repositories, token provider, webauthn, etc.).
4) `infra/`: implementations of `ports/` using concrete libs.
5) `component.rs`: composition root for the context (wires ports to usecases).

Rules:
- Usecases must not import concrete infra types.
- Entities must not depend on external libraries or delivery-layer types.
- `infra` is the only place where external libs (DB, webauthn, jwt, bcrypt, etc.) appear.

## Contracts (shared kernel)
- DTOs and error types should be portable and stable.
- Avoid exposing framework types (e.g., Actix, WebAuthn types) or hashing/token libs.
- Prefer plain structs/enums with serde-ready fields when needed.

## Adding a new bounded context
- Create a new crate at the workspace root: `context_name/`.
- Add `entities/`, `usecases/`, `ports/`, `infra/`, and `component.rs`.
- Expose only the component or public usecase APIs from `lib.rs`.
- Define shared DTOs in `contracts` if multiple contexts or delivery layers must share them.

## API integration
- API handlers translate `api_types` <-> `contracts` DTOs.
- API never accesses infra directly; it only calls the bounded context component.
- Keep HTTP concerns (status codes, headers, middleware) in `api/` only.
