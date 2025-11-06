## Why
- Our API lacks a unified, explicit definition for cursor-based pagination. Some endpoints describe `cursor`/`next_cursor` informally and inconsistently (e.g., whether `next_cursor` is omitted vs returned as `null`).
- Without a stable ordering contract and standardized cursor token, clients risk duplicates, gaps, or brittle integrations when paginating.

## What Changes
- Define a core, cross-cutting pagination contract in `api-core`:
  - Request query: `limit` (default 50, max 100) and `cursor` (opaque string).
  - Response shape: `next_cursor` always present; set to `null` when there are no additional pages.
  - Stable ordering: every list endpoint MUST specify a deterministic order with a unique tiebreaker (e.g., `created_at DESC, id DESC`).
  - Cursor token: opaque, Base64URL-encoded JSON of last item’s sort keys (minimum required keys only) with a version tag.
- Provide guidance for event-like vs catalog-like resources:
  - Event-like: `created_at DESC, id DESC` (append-only streams); cursor holds `{ "created_at": RFC3339, "id": UUID }`.
  - Catalog-like: `name ASC, id ASC` or endpoint-specific fields; cursor holds the same ordered keys as used by the endpoint.
- Document OpenAPI guidance: shared schema for paginated responses and `cursor` parameter docs.

## Impact
- Specs: Add a new requirement to `api-core` for Pagination & Cursors; clarify stable ordering and `next_cursor` behavior.
- Affected specs: `api-core` (new requirement); all list endpoints must reference this contract (e.g., providers list, connections list, jobs list).
- Affected code: Axum handlers for list endpoints, repository list queries (e.g., `src/repositories/connection.rs:214`), shared cursor codec module, OpenAPI schema definitions.
- Docs: Update endpoint docs to refer to the shared contract; remove contradictions around omitting vs null `next_cursor`.

## Non-goals
- Changing payload structures beyond pagination metadata.
- Adding encryption or signatures to cursors (tokens are opaque but not tamper-proof in v1).
