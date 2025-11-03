# api-connections Specification

## Purpose
TBD - created by archiving change add-oauth-start-endpoint. Update Purpose after archive.
## Requirements
### Requirement: OAuth Start Endpoint
The system SHALL expose `POST /connect/{provider}` to initiate an OAuth flow for the specified provider and tenant, returning an authorization URL for client redirection.

#### Scenario: Returns authorize URL for provider
- **WHEN** a client calls `POST /connect/{provider}` with a valid `Authorization` token and `X-Tenant-Id`
- **THEN** respond `200 OK` with JSON body `{ authorize_url: string }` where `authorize_url` is a fully formed provider URL (including `state` if used) that MUST be HTTPS, valid according to RFC 3986, with maximum length 2048 characters, and MUST NOT include a fragment component

#### Scenario: Unknown provider returns 404
- **WHEN** `{provider}` does not exist in the provider registry
- **THEN** respond `404` using the unified error envelope `{ code: "NOT_FOUND", message: "provider '{provider}' not found" }`

#### Scenario: Missing tenant header returns 400
- **WHEN** `X-Tenant-Id` is not provided
- **THEN** respond `400` using the unified error envelope `{ code: "VALIDATION_FAILED", message: "missing tenant header" }`

#### Scenario: Unauthorized without bearer token
- **WHEN** the `Authorization` header is missing or invalid
- **THEN** respond `401` using the unified error envelope `{ code: "UNAUTHORIZED", message: "invalid or missing authorization token" }`

#### Scenario: Provider param format
- **WHEN** `{provider}` is provided as a snake_case ID (e.g., `github`)
- **THEN** the system resolves it against the registry and rejects other formats as unknown

#### Scenario: OpenAPI documented
- **WHEN** OpenAPI is generated
- **THEN** the `/connect/{provider}` path, security requirements, `provider` path parameter, and response schemas are present in Swagger UI

### Requirement: Connections Listing Endpoint
The system SHALL expose `GET /connections` to list active connections for the specified tenant.

#### Scenario: Returns tenant-scoped connections
- **WHEN** a client calls `GET /connections` with a valid `Authorization` token and `X-Tenant-Id`
- **THEN** respond `200 OK` with JSON body `{ connections: [ { id: uuid, provider: string, expires_at?: RFC3339 string, metadata: object } ] }`

#### Scenario: Missing tenant header returns 400
- **WHEN** `X-Tenant-Id` is not provided
- **THEN** respond `400` using the unified error envelope `{ code: "VALIDATION_FAILED", message: "missing tenant header" }`

#### Scenario: Unauthorized without bearer token
- **WHEN** the `Authorization` header is missing or invalid
- **THEN** respond `401` using the unified error envelope `{ code: "UNAUTHORIZED", message: "invalid or missing authorization token" }`

#### Scenario: Filter by provider
- **WHEN** a `provider` query parameter is supplied (e.g., `?provider=github`)
- **THEN** only connections for that provider are returned

#### Scenario: Invalid provider returns 400
- **WHEN** a `provider` query parameter does not match a known provider in the registry
- **THEN** respond `400` using the unified error envelope `{ code: "VALIDATION_FAILED", message: "unknown provider" }`

#### Scenario: Empty result returns empty list
- **WHEN** the tenant has no connections (or none match the filter)
- **THEN** respond `200 OK` with `{ connections: [] }`

#### Scenario: Stable ordering
- **WHEN** multiple connections are returned
- **THEN** they are ordered by `id` ascending (stable ordering)

#### Scenario: OpenAPI documented
- **WHEN** OpenAPI is generated
- **THEN** the `/connections` path and response schemas are present in Swagger UI

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
- **THEN** respond `404` using the application/problem+json error envelope `{ code: "NOT_FOUND", message: "provider '{provider}' not found", trace_id: "..." }`

#### Scenario: Missing or invalid state returns 400
- **WHEN** `state` is missing, expired, or fails validation
- **THEN** respond `400` using the application/problem+json error envelope `{ code: "VALIDATION_FAILED", message: "missing, expired, or invalid state parameter", trace_id: "..." }`

#### Scenario: Missing code returns 400
- **WHEN** `code` is missing from the query
- **THEN** respond `400` using the application/problem+json error envelope `{ code: "VALIDATION_FAILED", message: "missing authorization code parameter", trace_id: "..." }`

#### Scenario: Provider denies authorization
- **WHEN** the provider redirects with `error` query parameter (e.g., `access_denied`)
- **THEN** respond `400` using the application/problem+json error envelope with details including the provider error reason `{ code: "VALIDATION_FAILED", message: "provider denied authorization", details: { provider_error: "access_denied" }, trace_id: "..." }`

#### Scenario: Upstream provider HTTP failure maps to 502
- **WHEN** the token exchange call to the provider fails with a non‑2xx HTTP status
- **THEN** respond `502` with `{ code: "PROVIDER_ERROR", message: "upstream provider error", details: { provider: { name: "{provider}", status: <http-status> } }, trace_id: "..." }`

#### Scenario: State parameter expired returns 400
- **WHEN** the `state` parameter has expired (e.g., older than 10 minutes as per OAuth 2.0 security best practices)
- **THEN** respond `400` with `{ code: "VALIDATION_FAILED", message: "state parameter has expired", details: { error_type: "state_expired" }, trace_id: "..." }`

#### Scenario: CSRF validation failure returns 400
- **WHEN** the `state` parameter cannot be validated against the stored value or signature verification fails
- **THEN** respond `400` with `{ code: "VALIDATION_FAILED", message: "CSRF validation failed", details: { error_type: "csrf_validation_failed" }, trace_id: "..." }`

#### Scenario: Malformed provider response returns 502
- **WHEN** the provider returns a malformed or unexpected response format during token exchange
- **THEN** respond `502` with `{ code: "PROVIDER_ERROR", message: "provider returned malformed response", details: { provider: { name: "{provider}", error: "malformed_response" } }, trace_id: "..." }`

#### Scenario: Duplicate state usage returns 400
- **WHEN** the `state` parameter has already been used successfully (prevents replay attacks)
- **THEN** respond `400` with `{ code: "VALIDATION_FAILED", message: "state parameter already used", details: { error_type: "state_reused" }, trace_id: "..." }`

#### Scenario: Provider param format
- **WHEN** `{provider}` is provided as a snake_case ID (e.g., `github`)
- **THEN** the system resolves it against the registry and rejects other formats as unknown

#### Scenario: OpenAPI documented
- **WHEN** OpenAPI is generated
- **THEN** the `/connect/{provider}/callback` path, query parameters (`code`, `state`), and response schemas are present in Swagger UI

