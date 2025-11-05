## ADDED Requirements
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

