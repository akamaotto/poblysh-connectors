# auth Specification

## Purpose
TBD - created by archiving change add-operator-bearer-auth-and-tenant-header. Update Purpose after archive.
## Requirements
### Requirement: Operator Bearer Authentication
The API SHALL require operator authentication using the HTTP `Authorization: Bearer <token>` scheme for protected endpoints.

Configuration (MVP):
- `POBLYSH_OPERATOR_TOKENS` (comma‑separated) or `POBLYSH_OPERATOR_TOKEN` (single) provides allowed tokens.
- For profiles `local` and `test`: at least one token MUST be present; for other profiles: startup MUST fail if none are provided.
- Token comparisons MUST be constant‑time to mitigate timing attacks.

#### Scenario: Missing Authorization returns 401
- WHEN a protected endpoint is called without `Authorization`
- THEN the response is HTTP 401 with `{ code: "unauthorized" }`

#### Scenario: Invalid token returns 401
- WHEN `Authorization: Bearer invalid` is provided
- THEN the response is HTTP 401 with `{ code: "unauthorized" }`

#### Scenario: Valid token authenticates operator
- GIVEN `POBLYSH_OPERATOR_TOKEN=secret123`
- WHEN `Authorization: Bearer secret123` is provided
- THEN the request is authenticated and proceeds to the handler

### Requirement: Tenant Header Enforcement
Protected endpoints MUST require `X-Tenant-Id` header with a valid UUID string. The effective tenant ID SHALL be propagated to request context for downstream database queries and auditing.

#### Scenario: Missing tenant header returns 400
- WHEN a protected endpoint is called without `X-Tenant-Id`
- THEN the response is HTTP 400 with `{ code: "validation_failed" }`

#### Scenario: Invalid tenant UUID returns 400
- WHEN `X-Tenant-Id` is present but not a valid UUID
- THEN the response is HTTP 400 with `{ code: "validation_failed" }`

#### Scenario: Tenant ID available to handler
- WHEN a protected endpoint is called with a valid `X-Tenant-Id`
- THEN the handler can access the parsed tenant UUID from request context

### Requirement: Public Endpoints Bypass
The following endpoints SHALL be accessible without authentication or tenant header: `/healthz`, `/readyz`, `/docs`, `/openapi.json`.

#### Scenario: Health without auth
- WHEN calling `GET /healthz` without headers
- THEN the response is HTTP 200

### Requirement: OpenAPI Security And Header Documentation
The OpenAPI document SHALL declare an HTTP bearer security scheme and annotate protected endpoints accordingly. The `X-Tenant-Id` header SHALL be documented as a required parameter for protected endpoints.

#### Scenario: Security scheme present
- WHEN fetching `/openapi.json`
- THEN the document includes a `securitySchemes` entry for HTTP bearer auth
- AND protected endpoints list the scheme in their `security` section
- AND the `X-Tenant-Id` header appears as a required header parameter

### Requirement: Error Model Integration
Authentication and tenant validation errors SHALL use the unified error envelope.

#### Scenario: Unauthorized uses unified shape
- WHEN authentication fails
- THEN the response body is `{ code: "unauthorized", message: "..." }` with media type `application/problem+json`

