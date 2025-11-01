## 1. Implementation
- [ ] 1.1 Add route `GET /connections` in `src/server.rs` and `src/handlers/connections.rs`.
- [ ] 1.2 Define DTOs: `ConnectionInfo { id, provider, expires_at?, metadata }`, `ConnectionsResponse { connections }` with `utoipa::ToSchema`.
- [ ] 1.3 Enforce auth + tenant header via existing middleware/guards.
- [ ] 1.4 Integrate repository: fetch tenant-scoped connections; support optional `provider` filter; stable sort by `id`.
- [ ] 1.5 Annotate endpoint with `#[utoipa::path(...)]` and add to `ApiDoc`.
- [ ] 1.6 Unit tests: 200 response shape, filter behavior, empty list, and auth/tenant enforcement.

## 2. Validation
- [ ] 2.1 `cargo test` green for new tests.
- [ ] 2.2 Swagger UI shows `/connections` and schemas.

## 3. Notes / Non-goals
- Pagination is deferred (MVP scope). Add later via `update-api-pagination-and-cursors` change.
- Error envelope mapping is handled by `add-error-model-and-problem-json` change.

