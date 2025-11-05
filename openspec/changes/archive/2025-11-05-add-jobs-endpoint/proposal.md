## Why
- Operators need visibility into background synchronization: what jobs are queued, running, or failed, and for which connections.
- Debugging and operations require filtering by status, provider, connection, and time windows.
- We need a standard, tenant-scoped listing with cursor pagination for consistent API ergonomics.

## What Changes
- Add HTTP endpoint: `GET /jobs` (tenant-scoped) to list sync jobs with filters and cursor pagination.
- Filters: `status` (one of `queued|running|succeeded|failed`), `provider` (slug), `connection_id` (UUID), `job_type` (`full|incremental|webhook`). Optional time filters via `started_after`, `finished_after`, which may be combined to bound both start and finish timestamps.
- Validation: Reject unknown enum values, malformed UUIDs, invalid RFC3339 timestamps, or malformed cursors with the unified error envelope.
- Pagination: `limit` (default 50, max 100) and opaque `cursor` with response `next_cursor` (always present; string when more pages, `null` otherwise) for continuation. Ordering by `scheduled_at DESC, id DESC`.
- Response includes essential job fields: id, provider_slug, connection_id, job_type, status, priority, attempts, scheduled_at, retry_after, started_at, finished_at.

## Impact
- Specs: New `api-jobs` capability defining `GET /jobs` behavior, filters, and pagination.
- Code: Route, query parsing, repository listing with tenant scoping and cursor support, OpenAPI docs, and tests.
- No changes to database schema or external APIs.
