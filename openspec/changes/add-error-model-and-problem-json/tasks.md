## 1. Implementation
- [ ] 1.1 Add `thiserror` dependency for error definitions.
- [ ] 1.2 Create `src/error.rs` with `ApiError` enum/struct implementing `IntoResponse` and `utoipa::ToSchema`.
- [ ] 1.3 Implement `From` conversions/mappers: validation errors, Axum rejections, DB unique violation, provider HTTP error wrapper, generic `anyhow` fallback.
- [ ] 1.4 Ensure all handlers return `Result<T, ApiError>` and map errors accordingly.
- [ ] 1.5 Set `Content-Type: application/problem+json` and include `Retry-After` header when `retry_after` is present.
- [ ] 1.6 Inject/propagate `trace_id` (from `tracing` span or request extension) into error responses.
- [ ] 1.7 Register OpenAPI schema and annotate endpoints with error responses.
- [ ] 1.8 Tests: unit tests for mappers, header/content type, and representative responses (400, 404, 409, 429, 500, 502).

## 2. Validation
- [ ] 2.1 `openspec validate add-error-model-and-problem-json --strict` passes.
- [ ] 2.2 `cargo test -q` passes for error module tests.

## 3. Notes / Non-goals
- No rate‑limiting implementation here; only response shape. Actual limits covered elsewhere.
- Auth/tenant enforcement added in separate change; map to `unauthorized`/`forbidden` when applicable.

