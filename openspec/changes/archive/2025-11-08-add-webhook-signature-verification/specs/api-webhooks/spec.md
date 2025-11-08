connectors/openspec/changes/add-webhook-signature-verification/specs/api-webhooks/spec.md
## MODIFIED Requirements

### Requirement: Webhook Ingest Endpoint (MVP)
The API SHALL provide a webhook ingest endpoint at `POST /webhooks/{provider}` to receive callbacks from external providers. For MVP, the endpoint is protected by operator auth and tenant scoping; a later change (this specification) adds public access with signature verification while preserving this behavior.

Details (MVP):
- Path parameter `provider` MUST match a known provider slug.
- Authentication: Requires `Authorization: Bearer <token>` and `X-Tenant-Id` (UUID) per auth spec.
- Headers: MAY include `X-Connection-Id` (UUID) to target a connection.
- Body: `Content-Type: application/json`; body is opaque to the API and MAY be recorded or routed in future changes.
- Responses:
  - Success: `202 Accepted` with JSON body `{ "status": "accepted" }`.
  - Errors: Use the unified error envelope (`application/problem+json`) defined in `api-core`, e.g.:
    - `{ "code": "NOT_FOUND", "message": "...", "details"?: { ... }, "trace_id"?: "..." }`.
    - `{ "code": "UNAUTHORIZED", "message": "...", "details"?: { ... }, "trace_id"?: "..." }`.
    - `{ "code": "INVALID_SIGNATURE", "message": "...", "details"?: { ... }, "trace_id"?: "..." }`.
    - `{ "code": "RATE_LIMITED", "message": "...", "details"?: { ... }, "trace_id"?: "..." }`.

#### Scenario: Accepts webhook and returns 202
- GIVEN a known provider slug `github` and valid operator auth with `X-Tenant-Id`
- WHEN calling `POST /webhooks/github` with a JSON body
- THEN the response is HTTP 202 with a minimal JSON body `{ "status": "accepted" }`

#### Scenario: Unknown provider returns 404
- GIVEN the `provider` path parameter does not match any known provider
- WHEN calling `POST /webhooks/unknown` with valid auth and tenant
- THEN the response is HTTP 404 with the unified error envelope and `code = "NOT_FOUND"`

#### Scenario: Missing tenant header returns 400
- GIVEN a request without `X-Tenant-Id`
- WHEN calling `POST /webhooks/github` with valid operator auth
- THEN the response is HTTP 400 with the unified error envelope and `code = "VALIDATION_FAILED"` explaining the missing header

---

## ADDED Requirements

### Requirement: Public Webhook Access With Signature Verification
The webhook ingest API SHALL support a public, signature-based endpoint at `POST /webhooks/{provider}/{tenant_id}` that:
- Bypasses operator bearer authentication when a valid provider signature is present.
- Preserves the existing operator-auth-only endpoint at `POST /webhooks/{provider}` for local/test and operational integrations.

Details:
- `tenant_id` path parameter:
  - MUST be a valid UUID that maps to a known tenant.
  - Provides tenant context for public webhook delivery.
- Authorization precedence and decision flow (normative order):
  1. If a valid operator bearer token is present:
     - The request SHALL be accepted (subject to normal provider/tenant validation); signature verification is OPTIONAL and SHALL NOT be required for success.
  2. Else, if the provider is supported AND the corresponding verification secret is configured AND the signature is present and valid:
     - The request SHALL be accepted (202).
  3. Else, if the provider is supported BUT:
     - The verification secret is missing; OR
     - The signature header(s) are missing; OR
     - The signature is malformed; OR
     - The signature verification fails:
     - The request SHALL be rejected with HTTP 401 and `code = "INVALID_SIGNATURE"` or `code = "UNAUTHORIZED"`, with details that do not leak secret material.
  4. Else, if the provider is not supported for webhook verification:
     - The request SHALL be rejected with HTTP 404 and `code = "NOT_FOUND"`.

- Provider secret requirements:
  - Public signature-based verification for a provider MUST ONLY be enabled when that provider’s secret is configured (see config spec).
  - When a provider secret is not configured:
    - Any request to `POST /webhooks/{provider}/{tenant_id}` without valid operator auth MUST be rejected with HTTP 401.
    - The implementation MUST NOT attempt verification with an empty or default secret.

- Error semantics:
  - Missing or malformed signature headers for supported providers without operator auth:
    - HTTP 401 with `code = "INVALID_SIGNATURE"` or `code = "UNAUTHORIZED"`.
  - Invalid signature:
    - HTTP 401 with `code = "INVALID_SIGNATURE"`.
  - Unsupported provider:
    - HTTP 404 with `code = "NOT_FOUND"`.

#### Scenario: Public signed request accepted
- GIVEN `POBLYSH_WEBHOOK_GITHUB_SECRET` is configured
- AND a valid GitHub signature is provided for the raw request body
- WHEN calling `POST /webhooks/github/{tenant_id}` without operator auth
- THEN the response is HTTP 202 and the webhook is accepted for processing

#### Scenario: Invalid signature rejected (no operator auth)
- GIVEN `POBLYSH_WEBHOOK_GITHUB_SECRET` is configured
- WHEN calling `POST /webhooks/github/{tenant_id}` with an invalid `X-Hub-Signature-256`
- AND without operator auth
- THEN the response is HTTP 401 with `code = "INVALID_SIGNATURE"` and the request is not processed

#### Scenario: Missing signature rejected (no operator auth)
- GIVEN `POBLYSH_WEBHOOK_GITHUB_SECRET` is configured
- WHEN calling `POST /webhooks/github/{tenant_id}` without `X-Hub-Signature-256`
- AND without operator auth
- THEN the response is HTTP 401 with `code = "INVALID_SIGNATURE"` and the request is not processed

#### Scenario: Public verification disabled when secret missing
- GIVEN `POBLYSH_WEBHOOK_GITHUB_SECRET` is NOT configured
- WHEN calling `POST /webhooks/github/{tenant_id}` without operator auth, regardless of headers
- THEN the response is HTTP 401 with `code = "UNAUTHORIZED"` and public verification remains disabled

#### Scenario: Unsupported provider returns 404
- GIVEN `foo` is not a supported webhook provider
- WHEN calling `POST /webhooks/foo/{tenant_id}` without operator auth
- THEN the response is HTTP 404 with `code = "NOT_FOUND"`

---

### Requirement: GitHub HMAC-SHA256 Verification
GitHub webhooks MUST be verified using HMAC-SHA256 with the configured secret over the exact raw request body bytes.

Details:
- Computation:
  - `digest = hex(hmac_sha256(github_secret, raw_body))`
  - Expected header: `X-Hub-Signature-256: sha256=<hex-digest>`
- Comparison:
  - The implementation MUST parse and validate the `sha256=` prefix.
  - The implementation MUST compare the expected signature and the received signature using a constant-time comparison function.
- Replay:
  - MVP SHALL NOT implement GitHub-specific replay prevention beyond HMAC verification.
  - The implementation SHOULD log `X-GitHub-Delivery` for potential future replay detection and diagnostics.

#### Scenario: Valid GitHub signature returns 202
- GIVEN `POBLYSH_WEBHOOK_GITHUB_SECRET` is configured
- AND `X-Hub-Signature-256` equals `sha256=` + hex(hmac_sha256(secret, raw_body))
- WHEN calling `POST /webhooks/github/{tenant_id}` without operator auth
- THEN the response is HTTP 202

#### Scenario: Missing GitHub signature returns 401
- GIVEN `POBLYSH_WEBHOOK_GITHUB_SECRET` is configured
- WHEN calling `POST /webhooks/github/{tenant_id}` without `X-Hub-Signature-256`
- THEN the response is HTTP 401 with `code = "INVALID_SIGNATURE"`

#### Scenario: Malformed GitHub signature returns 401
- GIVEN `POBLYSH_WEBHOOK_GITHUB_SECRET` is configured
- WHEN `X-Hub-Signature-256` does not start with `sha256=` or contains non-hex characters
- THEN the response is HTTP 401 with `code = "INVALID_SIGNATURE"`

---

### Requirement: Slack v2 Signature Verification
Slack webhooks MUST be verified using HMAC-SHA256 over the base string `v0:{timestamp}:{raw_body}` with the configured Slack signing secret, with mandatory timestamp tolerance to prevent replay.

Details:
- Headers:
  - `X-Slack-Signature: v0=<hex-digest>`
  - `X-Slack-Request-Timestamp: <unix-seconds>`
- Base string:
  - `basestring = "v0:" + timestamp + ":" + raw_body`
- Signature:
  - `digest = hex(hmac_sha256(slack_signing_secret, basestring))`
  - Compare `v0=<digest>` to `X-Slack-Signature`.
- Timestamp tolerance:
  - The server MUST reject requests where:
    - `X-Slack-Request-Timestamp` is missing, non-numeric, or
    - The absolute difference between server time and timestamp exceeds `POBLYSH_WEBHOOK_SLACK_TOLERANCE_SECONDS` (default 300).
- Comparison:
  - The implementation MUST use constant-time comparison for signature equality.

#### Scenario: Valid Slack signature within window returns 202
- GIVEN `POBLYSH_WEBHOOK_SLACK_SIGNING_SECRET` is configured
- AND `X-Slack-Request-Timestamp` is within the configured tolerance window
- AND `X-Slack-Signature` is a valid `v0=` HMAC-SHA256 signature over `v0:timestamp:raw_body`
- WHEN calling `POST /webhooks/slack/{tenant_id}` without operator auth
- THEN the response is HTTP 202

#### Scenario: Slack timestamp too old returns 401
- GIVEN `POBLYSH_WEBHOOK_SLACK_SIGNING_SECRET` is configured
- WHEN `X-Slack-Request-Timestamp` is older than the allowed tolerance window
- THEN the response is HTTP 401 with `code = "INVALID_SIGNATURE"` or `code = "UNAUTHORIZED"` and the request is not processed

#### Scenario: Missing or malformed Slack timestamp returns 401
- GIVEN `POBLYSH_WEBHOOK_SLACK_SIGNING_SECRET` is configured
- WHEN `X-Slack-Request-Timestamp` is missing or cannot be parsed as an integer
- THEN the response is HTTP 401 with `code = "INVALID_SIGNATURE"`

#### Scenario: Invalid Slack signature returns 401
- GIVEN `POBLYSH_WEBHOOK_SLACK_SIGNING_SECRET` is configured
- WHEN the computed `v0=` signature does not match `X-Slack-Signature` using constant-time comparison
- THEN the response is HTTP 401 with `code = "INVALID_SIGNATURE"`

---

### Requirement: OpenAPI Documentation (Public Webhooks)
The OpenAPI document for the connectors API SHALL accurately describe the webhook endpoints and authentication models.

Details:
- For `POST /webhooks/{provider}`:
  - MUST document operator bearer authentication and `X-Tenant-Id` requirements.
- For `POST /webhooks/{provider}/{tenant_id}`:
  - MUST:
    - Indicate that bearer authentication is NOT required when a valid provider signature is supplied.
    - Describe provider-specific signature headers (e.g. `X-Hub-Signature-256`, `X-Slack-Signature`, `X-Slack-Request-Timestamp`) as required/conditional parameters.
    - Clarify that missing/invalid signatures without operator auth result in HTTP 401 with problem+json.

#### Scenario: OpenAPI includes signature header schemas
- WHEN fetching `/openapi.json`
- THEN `POST /webhooks/{provider}/{tenant_id}`:
  - Documents provider-specific signature headers
  - Does not require bearer auth in its security schema for signed flows
  - References standardized problem+json error responses for 401/404/429

---

### Requirement: Webhook Verification Telemetry and Protections
The system SHALL emit structured telemetry and enforce rate limiting and abuse protections for public webhook verification.

Details:
- Rate limiting:
  - Public webhook endpoints (`/webhooks/{provider}/{tenant_id}`) MUST enforce:
    - A configurable global rate limit, and
    - A configurable per-IP (or equivalent) rate limit
    - BEFORE performing signature verification for unauthenticated requests where feasible.
  - Exceeded limits MUST return HTTP 429 with `code = "RATE_LIMITED"` in problem+json.
  - Implementations MAY delegate additional protection to upstream WAF/CDN but MUST document this in operations guidance.
- Logging:
  - The system MUST emit structured logs for verification attempts including:
    - Provider, tenant_id (or an internal tenant reference), outcome (success/failure), and high-level error reason.
  - Logs MUST NOT include secrets, raw HMAC keys, or full signatures; partial redaction MAY be used.
- Metrics:
  - The system MUST expose metrics suitable for dashboards and alerts, for example:
    - `signature_verification_success`
    - `signature_verification_failure`
    - `signature_verification_replay_reject`
    - Latency histograms for verification paths
  - Metrics labels MUST avoid unbounded cardinality (e.g. MUST NOT use raw IP addresses or arbitrary IDs as labels).
  - Provider identifiers MAY be used as labels if from a bounded set.

#### Scenario: Rate limit enforced before expensive verification
- GIVEN the configured rate limit for a single IP is exceeded on `/webhooks/{provider}/{tenant_id}`
- WHEN additional unauthenticated requests are sent from that IP
- THEN the system responds with HTTP 429 and `code = "RATE_LIMITED"` without performing full signature verification

#### Scenario: Verification attempts logged and metered
- WHEN processing webhook requests on `/webhooks/{provider}/{tenant_id}`
- THEN structured logs and metrics capture:
  - The provider, high-level outcome (success/failure/rate-limited), and redacted context
  - WITHOUT logging secrets or full signatures

---