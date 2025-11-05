## MODIFIED Requirements

### Requirement: Webhook Ingest Endpoint (MVP)
The API SHALL provide a webhook ingest endpoint at `POST /webhooks/{provider}` to receive callbacks from external providers. For MVP, the endpoint is protected by operator auth and tenant scoping; a later change will make it publicly accessible with signature verification.

Details (MVP):
- Path parameter `provider` MUST match a known provider slug.
- Authentication: Requires `Authorization: Bearer <token>` and `X-Tenant-Id` (UUID) per auth spec.
- Headers: MAY include `X-Connection-Id` (UUID) to target a connection.
- Body: `Content-Type: application/json`; body is opaque to the API and MAY be recorded in a future change.
- Responses:
  - Success: `202 Accepted` with JSON body `{ "status": "accepted" }`.
  - Errors: Use the unified error envelope (`application/problem+json`) defined in `api-core`, e.g., `{ code: "NOT_FOUND", message: "...", details?: { ... }, trace_id?: "..." }`.

#### Scenario: Accepts webhook and returns 202
- GIVEN a known provider slug `github` and valid operator auth with `X-Tenant-Id`
- WHEN calling `POST /webhooks/github` with a JSON body
- THEN the response is HTTP 202 with a minimal JSON body `{ "status": "accepted" }`

#### Scenario: Unknown provider returns 404
- WHEN calling `POST /webhooks/unknown` with valid auth and tenant
- THEN the response is HTTP 404 with the unified error envelope, e.g., `{ code: "NOT_FOUND", message: "Unknown provider: unknown" }`

#### Scenario: Missing tenant header returns 400
- WHEN calling `POST /webhooks/github` without `X-Tenant-Id`
- THEN the response is HTTP 400 with the unified error envelope, e.g., `{ code: "VALIDATION_FAILED", message: "Missing X-Tenant-Id" }`

## ADDED Requirements

### Requirement: Public Webhook Access With Signature Verification
The webhook ingest endpoint SHALL accept public requests without operator auth when a valid provider signature is present. Public calls SHALL be supported at `POST /webhooks/{provider}/{tenant_id}` to convey tenant context.

Details:
- Authorization precedence: A valid operator bearer token SHALL grant access regardless of signature presence or validity. Signature verification is evaluated only when no valid operator token is present.
- The path variant `POST /webhooks/{provider}/{tenant_id}` SHALL be documented and recommended for provider configuration. The operator-protected path `POST /webhooks/{provider}` SHALL continue to function for local/test.
- If no valid operator token is supplied, requests lacking a valid provider signature MUST be rejected with HTTP 401.

Validation order (decision flow):
- Operator token present and valid → ACCEPT (202); signature not required/ignored.
- Operator token missing/invalid AND signature present and valid → ACCEPT (202).
- Operator token missing/invalid AND signature missing/invalid → REJECT (401 Unauthorized).

#### Scenario: Public signed request accepted
- GIVEN `POBLYSH_WEBHOOK_GITHUB_SECRET` is configured and a valid GitHub signature is provided
- WHEN calling `POST /webhooks/github/{tenant}` without operator auth
- THEN the response is HTTP 202 and the request is processed

#### Scenario: Invalid signature rejected (no operator auth)
- WHEN calling `POST /webhooks/github/{tenant}` with an invalid signature and without operator auth
- THEN the response is HTTP 401 and the request is not processed

#### Scenario: Missing signature rejected (no operator auth)
- WHEN calling `POST /webhooks/github/{tenant}` without `X-Hub-Signature-256`
- THEN the response is HTTP 401 and the request is not processed

### Requirement: GitHub HMAC-SHA256 Verification
GitHub webhooks MUST be verified using HMAC-SHA256 with the configured secret. The server SHALL compute `digest = hex(hmac_sha256(secret, raw_body))` and compare in constant time to the `X-Hub-Signature-256` header value (prefix `sha256=`).

Headers:
- `X-Hub-Signature-256: sha256=<hex-digest>`
- `X-GitHub-Delivery` MAY be recorded for diagnostics

#### Scenario: Valid GitHub signature returns 202
- GIVEN a request body `B` and secret `S`
- AND header `X-Hub-Signature-256` equals `sha256=` + hex(hmac_sha256(S, B))
- WHEN calling the endpoint
- THEN the response is HTTP 202

#### Scenario: Missing or malformed GitHub signature returns 401
- WHEN `X-Hub-Signature-256` is missing or not prefixed with `sha256=`
- THEN the response is HTTP 401

### Requirement: Slack v2 Signature Verification
Slack webhooks MUST be verified using HMAC-SHA256 over the base string `v0:{timestamp}:{raw_body}` with the configured signing secret. The server SHALL enforce a timestamp tolerance window of 5 minutes to prevent replay.

Headers:
- `X-Slack-Signature: v0=<hex-digest>`
- `X-Slack-Request-Timestamp: <unix-seconds>`

#### Scenario: Valid Slack signature within window returns 202
- GIVEN `X-Slack-Request-Timestamp` is within 300 seconds of server time
- AND `X-Slack-Signature` equals `v0=` + hex(hmac_sha256(secret, "v0:" + ts + ":" + raw_body))
- WHEN calling the endpoint
- THEN the response is HTTP 202

#### Scenario: Slack timestamp too old returns 401
- GIVEN `X-Slack-Request-Timestamp` is older than 300 seconds
- WHEN calling the endpoint
- THEN the response is HTTP 401

#### Scenario: Invalid Slack signature returns 401
- WHEN the computed signature does not match
- THEN the response is HTTP 401

### Requirement: OpenAPI Documentation (Public Webhooks)
The OpenAPI document SHALL document signature headers for supported providers and indicate that the public path variant does not require bearer auth when the signature is valid.

#### Scenario: OpenAPI includes signature header schemas
- WHEN fetching `/openapi.json`
- THEN `POST /webhooks/{provider}/{tenant_id}` documents provider-specific signature headers and no bearer auth requirement

### Requirement: Webhook Verification Telemetry and Protections
The system SHALL emit structured telemetry and enforce abuse protections for public webhook verification.

Details:
- Apply per-IP and global rate limiting (document burst/window defaults) before signature evaluation for unauthenticated requests; limits SHOULD be configurable.
- Emit structured logs for every verification attempt (provider, tenant, request ID if present, outcome, and error reason) with secrets redacted.
- Expose metrics counters (`signature_verification_success`, `signature_verification_failure`, `signature_verification_replay_reject`) and latency histograms suitable for dashboards and alerting.
- Document integration points for upstream WAF/CDN shields and retention/alert SLAs (30–90 day log retention, alerts on failure spikes or DDoS indicators).

#### Scenario: Rate limit enforced before signature evaluation
- GIVEN a single IP exceeds the configured rate limit
- WHEN sending additional unsigned requests
- THEN the system rejects them per rate-limit policy before attempting signature verification

#### Scenario: Verification attempts logged and metered
- WHEN processing webhook requests
- THEN structured logs and metrics capture outcome, provider, and redacted context for every attempt
