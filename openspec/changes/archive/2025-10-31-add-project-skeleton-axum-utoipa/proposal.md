## Why
We need a minimal API skeleton to begin local development: an Axum server with a root "hello" route and Swagger documentation via utoipa. This enables quick smoke testing and provides a foundation for subsequent connectors and endpoints.

## What Changes
- Bootstrap an Axum HTTP server with Tokio runtime.
- Add `GET /` route returning a JSON payload with service name and version.
- Integrate `utoipa` + `utoipa-swagger-ui` and serve docs at `GET /docs`.
- Expose OpenAPI JSON at `GET /openapi.json`.
- Configuration: env `POBLYSH_API_BIND_ADDR` (default `0.0.0.0:8080`).
- Localhost only; no Docker or cloud dependencies in this change.

Non-goals (deferred to later proposals):
- Health/readiness endpoints
- Operator auth and tenant scoping
- Database and storage layers

## Impact
- Affected specs: `api-core`
- Affected code (expected): `Cargo.toml`, `src/main.rs`, `src/api/routes.rs`
- Tooling: add dependencies (`axum`, `tokio`, `serde`, `utoipa`, `utoipa-swagger-ui`)

