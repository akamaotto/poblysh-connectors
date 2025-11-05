## 1. Implementation
- [x] 1.1 Add route `GET /signals` in API router; require `Authorization` and `X-Tenant-Id`.
- [x] 1.2 Define query struct: `provider?`, `connection_id?`, `kind?`, `occurred_after?`, `occurred_before?`, `limit?` (default 50, max 100), `cursor?`, `include_payload?` (default false).
- [x] 1.3 Implement repository method to list tenant-scoped signals ordered by `occurred_at DESC, id DESC` with filters and cursor continuation.
- [x] 1.4 Implement opaque cursor encoding/decoding (e.g., base64 of `{ occurred_at, id }`) and reject malformed/foreign cursors with `400 VALIDATION_FAILED`.
- [x] 1.5 Response `{ signals: [...], next_cursor: string|null }`; ensure `next_cursor` is always present (non-empty when more data, `null` on last page) and map validation errors via the unified error envelope.
- [x] 1.6 OpenAPI: schemas for `Signal`, `SignalsResponse`, and document query params.
- [x] 1.7 Tests: auth/tenant enforcement, all filters (provider, connection+kind, occurred_after/before combos), pagination continuation with `next_cursor` null on last page, `include_payload`, and validation errors (limit bounds, malformed cursor, non-UUID `connection_id`, non-RFC3339 timestamps).

## 2. Notes / Non-goals
- No mutation or deletion endpoints in this change.
- Full-text search and sorting customization are out of scope.
