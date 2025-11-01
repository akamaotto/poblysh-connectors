## 1. Implementation
- [x] 1.1 Add `thiserror` dependency for error definitions.
- [x] 1.2 Create `src/error.rs` with `ApiError` enum/struct implementing `IntoResponse` and `utoipa::ToSchema`.
- [x] 1.3 Implement `From` conversions/mappers: validation errors, Axum rejections, DB unique violation, provider HTTP error wrapper, generic `anyhow` fallback.
- [x] 1.4 Ensure all handlers return `Result<T, ApiError>` and map errors accordingly.
- [x] 1.5 Set `Content-Type: application/problem+json` and include `Retry-After` header when `retry_after` is present.
- [x] 1.6 Inject/propagate `trace_id` (from `tracing` span or request extension) into error responses.
- [x] 1.7 Register OpenAPI schema and annotate endpoints with error responses.
- [x] 1.8 Tests: unit tests for mappers, header/content type, and representative responses (400, 404, 409, 429, 500, 502).

## 2. Validation
- [x] 2.1 `openspec validate add-error-model-and-problem-json --strict` passes.
- [x] 2.2 `cargo test -q` passes for error module tests.

## 3. Notes / Non-goals
- No rate‑limiting implementation here; only response shape. Actual limits covered elsewhere.
- Auth/tenant enforcement added in separate change; map to `unauthorized`/`forbidden` when applicable.

