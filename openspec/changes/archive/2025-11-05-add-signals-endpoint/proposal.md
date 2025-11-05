## Why
- Operators need to inspect normalized Signals for debugging, verification, and exploratory checks per tenant.
- Queries typically need filters by provider, connection, kind, and occurred_at time windows.
- Cursor pagination is required for consistency and performance; offset pagination is not acceptable at scale.

## What Changes
- Add HTTP endpoint: `GET /signals` (tenant-scoped) to list Signals with filters and cursor pagination.
- Filters: `provider` (slug), `connection_id` (UUID), `kind` (string), `occurred_after`/`occurred_before` (RFC3339 timestamps).
- Pagination: `limit` (default 50, max 100) and opaque `cursor` with response `next_cursor` (always present: opaque string when more data, `null` otherwise); reject malformed/foreign cursors with `400 VALIDATION_FAILED`. Ordering by `occurred_at DESC, id DESC` for stability.
- Response fields: `id`, `provider_slug`, `connection_id`, `kind`, `occurred_at`, `received_at`, optional `payload` when `include_payload=true`, plus `next_cursor: string|null`.

## Impact
- Specs: New `api-signals` capability specifying the endpoint, filters, pagination, and response shape.
- Code: Route + handler, query parsing/validation, repository method for tenant-scoped listing with cursor support, OpenAPI docs, and tests.
- No database changes; leverages existing `signals` table.
