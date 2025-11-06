## ADDED Requirements
### Requirement: Cursor Pagination Primitives
The API MUST implement cursor-based pagination consistently across all list endpoints.

- Request parameters:
  - `limit` (integer): default 50, max 100, min 1; applies server-side bounds.
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
- WHEN `limit` < 1 or > 100
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
Cursor tokens SHALL be opaque to clients. Servers MAY encode minimal ordered keys as an internal Base64URL JSON format.

- The server MUST accept only cursors it produced and MAY reject malformed or unrecognized tokens (including unsupported versions) with a 400 error.
- Cursors are valid only for the same filter set used to produce them.

#### Scenario: Malformed cursor rejected
- WHEN the `cursor` cannot be decoded or validated
- THEN respond `400` with `code: "VALIDATION_FAILED"`
