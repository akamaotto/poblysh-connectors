## Context
We need a single, enforceable pagination and cursor contract that all list endpoints follow. Current specs mention pagination in places, but behavior like whether `next_cursor` is omitted vs returned as `null` is inconsistent. The data layer uses SeaORM; API uses Axum with Utoipa for OpenAPI.

## Goals / Non-Goals
- Goals: stable pagination across endpoints, explicit `next_cursor` semantics, minimal and opaque cursor tokens, deterministic ordering with tie-breakers.
- Non-Goals: cryptographic integrity of cursors, changing unrelated payloads, introducing heavy frameworks.

## Decisions
- Cursor token format: Base64-encoded JSON object (aligned with existing implementation)
  - Shape: `{ "occurred_at": RFC3339, "id": UUID }` for existing signals; extendable for other endpoints
  - Example (event-like): `{ "occurred_at":"2024-11-01T12:00:00Z", "id":"11111111-1111-1111-1111-111111111111" } }`
  - Encoding: Base64 standard encoding (with padding) to match current `src/cursor.rs` implementation
  - Rationale: maintains backward compatibility, leverages existing security validation, opaque to clients
- Stable ordering rule:
  - Every endpoint MUST specify a total order using a unique tiebreaker (typically the primary key `id`).
  - Event-like streams: `created_at DESC, id DESC` for reverse chronological views.
  - Catalog-like lists: endpoint-specific (e.g., `name ASC, id ASC`).
- Response shape: always include `next_cursor`; use `null` when there is no subsequent page.
- Request params: `?limit` (default 50, max 100) and `?cursor` (opaque string).

## Risks / Trade-offs
- Cursor visibility: Base64 encoding means clients can inspect and potentially tamper with cursors; mitigation is comprehensive server-side validation (size limits, timestamp bounds, UUID validation) already implemented in `src/cursor.rs`.
- Ordering drift: If queries forget the secondary `id` tiebreaker, pagination may duplicate/skip records; add tests and code review checklists.

## Migration Plan
1) Land `api-core` spec requirement and update endpoint docs to reference it.
2) Add a small internal `cursor` module to encode/decode with standard Base64 JSON.
3) Standardize queries to include a unique tiebreaker and apply `limit+1` fetch for `next_cursor` detection.
4) Update response DTOs to always include `next_cursor` (nullable).
5) Add unit and integration tests for first/next page, cursor round-trip, and stability under ties.

## Research Algorithm (lightweight deep search)
Parallel phase (kick-off 5–7 searches at once):
- SeaORM cursor pagination examples and best practices
- Axum query extraction and pagination parameters patterns
- Utoipa generic schema patterns for reusable `Paginated<T>`
- API design best practices: cursor-based pagination, stable ordering, tiebreakers
- Base64 URL-safe token formats and opaque token guidance

Sequential reinforcement phase (2–3 iterations):
1) Skim official docs for the currently used versions (below), bookmark examples and constraints.
2) Cross-check with community posts/issues for edge cases (null ordering, `limit+1` approach, composite cursors).
3) Contrast alternatives (offset vs cursor; plain delimiter vs Base64 JSON) and confirm the chosen approach aligns with our stack and simplicity goals.

## Tech & Docs (versions in repo)
- Axum `0.8.6`: routing, extractors; used for query params and response models
  - Docs: https://docs.rs/axum/0.8/axum/
- Utoipa `5.3.1`: OpenAPI schema derivation
  - Docs: https://docs.rs/utoipa/5.3.1/utoipa/
- SeaORM `1.1.17`: DB querying, ordering, and conditional filters
  - Docs: https://www.sea-ql.org/SeaORM/docs/
- Tokio `1.48.0`: async runtime
  - Docs: https://docs.rs/tokio/1.48.0/tokio/
- Serde `1.0.217`, Serde JSON `1.0.138`: cursor payload serialization
  - Docs: https://docs.rs/serde/1.0/serde/, https://docs.rs/serde_json/1.0/serde_json/
- Chrono `0.4.38`: RFC3339 timestamps for sort keys
  - Docs: https://docs.rs/chrono/0.4.38/chrono/
- UUID `1.11.0`: tiebreak keys and cursor content
  - Docs: https://docs.rs/uuid/1.11.0/uuid/
- Base64 `0.22` (to add): Standard encoding for opaque tokens
  - Docs: https://docs.rs/base64/0.22.1/base64/

## Open Questions
- For endpoints with optional filters, do we embed filter snapshot in the cursor to guard against drifting result sets? (Prefer not in v1; document that cursors are only valid with the same filter set.)

