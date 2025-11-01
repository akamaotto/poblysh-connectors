## ADDED Requirements
### Requirement: Connections Listing Endpoint
The system SHALL expose `GET /connections` to list active connections for the specified tenant.

#### Scenario: Returns tenant-scoped connections
- **WHEN** a client calls `GET /connections` with a valid `Authorization` token and `X-Tenant-Id`
- **THEN** respond `200 OK` with JSON body `{ connections: [ { id: uuid, provider: string, expires_at?: RFC3339 string, metadata: object } ] }`

#### Scenario: Missing tenant header returns 400
- **WHEN** `X-Tenant-Id` is not provided
- **THEN** respond `400` using the unified error envelope

#### Scenario: Unauthorized without bearer token
- **WHEN** the `Authorization` header is missing or invalid
- **THEN** respond `401` using the unified error envelope

#### Scenario: Filter by provider
- **WHEN** a `provider` query parameter is supplied (e.g., `?provider=github`)
- **THEN** only connections for that provider are returned

#### Scenario: Empty result returns empty list
- **WHEN** the tenant has no connections (or none match the filter)
- **THEN** respond `200 OK` with `{ connections: [] }`

#### Scenario: Stable ordering
- **WHEN** multiple connections are returned
- **THEN** they are ordered by `id` ascending (stable ordering)

#### Scenario: OpenAPI documented
- **WHEN** OpenAPI is generated
- **THEN** the `/connections` path and response schemas are present in Swagger UI

