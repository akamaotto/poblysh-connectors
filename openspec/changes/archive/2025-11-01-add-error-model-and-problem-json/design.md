## Context
A single, predictable error model simplifies client integration and operator debugging. Axum, SeaORM, and upstream provider failures should all map into a common envelope with consistent HTTP status codes, problem+json media type, and correlation identifiers.

## Goals / Non-Goals
- Goals: unified error envelope and mappers, OpenAPI documentation, trace ID propagation, headers for retry semantics.
- Non-Goals: implementing rate‑limiting/auth itself; only error mapping and shape.

## Decisions
- Use `application/problem+json` with a compact field set: `code`, `message`, `details?`, `retry_after?`, `trace_id?`.
- Implement `ApiError` with `IntoResponse` to set status, headers, and serialized body.
- Define an upstream provider error wrapper including provider slug, HTTP status, and body snippet for diagnostics.
- Map Postgres unique violations to 409 Conflict; avoid exposing raw SQL error strings to clients.
- Source `trace_id` from the current `tracing` span or a request ID middleware; include in the body and logs.

## Alternatives Considered
- Strict RFC7807 fields (`type`, `title`, `status`, `detail`, `instance`): project uses a simplified shape for clarity; may add RFC fields later if needed.
- Returning plain strings for errors: rejected due to poor structure and client behavior.

## OpenAPI
- Derive `utoipa::ToSchema` for `ApiError` and reference it in endpoint annotations for standard non‑2xx responses.

## Testing
- Table‑driven tests for each mapper ensuring status code, headers, and body fields match expectations.

