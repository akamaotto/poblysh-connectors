## Why
Clients need a consistent, documented error response across the API. Today errors vary by source (Axum rejections, DB, internal) and lack a uniform envelope. A unified problem+json-style model with mappers improves debuggability, OpenAPI docs, and client ergonomics while enabling correlation via trace IDs and rate-limit hints.

## What Changes
- Introduce a unified error envelope returned as `application/problem+json` with fields: `code`, `message`, optional `details`, optional `retry_after`, and optional `trace_id`.
- Add error type (`ApiError`) with `IntoResponse` and `utoipa::ToSchema` implementations.
- Implement mappers for common sources: validation, auth, not found, conflict/unique violation, rate limit, provider upstream errors, and internal errors.
- Propagate correlation/trace ID into error responses and structured logs.
- Update handlers to return `Result<T, ApiError>` and register OpenAPI error responses.

## Impact
- Affected specs: `api-core`
- Affected code: `src/error.rs` (new), `src/server.rs`, handlers in `src/handlers/` to use `ApiError` and declare responses
- Dependencies: add `thiserror` (derive), reuse `utoipa`

