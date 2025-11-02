## ADDED Requirements
### Requirement: Providers Listing Endpoint
The system SHALL expose `GET /providers` returning available provider integrations with registry metadata.

#### Scenario: Returns provider registry metadata
- **WHEN** a client calls `GET /providers`
- **THEN** respond `200 OK` with JSON body `{ providers: [ { name: string, auth_type: string, scopes: string[], webhooks: boolean } ] }`

#### Scenario: Empty registry returns empty list
- **WHEN** no providers are registered
- **THEN** respond `200 OK` with `{ providers: [] }`

#### Scenario: Sorted by name ascending
- **WHEN** multiple providers exist
- **THEN** the response array is sorted by `name` ascending with stable ordering

#### Scenario: OpenAPI documented
- **WHEN** OpenAPI is generated
- **THEN** the `/providers` path and response schemas are present in Swagger UI

#### Scenario: Public endpoint (no auth)
- **WHEN** a request lacks `Authorization` and `X-Tenant-Id`
- **THEN** the endpoint still responds with `200 OK` and the providers list

