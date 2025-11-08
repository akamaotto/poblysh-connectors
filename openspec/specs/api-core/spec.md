# api-core Specification

## Purpose
This specification defines the core REST surface shared by all Poblysh connectors, detailing the foundational contracts such as error envelopes, authentication headers, and pagination primitives that other capabilities build upon. It targets backend service developers, integration partners, and API reviewers who rely on a stable definition of cross-cutting behaviors. The document aims to preserve long-term compatibility, enforce consistent security posture, and minimize breaking changes by clearly stating the mandatory guarantees for every endpoint within the public connectors API.
## Requirements
### Requirement: Unified Error Envelope
The API SHALL return errors using a single JSON envelope with media type `application/problem+json`.

Shape (MVP):

- `code` (string, machine‑readable; e.g., `VALIDATION_FAILED`, `UNAUTHORIZED`, `NOT_FOUND`, `CONFLICT`, `RATE_LIMITED`, `PROVIDER_ERROR`, `INTERNAL_SERVER_ERROR`)
- `message` (string, human‑readable debug message; safe to show to operators)
- `details` (object, optional; key‑value metadata like field errors)
- `retry_after` (integer, optional; seconds until retry is advised)
- `trace_id` (string, optional; correlation ID for logs/traces)

#### Details Structure
- For validation errors, use a flat or nested object that maps field names to error messages (string), preserving dotted keys when helpful for deeply nested fields.
- For provider errors, include an object `{ "provider": "<name>", "status": <http_code>, "upstream_error": "<message>" }` to surface upstream context.
- All values in `details` MUST be JSON-serializable (strings, numbers, booleans, null, objects, arrays).
- Error messages inside `details` SHALL be human-readable but NOT localized; clients are responsible for mapping stable keys to localized UI text.

#### Scenario: Validation error returns 400 with details
- GIVEN a request body fails validation for field `name`
- WHEN the handler rejects the request
- THEN the response is HTTP 400 with `Content-Type: application/problem+json`
- AND the body contains `{ code: "VALIDATION_FAILED", message: "...", details: { "name": "..." }, trace_id: "..." }`

#### Scenario: Not found returns 404
- WHEN a resource is not found
- THEN the response is HTTP 404 with `{ code: "NOT_FOUND", message: "..." }`

#### Scenario: Rate limited returns 429 with Retry-After
- WHEN the request exceeds rate limits
- THEN the response is HTTP 429 with `{ code: "RATE_LIMITED", retry_after: N }`
- AND the `Retry-After` header is present with the same value `N`

#### Scenario: Internal error returns 500 with trace id
- WHEN an unexpected error occurs
- THEN the response is HTTP 500 with `{ code: "INTERNAL_SERVER_ERROR", message: "Internal server error", trace_id: "..." }`

### Requirement: Error Mapping
The system SHALL map common error sources to the unified envelope with appropriate HTTP status codes.

Mappings (MVP):
- Validation errors → 400 `VALIDATION_FAILED` with `details` per field
- Auth failures → 401 `UNAUTHORIZED`; forbidden → 403 `FORBIDDEN`
- Not found → 404 `NOT_FOUND`
- Unique constraint violation → 409 `CONFLICT`
- Rate limit triggers → 429 `RATE_LIMITED` with `retry_after` and header
- Provider upstream HTTP errors → 502 `PROVIDER_ERROR` with provider/status metadata in `details`
- Fallback/unknown → 500 `INTERNAL_SERVER_ERROR` with `trace_id`

#### Scenario: Unique violation maps to 409
- GIVEN a Postgres unique key violation occurs while inserting
- WHEN the error is handled
- THEN the response is HTTP 409 with `{ code: "CONFLICT" }`

#### Scenario: Provider error maps to 502
- GIVEN an upstream provider returns HTTP 503
- WHEN the connector path propagates the error
- THEN the API responds with HTTP 502 `{ code: "PROVIDER_ERROR", details: { provider: "github", status: 503 } }`

### Requirement: OpenAPI Error Schema
The error model SHALL be documented in OpenAPI and referenced by endpoints so clients can rely on the shape.

#### Scenario: Schema present in OpenAPI
- WHEN fetching `/openapi.json`
- THEN a schema for `ApiError` exists
- AND endpoints declare `ApiError` for non‑2xx responses where applicable

### Requirement: Correlation ID Propagation
Error responses SHALL include a `trace_id` correlating with structured logs for the same request.

#### Scenario: Trace ID included
- WHEN an error occurs during a request
- THEN the error body includes `trace_id`
- AND a matching `trace_id` appears in server logs

### Requirement: Health Endpoint
The system SHALL expose a liveness endpoint at `GET /healthz` that responds quickly without calling external dependencies.

Response (MVP):
- HTTP 200 with `Content-Type: application/json`
- Body includes `{ "status": "ok", "service": "<service-id>", "version": "<semver>" }`; additional diagnostic fields (e.g., `timestamp`) MAY be present
- No authentication or tenant header required

#### Scenario: Health returns 200 without auth
- WHEN calling `GET /healthz` without any headers
- THEN the response is HTTP 200 with JSON body including `{ "status": "ok", "service": "...", "version": "..." }`

### Requirement: Readiness Endpoint
The system SHALL expose a readiness endpoint at `GET /readyz` that reflects dependency health.

Checks (MVP):
- Database reachable: ability to acquire a connection and run a trivial query (e.g., `SELECT 1`)
- No pending migrations: migrator reports zero pending migrations for the current schema

Responses:
- Ready: HTTP 200 with `Content-Type: application/json` and JSON body
  ```json
  {
    "status": "ready",
    "checks": {
      "database": "ok",
      "migrations": "ok"
    }
  }
  ```
- Each entry in the `checks` object SHALL be either `"ok"` or `"error"` and use the same key names as the defined readiness checks.
- Not ready: HTTP 503 with `Content-Type: application/problem+json` using the unified `ApiError` envelope. The response SHALL set `code` to `SERVICE_UNAVAILABLE`, surface the failing checks inside `details.checks`, and include a human-readable `message`. Example:
  ```json
  {
    "code": "SERVICE_UNAVAILABLE",
    "message": "Database not reachable",
    "details": {
      "checks": {
        "database": "error",
        "migrations": "ok"
      }
    }
  }
  ```
- No authentication or tenant header required

#### Scenario: Ready when DB reachable and no pending migrations
- GIVEN the database is reachable and the migrator reports zero pending migrations
- WHEN calling `GET /readyz`
- THEN the response is HTTP 200 with JSON
  ```json
  {
    "status": "ready",
    "checks": {
      "database": "ok",
      "migrations": "ok"
    }
  }
  ```

#### Scenario: Not ready when DB not reachable
- GIVEN the database is not reachable
- WHEN calling `GET /readyz`
- THEN the response is HTTP 503 with JSON
  ```json
  {
    "code": "SERVICE_UNAVAILABLE",
    "message": "Database not reachable",
    "details": {
      "checks": {
        "database": "error",
        "migrations": "ok"
      }
    }
  }
  ```

#### Scenario: Not ready when pending migrations
- GIVEN there are pending migrations
- WHEN calling `GET /readyz`
- THEN the response is HTTP 503 with JSON
  ```json
  {
    "code": "SERVICE_UNAVAILABLE",
    "message": "Migrations pending",
    "details": {
      "checks": {
        "database": "ok",
        "migrations": "error"
      }
    }
  }
  ```

### Requirement: OpenAPI Documentation for Health and Readiness
The OpenAPI document SHALL describe `GET /healthz` and `GET /readyz` endpoints and mark them as public (no auth required).

#### Scenario: OpenAPI has health and readiness paths
- WHEN fetching `/openapi.json`
- THEN the document contains entries for `/healthz` and `/readyz` with `GET` operations
- AND the operations do not list bearer auth requirements

### Requirement: Cursor-Based Pagination
All list endpoints SHALL use a consistent cursor-based pagination contract for stable, forward-only navigation through result sets.

Request Parameters:
- `limit` (integer, optional): Maximum number of items to return; default 50, maximum 100
- `cursor` (string, optional): Opaque token representing the position to resume from; not provided for the first page

Response Shape:
- `data` (array): The list of items for the current page
- `next_cursor` (string, nullable): Opaque token for the next page; `null` when no additional pages exist
- `has_more` (boolean, optional): Convenience field indicating if more pages exist; true when `next_cursor` is not null

Cursor Implementation:
- Tokens are opaque Base64-encoded JSON objects containing the sort keys of the last item
- Content includes the primary ordering field(s) plus a unique tiebreaker (typically the primary key `id`)
- Example for event-like data: `eyJjcmVhdGVkX2F0IjoiMjAyNC0xMS0wMVQxMjowMDowMFoiLCJpZCI6IjExMTExMTExLTExMTEtMTExMS0xMTExLTExMTExMTExMTExMSJ9`
- Tokens are validated server-side; clients MUST treat them as opaque

Stable Ordering Requirement:
- Every list endpoint MUST specify a deterministic ordering with a unique tiebreaker
- Event-like streams (append-only): `created_at DESC, id DESC` for reverse chronological views
- Catalog-like lists: endpoint-specific ordering (e.g., `name ASC, id ASC`)
- The tiebreaker MUST be unique to prevent duplicates/skips when values are tied

#### Scenario: First page request
- GIVEN a client requests a list without a cursor
- WHEN the server processes the request
- THEN the response includes up to `limit` items
- AND `next_cursor` is provided if more items exist, or `null` if this is the last page

#### Scenario: Next page request
- GIVEN a client provides a `cursor` from a previous response
- WHEN the server processes the request
- THEN the response starts after the position indicated by the cursor
- AND includes up to `limit` items with deterministic ordering
- AND `next_cursor` reflects the new position or `null` if complete

#### Scenario: Cursor round-trip stability
- GIVEN a client receives a cursor and immediately uses it
- WHEN the underlying data hasn't changed
- THEN the next page MUST start exactly where the previous page ended
- AND no items are duplicated or skipped across page boundaries

#### Scenario: Stable ordering under ties
- GIVEN multiple items have identical primary sort values
- WHEN paginating through the result set
- THEN the secondary tiebreaker MUST ensure consistent ordering
- AND page boundaries MUST be deterministic across requests

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

