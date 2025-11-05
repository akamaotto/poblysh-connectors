## ADDED Requirements
### Requirement: Signals Listing Endpoint
The system SHALL expose `GET /signals` to list normalized Signals for the specified tenant with filters and cursor pagination.

#### Scenario: Returns tenant-scoped signals
- **WHEN** a client calls `GET /signals` with a valid `Authorization` token and `X-Tenant-Id`
- **THEN** respond `200 OK` with JSON body `{ signals: [ { id: uuid, provider_slug: string, connection_id: uuid, kind: string, occurred_at: RFC3339, received_at: RFC3339, payload?: object } ], next_cursor: string|null }`

#### Scenario: Missing tenant header returns 400
- **WHEN** `X-Tenant-Id` is not provided
- **THEN** respond `400` using the unified error envelope

#### Scenario: Unauthorized without bearer token
- **WHEN** the `Authorization` header is missing or invalid
- **THEN** respond `401` using the unified error envelope

#### Scenario: Filter by provider
- **WHEN** `?provider=github` is supplied
- **THEN** only signals for `provider_slug='github'` are returned

#### Scenario: Filter by connection and kind
- **WHEN** `?connection_id=<uuid>&kind=issue_created` is supplied
- **THEN** only signals matching both are returned

#### Scenario: Time window filters
- **WHEN** `?occurred_after=2024-10-01T00:00:00Z` is supplied
- **THEN** only signals with `occurred_at >=` the value are returned
- **WHEN** `?occurred_before=2024-10-02T00:00:00Z` is supplied
- **THEN** only signals with `occurred_at <=` the value are returned
- **WHEN** both `occurred_after` and `occurred_before` are supplied
- **THEN** only signals with `occurred_at` within the inclusive window are returned

### Requirement: Cursor Pagination
The endpoint MUST support cursor-based pagination using `limit` and `cursor`. Results SHALL be ordered by `occurred_at DESC, id DESC` to ensure a stable cursor.

- Request: `?limit=50&cursor=<opaque>` where `limit` default is 50 and max is 100
- Response: always includes `next_cursor`; set it to a non-empty opaque string when more results are available or to `null` when this is the last page
- The `cursor` is an opaque token encoding the last item position

#### Scenario: First page with next_cursor
- **GIVEN** more than `limit` matching signals exist
- **WHEN** requesting `GET /signals?limit=2`
- **THEN** response contains 2 signals in order and a non-empty `next_cursor`

#### Scenario: Next page returns subsequent items
- **GIVEN** a `next_cursor` from a prior response
- **WHEN** calling `GET /signals?cursor=<token>&limit=2`
- **THEN** the next 2 signals are returned

#### Scenario: Last page sets next_cursor to null
- **GIVEN** the current page consumes the remaining results
- **WHEN** calling `GET /signals` such that no further pages exist
- **THEN** respond with the remaining signals and `next_cursor: null`

#### Scenario: Limit bounds enforced
- **WHEN** `limit` is greater than 100 or less than 1
- **THEN** respond `400` with a validation error using the unified envelope

#### Scenario: Empty result returns empty list
- **WHEN** no signals match the filters
- **THEN** respond `200 OK` with `{ signals: [], next_cursor: null }`

#### Scenario: Invalid cursor rejected
- **WHEN** the supplied `cursor` cannot be decoded or does not align with the pagination ordering
- **THEN** respond `400` with the unified error envelope and `details.cursor` describing the problem

### Requirement: Optional Payload Inclusion
The endpoint SHALL support an `include_payload` boolean query parameter (default `false`). When `true`, the `payload` field is included in each signal item.

#### Scenario: Payload included when requested
- **WHEN** `GET /signals?include_payload=true`
- **THEN** each signal contains the `payload` object in the response

### Requirement: Query Validation
All query parameters MUST be validated before executing the listing.

#### Scenario: Invalid UUID rejected
- **WHEN** `?connection_id=not-a-uuid` is supplied
- **THEN** respond `400` with the unified error envelope and `details.connection_id` explaining the UUID parsing error

#### Scenario: Invalid timestamp rejected
- **WHEN** `?occurred_after=yesterday` or `?occurred_before=2024-99-01T00:00:00Z` is supplied
- **THEN** respond `400` with the unified error envelope and `details.occurred_after`/`details.occurred_before` indicating the value must be RFC3339

### Requirement: Stable Ordering
Results MUST be deterministically ordered by `occurred_at DESC, id DESC` for pagination stability.

#### Scenario: Ties broken by id
- **WHEN** multiple signals share the same `occurred_at`
- **THEN** they are secondarily ordered by `id DESC` to maintain consistency

#### Scenario: OpenAPI documented
- **WHEN** OpenAPI is generated
- **THEN** the `/signals` path and response schemas are present in Swagger UI
