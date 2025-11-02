## ADDED Requirements

### Requirement: Webhook Ingest Endpoint (MVP)
The API SHALL provide a webhook ingest endpoint at `POST /webhooks/{provider}` to receive callbacks from external providers. For MVP, the endpoint is protected by operator auth and tenant scoping; a later change will make it publicly accessible with signature verification.

Details (MVP):
- Path parameter `provider` MUST match a known provider slug.
- Authentication: Requires `Authorization: Bearer <token>` and `X-Tenant-Id` (UUID) per auth spec.
- Headers: MAY include `X-Connection-Id` (UUID) to target a connection.
- Body: `Content-Type: application/json`; body is opaque to the API and MAY be recorded in a future change.
- Response: `202 Accepted` on success; `404` if provider/connection not found; `400` on validation errors.

#### Scenario: Accepts webhook and returns 202
- GIVEN a known provider slug `github` and valid operator auth with `X-Tenant-Id`
- WHEN calling `POST /webhooks/github` with a JSON body
- THEN the response is HTTP 202 with a minimal JSON body `{ "status": "accepted" }`

#### Scenario: Unknown provider returns 404
- WHEN calling `POST /webhooks/unknown` with valid auth and tenant
- THEN the response is HTTP 404 with `{ code: "not_found" }`

#### Scenario: Missing tenant header returns 400
- WHEN calling `POST /webhooks/github` without `X-Tenant-Id`
- THEN the response is HTTP 400 with `{ code: "validation_failed" }`

### Requirement: Enqueue Webhook Sync Job
If a valid `X-Connection-Id` is provided and belongs to the tenant and provider, the system SHALL enqueue a `sync_jobs` row with `job_type = "webhook"` to process the event asynchronously.

Details (MVP):
- The job SHALL reference `(tenant_id, provider_slug, connection_id)`.
- The job MAY include a minimal JSON `cursor` payload capturing webhook context (e.g., event id, delivery id).
- The handler SHALL return `202 Accepted` regardless of subsequent job execution outcome.

#### Scenario: Enqueues job when connection is valid
- GIVEN a connection `(tenant=T1, provider='github', id=C1)` exists
- WHEN calling `POST /webhooks/github` with headers `X-Tenant-Id=T1` and `X-Connection-Id=C1`
- THEN a `sync_jobs` row is created with `job_type='webhook'` referencing `(T1, 'github', C1)`
- AND the API responds `202 Accepted`

#### Scenario: Invalid connection returns 404
- GIVEN tenant `T1`
- WHEN calling with `X-Connection-Id=CX` that does not exist for `(T1, 'github')`
- THEN the response is HTTP 404 with `{ code: "not_found" }`

### Requirement: OpenAPI Documentation
The OpenAPI document SHALL include `POST /webhooks/{provider}` with path parameter `provider`, optional `X-Connection-Id` header, and indicate that authentication is required.

#### Scenario: OpenAPI path documented
- WHEN fetching `/openapi.json`
- THEN the document describes `POST /webhooks/{provider}` with the expected parameters and security requirements

