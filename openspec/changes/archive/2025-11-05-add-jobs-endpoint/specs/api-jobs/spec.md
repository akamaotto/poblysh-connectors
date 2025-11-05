## ADDED Requirements
### Requirement: Jobs Listing Endpoint
The system SHALL expose `GET /jobs` to list sync jobs for the specified tenant with filters and cursor pagination.

#### Scenario: Returns tenant-scoped jobs
- **WHEN** a client calls `GET /jobs` with a valid `Authorization` token and `X-Tenant-Id`
- **THEN** respond `200 OK` with JSON body `{ jobs: [ { id: uuid, provider_slug: string, connection_id: uuid, job_type: "full"|"incremental"|"webhook", status: "queued"|"running"|"succeeded"|"failed", priority: integer, attempts: integer, scheduled_at: RFC3339, retry_after?: RFC3339, started_at?: RFC3339, finished_at?: RFC3339 } ], next_cursor: string|null }`
- **AND** `next_cursor` is always present: use a non-empty opaque string when another page exists, otherwise set it to `null`

#### Scenario: Missing tenant header returns 400
- **WHEN** `X-Tenant-Id` is not provided
- **THEN** respond `400` using the unified error envelope

#### Scenario: Unauthorized without bearer token
- **WHEN** the `Authorization` header is missing or invalid
- **THEN** respond `401` using the unified error envelope

#### Scenario: Filter by status
- **WHEN** `?status=running` is supplied
- **THEN** only jobs with `status = 'running'` are returned

#### Scenario: Filter by provider and connection
- **WHEN** `?provider=github&connection_id=<uuid>` is supplied
- **THEN** only jobs for `provider_slug='github'` and the specified `connection_id` are returned

#### Scenario: Filter by job_type
- **WHEN** `?job_type=incremental` is supplied
- **THEN** only incremental jobs are returned

#### Scenario: Time window filters
- **WHEN** `?started_after=2024-10-01T00:00:00Z` is supplied
- **THEN** only jobs with `started_at >=` the value are returned
- **WHEN** `?finished_after=2024-10-15T00:00:00Z` is supplied (alone or combined with `started_after`)
- **THEN** only jobs with `finished_at >=` the value are returned while still respecting the `started_after` filter when both are present

#### Scenario: Reject unknown status or job_type
- **WHEN** `?status=pending` or `?job_type=batch` is supplied
- **THEN** respond `400` with the unified error envelope and `details` indicating the unsupported value

### Requirement: Cursor Pagination
The endpoint MUST support cursor-based pagination using `limit` and `cursor`. Results SHALL be ordered by `scheduled_at DESC, id DESC` to ensure a stable cursor.

- Request: `?limit=50&cursor=<opaque>` where `limit` default is 50 and max is 100
- Response: always includes `next_cursor`; set it to a non-empty string when more results are available or to `null` when this is the last page
- The `cursor` is an opaque token encoding the last item position

#### Scenario: First page with next_cursor
- **GIVEN** more than `limit` matching jobs exist
- **WHEN** requesting `GET /jobs?limit=2`
- **THEN** response contains 2 jobs in order and a non-empty `next_cursor`

#### Scenario: Next page returns subsequent items
- **GIVEN** a `next_cursor` from a prior response
- **WHEN** calling `GET /jobs?cursor=<token>&limit=2`
- **THEN** the next 2 jobs are returned

#### Scenario: Limit bounds enforced
- **WHEN** `limit` is greater than 100 or less than 1
- **THEN** respond `400` with a validation error using the unified envelope

#### Scenario: Empty result returns empty list
- **WHEN** no jobs match the filters
- **THEN** respond `200 OK` with `{ jobs: [], next_cursor: null }`

#### Scenario: Invalid cursor rejected
- **WHEN** the supplied `cursor` cannot be decoded or does not match the pagination ordering
- **THEN** respond `400` with the unified error envelope and structured `details.cursor` explaining the problem

#### Scenario: OpenAPI documented
- **WHEN** OpenAPI is generated
- **THEN** the `/jobs` path and response schemas are present in Swagger UI

### Requirement: Stable Ordering
Results MUST be deterministically ordered by `scheduled_at DESC, id DESC` for pagination stability.

#### Scenario: Ties broken by id
- **WHEN** multiple jobs share the same `scheduled_at`
- **THEN** they are secondarily ordered by `id DESC` to maintain consistency

### Requirement: Query Validation
All query parameters MUST be validated before executing the listing.

#### Scenario: Invalid UUID rejected
- **WHEN** `?connection_id=not-a-uuid` is supplied
- **THEN** respond `400` with the unified error envelope and `details.connection_id` explaining the UUID parsing error

#### Scenario: Invalid timestamp rejected
- **WHEN** `?started_after=yesterday` or `?finished_after=2024-99-01T00:00:00Z` is supplied
- **THEN** respond `400` with the unified error envelope and `details.started_after`/`details.finished_after` indicating the value must be RFC3339
