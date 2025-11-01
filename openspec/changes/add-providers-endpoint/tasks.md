## 1. Implementation
- [ ] 1.1 Add route `GET /providers` in `src/server.rs` and `src/handlers/providers.rs`.
- [ ] 1.2 Define DTOs: `ProviderInfo { name, auth_type, scopes, webhooks }`, `ProvidersResponse { providers }` with `utoipa::ToSchema`.
- [ ] 1.3 Implement handler returning registry metadata (static list for MVP; registry wires in separate change).
- [ ] 1.4 Annotate endpoint with `#[utoipa::path(...)]` and add to `ApiDoc`.
- [ ] 1.5 Unit test handler returns 200 with expected shape and stable sort.

## 2. Validation
- [ ] 2.1 `cargo test` green (new tests only).
- [ ] 2.2 Swagger UI shows `/providers` and schemas.

## 3. Notes / Non-goals
- Public endpoint; does not require operator auth or tenant header.
- Does not implement connector registry (tracked by `add-connector-trait-and-registry`).

