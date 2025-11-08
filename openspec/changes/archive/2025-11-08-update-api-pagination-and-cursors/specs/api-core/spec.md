## ADDED Requirements
### Requirement: Cursor Pagination Primitives
The API MUST implement cursor-based pagination consistently across all list endpoints.

- Request parameters:
  - `limit` (integer): default 50, max 100, min 1; must be a positive integer (no decimals, no negatives). Server applies bounds and validates type.
  - `cursor` (string, optional): opaque token representing the last seen item position.
- Response metadata:
  - `next_cursor` (string | null): always present; `null` when there are no further pages.
  - The `items` array name MAY vary per endpoint (e.g., `jobs`, `connections`), but the `next_cursor` property name is uniform.

#### Scenario: First page with next_cursor
- GIVEN more than `limit` matches exist
- WHEN requesting a list with `?limit=2`
- THEN the response contains 2 items and a non-empty `next_cursor`

#### Scenario: Last page sets next_cursor to null
- GIVEN a final page of results
- WHEN the client fetches the last page
- THEN the response contains items (possibly zero) and `next_cursor: null`

#### Scenario: Limit bounds enforced
- WHEN `limit` < 1 or > 100 or is not a positive integer (e.g., decimal, negative, non-numeric)
- THEN respond `400` with `code: "VALIDATION_FAILED"`

### Requirement: Stable Ordering
All list endpoints MUST define a deterministic total order and use a unique tiebreaker to avoid duplicates or gaps during pagination.

- Event-like lists (e.g., activity, jobs): `created_at DESC, id DESC` or endpoint-specific timestamp `DESC` with `id DESC` tiebreaker.
- Catalog-like lists (e.g., providers): `name ASC, id ASC` or endpoint-specific field `ASC` with `id ASC` tiebreaker.
- The cursor MUST encode only the ordered keys needed to resume scanning.

#### Scenario: Ties broken by id
- GIVEN multiple items share the same primary sort value
- WHEN fetching a page boundary at the tie
- THEN subsequent pages continue with consistent `id` tiebreaker ordering with no duplicates

### Requirement: Cursor Token Opaqueness
Cursor tokens SHALL be opaque to clients. Servers encode minimal ordered keys as standard Base64-encoded JSON with padding.

- The server MUST validate cursor structure and integrity, and MUST reject malformed or unrecognized tokens with a 400 error.
- The server MAY reject previously-issued cursors that are no longer valid due to data changes, retention policies, or other operational constraints, using the same `VALIDATION_FAILED` semantics.
- Cursors are valid only for the same filter set used to produce them.

#### Scenario: Malformed cursor rejected
- WHEN the `cursor` cannot be decoded or validated
- THEN respond `400` with `code: "VALIDATION_FAILED"`

#### Scenario: Cursor security validation
- WHEN the cursor fails security validation (too long, invalid characters, invalid UTF-8, invalid JSON, out-of-bounds timestamps, nil UUID)
- THEN respond `400` with `code: "VALIDATION_FAILED"`
- Validation MUST enforce reasonable bounds (for example: max 1000 characters, decoded max 500 bytes, timestamp within ±1 year, non-nil UUID)

#### Scenario: Cursor rejected with different filters
- WHEN a cursor produced with one filter set is used with a different filter set
- THEN the server MAY reject the cursor with `400` and `code: "VALIDATION_FAILED"`
- Clients MUST treat cursors as bound to the exact filter set used to produce them
