## ADDED Requirements
### Requirement: OAuth Callback Endpoint
The system SHALL expose `GET /connect/{provider}/callback` to finalize OAuth by exchanging the authorization `code` for tokens and creating a tenant-scoped connection.

#### Scenario: Successful token exchange creates connection
- **WHEN** the provider redirects to `GET /connect/{provider}/callback?code=...&state=...`
- **THEN** the server validates `state`, resolves the tenant, exchanges `code` via the connector, persists the connection, and responds `200 OK` with `{ connection: { id: uuid, provider: string, expires_at?: RFC3339 string, metadata: object } }`

#### Scenario: Public endpoint (no auth)
- **WHEN** the callback is invoked without `Authorization` or `X-Tenant-Id`
- **THEN** the endpoint still processes the request based on `state` and responds accordingly

#### Scenario: Unknown provider returns 404
- **WHEN** `{provider}` does not exist in the provider registry
- **THEN** respond `404` using the unified error envelope `{ code: "not_found", message: "unknown provider" }`

#### Scenario: Missing or invalid state returns 400
- **WHEN** `state` is missing, expired, or fails validation
- **THEN** respond `400` using the unified error envelope `{ code: "validation_failed" }`

#### Scenario: Missing code returns 400
- **WHEN** `code` is missing from the query
- **THEN** respond `400` using the unified error envelope `{ code: "validation_failed" }`

#### Scenario: Provider denies authorization
- **WHEN** the provider redirects with `error` query parameter (e.g., `access_denied`)
- **THEN** respond `400` using the unified error envelope with details including the provider error reason

#### Scenario: Upstream provider HTTP failure maps to 502
- **WHEN** the token exchange call to the provider fails with a non‑2xx HTTP status
- **THEN** respond `502` with `{ code: "provider_error", details: { provider: "{provider}", status: <http-status> } }`

#### Scenario: OpenAPI documented
- **WHEN** OpenAPI is generated
- **THEN** the `/connect/{provider}/callback` path, query parameters (`code`, `state`), and response schemas are present in Swagger UI

