# api-webhooks Specification

## Purpose
TBD - created by archiving change add-webhook-ingest-endpoint. Update Purpose after archive.
## Requirements
### Requirement: Webhook Ingest Endpoint (MVP)
The API SHALL provide a webhook ingest endpoint at `POST /webhooks/{provider}` to receive callbacks from external providers. For MVP, the endpoint is protected by operator auth and tenant scoping; a later change will make it publicly accessible with signature verification.

Details (MVP):
- Path parameter `provider` MUST match a known provider slug.
- Authentication: Requires `Authorization: Bearer <token>` and `X-Tenant-Id` (UUID) per auth spec.
- Headers: MAY include `X-Connection-Id` (UUID) to target a connection; malformed UUIDs SHALL be rejected with HTTP 400 using the unified error envelope.
- Headers: ALL incoming headers SHALL be captured in canonical lower-case form and stored in the job cursor for downstream processing (required for signature verification and event routing by connectors), EXCEPT for sensitive authentication headers which SHALL be filtered out for security: `authorization`, `cookie`, `set-cookie`, `proxy-authorization`, `www-authenticate`, `authentication-info`, `x-api-key`, `x-auth-token`, `x-csrf-token`, `x-xsrf-token`.
- Body: `Content-Type: application/json`; body is captured regardless of Content-Length or Transfer-Encoding and stored in the job cursor.
- Response: `202 Accepted` on success; validation/auth failures respond with HTTP 400/401; unknown provider or tenant-scoped connection lookups respond with HTTP 404. All non-2xx responses MUST use the unified problem+json error envelope with SCREAMING_SNAKE_CASE `code`, descriptive `message`, optional `details`, and emitted `trace_id`.

#### Scenario: Accepts webhook and returns 202
- GIVEN a known provider slug `github` and valid operator auth with `X-Tenant-Id`
- WHEN calling `POST /webhooks/github` with a JSON body
- THEN the response is HTTP 202 with a minimal JSON body `{ "status": "accepted" }`

#### Scenario: Missing authorization returns 401
- WHEN calling `POST /webhooks/github` without `Authorization: Bearer`
- THEN the response is HTTP 401 with problem+json body `{ "code": "UNAUTHORIZED", "message": "missing or invalid operator token", "trace_id": "..." }`

#### Scenario: Unknown provider returns 404
- WHEN calling `POST /webhooks/unknown` with valid auth and tenant
- THEN the response is HTTP 404 with problem+json body `{ "code": "NOT_FOUND", "message": "provider 'unknown' not found", "trace_id": "..." }`

#### Scenario: Missing tenant header returns 400
- WHEN calling `POST /webhooks/github` without `X-Tenant-Id`
- THEN the response is HTTP 400 with problem+json body `{ "code": "VALIDATION_FAILED", "message": "missing tenant header", "trace_id": "..." }`

#### Scenario: Malformed connection header returns 400
- WHEN calling `POST /webhooks/github` with `X-Connection-Id: not-a-uuid`
- THEN the response is HTTP 400 with problem+json body `{ "code": "VALIDATION_FAILED", "message": "invalid X-Connection-Id header", "trace_id": "..." }`

### Requirement: Enqueue Webhook Sync Job
If a valid `X-Connection-Id` is provided and belongs to the tenant and provider, the system SHALL enqueue a `sync_jobs` row with `job_type = "webhook"` to process the event asynchronously.

Details (MVP):
- The job SHALL reference `(tenant_id, provider_slug, connection_id)`.
- The job SHALL include a JSON `cursor` payload with the following structure:
  ```json
  {
    "webhook_headers": {
      "content-type": "application/json",
      "x-github-delivery": "12345678-1234-1234-1234-123456789012",
      "x-github-event": "push",
      "x-hub-signature-256": "sha256=...",
      "user-agent": "GitHub-Hookshot/abc123"
    },
    "webhook_payload": {
      "repository": { "name": "example-repo" },
      "ref": "refs/heads/main",
      "commits": [...]
    },
    "received_at": "2025-01-15T10:30:00Z"
  }
  ```
- Headers SHALL be stored in canonical lower-case form for consistent access by connectors.
- Sensitive headers SHALL be filtered out and NOT persisted in the job cursor for security reasons.
- The payload SHALL be captured as parsed JSON when possible; malformed JSON SHALL be stored as null.
- The handler SHALL return `202 Accepted` regardless of subsequent job execution outcome.

#### Scenario: Enqueues job when connection is valid
- GIVEN a connection `(tenant=T1, provider='github', id=C1)` exists
- WHEN calling `POST /webhooks/github` with headers `X-Tenant-Id=T1` and `X-Connection-Id=C1`
- THEN a `sync_jobs` row is created with `job_type='webhook'` referencing `(T1, 'github', C1)`
- AND the API responds `202 Accepted`

#### Scenario: Invalid connection returns 404
- GIVEN tenant `T1`
- WHEN calling with `X-Connection-Id=CX` that does not exist for `(T1, 'github')`
- THEN the response is HTTP 404 with problem+json body `{ "code": "NOT_FOUND", "message": "connection not found for tenant/provider", "trace_id": "..." }`

### Requirement: OpenAPI Documentation
The OpenAPI document SHALL include `POST /webhooks/{provider}` with path parameter `provider`, optional `X-Connection-Id` header, and indicate that authentication is required.

#### Scenario: OpenAPI path documented
- WHEN fetching `/openapi.json`
- THEN the document describes `POST /webhooks/{provider}` with the expected parameters and security requirements

