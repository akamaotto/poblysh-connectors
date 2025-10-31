## 1. Implementation
- [x] 1.1 Create Rust crate (if absent) and initialize `Cargo.toml`
- [x] 1.2 Add crates: `axum`, `tokio` (rt-multi-thread, macros), `serde`, `serde_json`, `utoipa`, `utoipa-swagger-ui`
- [x] 1.3 Implement server bootstrap reading `POBLYSH_API_BIND_ADDR` (default `0.0.0.0:8080`)
- [x] 1.4 Add `GET /` route returning `{ "service": "poblysh-connectors", "version": "0.1.0" }`
- [x] 1.5 Wire Swagger UI at `/docs` and OpenAPI JSON at `/openapi.json`
- [x] 1.6 Add minimal `README` section (local run instructions) and update plan docs if needed
- [x] 1.7 Add a smoke test that boots the router in-memory and asserts `GET /` returns 200 JSON

## 2. Validation
- [x] 2.1 Run the server locally and verify `/` and `/docs`
- [x] 2.2 Confirm OpenAPI JSON served at `/openapi.json`
- [x] 2.3 Validate change using `openspec validate add-project-skeleton-axum-utoipa --strict`

## 3. Out of Scope
- No Docker or cloud dependencies in this change
- No DB, auth, or health endpoints

