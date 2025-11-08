## Why

The webhook ingest endpoint currently requires operator authentication for MVP. To support real provider callbacks from GitHub and Slack safely, we must:

- Expose a public, tenant-aware webhook endpoint.
- Authenticate requests using provider-specific signatures.
- Enforce replay protection (where supported), rate limiting, and strong observability.
- Preserve backwards-compatible operator-auth behavior for local/test and migrations.

This change introduces standards-aligned webhook signature verification and public access while maintaining a conservative, auditable, and operations-friendly security posture.

## What Changes

### 1. Public Webhook Endpoints With Signature Verification

- Introduce a public, tenant-aware webhook path:

  - `POST /webhooks/{provider}/{tenant_id}`

- Behavior:
  - Requests to this path MAY omit operator bearer auth if they include a valid provider signature as defined in the webhooks spec.
  - When a valid operator bearer token is present:
    - The request SHALL be accepted (subject to normal processing) regardless of signature presence.
    - Signature evaluation is skipped for precedence and simplicity.
  - When no valid operator token is present:
    - A supported provider with verification enabled MUST present a valid signature or receive HTTP 401.
    - An unsupported provider MUST receive HTTP 404.
    - A supported provider without verification enabled (missing secret) MUST reject unsigned/invalid requests with HTTP 401.

- Backwards compatibility:
  - The existing operator-auth-only path:
    - `POST /webhooks/{provider}`
  - SHALL remain supported and continue to require:
    - Valid operator `Authorization: Bearer <token>`
    - `X-Tenant-Id` header.
  - This path is recommended for local/test scenarios or internal/operator-triggered deliveries.

### 2. Provider-Specific Verification Rules

The implementation MUST follow official provider documentation for signature computation and verification.

#### GitHub (HMAC-SHA256)

- Verification is based on:
  - Header: `X-Hub-Signature-256`
    - Value format: `sha256=<hex-digest>`
  - Algorithm:
    - `hex_digest = hex(hmac_sha256(GITHUB_SECRET, raw_body_bytes))`
    - Compare `sha256=<hex_digest>` against the header value using constant-time comparison.
- Requirements:
  - Use the raw, unmodified request body bytes as received.
  - Use constant-time comparison for the entire expected-vs-actual signature string.
  - If header is missing, malformed, or does not match:
    - Respond with HTTP 401 and a problem+json error (see Error Semantics below).
- Replay:
  - No timestamp is provided by GitHub.
  - MVP SHALL NOT implement replay de-duplication.
  - The system SHOULD log `X-GitHub-Delivery` to enable future replay detection and investigations.

#### Slack v2 (Signed Secrets)

- Verification is based on:
  - Headers:
    - `X-Slack-Signature` (or case-insensitive equivalent)
      - Value format: `v0=<hex-digest>`
    - `X-Slack-Request-Timestamp`
  - Algorithm:
    - Construct base string: `v0:{timestamp}:{raw_body}`
    - Compute `hex_digest = hex(hmac_sha256(SLACK_SIGNING_SECRET, base_string))`
    - Expected signature: `v0=<hex_digest>`
- Requirements:
  - Use raw request body bytes (no pre-processing).
  - Enforce a configurable timestamp tolerance window:
    - Default: 300 seconds.
    - Value sourced from `POBLYSH_WEBHOOK_SLACK_TOLERANCE_SECONDS`.
  - Validation steps:
    1. Reject if timestamp header is missing or not a valid integer → HTTP 401.
    2. Reject if `abs(now - timestamp) > tolerance` → HTTP 401 (replay protection).
    3. Compute expected signature and compare to header using constant-time comparison.
    4. Reject on any mismatch → HTTP 401.
- This behavior MUST align with Slack’s “Verifying requests from Slack” guidance.

### 3. Auth Precedence, Provider Handling, and Routing

The following decision flow MUST be implemented for webhook requests:

1. Evaluate operator authentication:
   - If a valid operator bearer token is present (per existing auth spec):
     - Request SHALL be accepted (202 on successful processing), regardless of signature presence or validity.
     - Signature verification MAY be skipped in this case.
2. If operator auth is missing or invalid:
   - Determine provider by `{provider}` segment:
     - If provider is not recognized:
       - Respond with HTTP 404 (`NOT_FOUND`).
     - If provider is recognized:
       - Check if provider verification is enabled (secret configured).
3. For supported providers with verification enabled:
   - If required signature headers are missing, malformed, or invalid:
     - Respond with HTTP 401 (`INVALID_SIGNATURE` or equivalent).
   - If signature is valid:
     - Respond with HTTP 202 on successful processing.
4. For supported providers without verification enabled (secret missing/unset):
   - Public verification MUST be considered disabled.
   - Requests without valid operator auth MUST be rejected with HTTP 401.
   - The operator-auth-only endpoint `/webhooks/{provider}` remains usable with valid bearer auth.

This flow MUST be consistently applied for both the operator path and the public tenant-aware path, with the caveat that the operator path is only intended for authenticated use.

### 4. Configuration and Secrets Management

Add configuration for provider verification secrets:

- `POBLYSH_WEBHOOK_GITHUB_SECRET` (string, optional)
- `POBLYSH_WEBHOOK_SLACK_SIGNING_SECRET` (string, optional)
- `POBLYSH_WEBHOOK_SLACK_TOLERANCE_SECONDS` (integer, default 300)

Behavior:

- If a provider’s secret is set:
  - Public signature verification for that provider MUST be enabled.
  - Unsigned or invalidly signed public requests for that provider MUST be rejected with HTTP 401.
- If a provider’s secret is not set:
  - Public (unauthenticated) requests for that provider MUST be rejected with HTTP 401.
  - Only properly operator-authenticated requests are allowed.
- Secrets MUST:
  - Be treated as sensitive.
  - Never be logged or exposed in responses.
  - Be sourced from an approved centralized secrets manager in production:
    - AES-256 encryption at rest.
    - Audited access.
    - Per-environment isolation.
- Operational guidance MUST:
  - Recommend a 30–90 day rotation cadence.
  - Support dual-key (overlapping) rotation windows.
  - Provide a clearly documented, encrypted local development-only fallback:
    - MUST be explicitly marked as non-production and prohibited in production environments.

### 5. Rate Limiting and Abuse Protections

To protect public endpoints from abuse and reduce unnecessary signature computations:

- The system SHALL enforce:
  - Configurable global rate limiting on the public webhook path(s).
  - Configurable per-IP (or equivalent source) rate limiting where feasible.
- Rate limit evaluation MUST occur before expensive signature verification, when practical.
- On rate limit exceed:
  - Respond with HTTP 429 using the standard problem+json envelope.
- Integration expectations:
  - Document how upstream WAF/CDN solutions SHOULD be used:
    - IP reputation.
    - Request scoring.
    - Geo/IP-based controls.
    - DDoS mitigation.
- Metrics and logs SHOULD:
  - Avoid unbounded cardinality labels (e.g., do not use raw IP addresses or arbitrary high-cardinality IDs as labels).
  - Use bounded dimensions, such as:
    - provider (enum-like)
    - outcome (success/failure/rate_limited/replay_reject)
    - error category.

### 6. Telemetry, Logging, and Metrics

The system SHALL provide comprehensive, privacy- and security-conscious observability for webhook verification:

- Logging:
  - Structured logs for each verification attempt:
    - Fields SHOULD include:
      - `provider`
      - `tenant_id` or a stable, bounded tenant identifier
      - outcome (success/failure/replay_reject/rate_limited)
      - reason code/category (e.g., missing_header, bad_format, invalid_signature)
      - request or trace ID (for correlation)
    - MUST NOT include secrets, raw signatures, or sensitive payload content.
    - MAY include safe identifiers like `X-GitHub-Delivery` in redacted or hashed form for GitHub.
- Metrics:
  - Counters and histograms SHOULD include (names indicative, final names in spec):
    - `signature_verification_success`
    - `signature_verification_failure`
    - `signature_verification_replay_reject`
    - `signature_verification_rate_limited`
    - Latency histogram (e.g., `signature_verification_latency_seconds`)
  - Metrics MUST respect cardinality constraints:
    - Use small, bounded label sets (e.g., provider, outcome).
- Alerting and retention:
  - Documentation MUST recommend:
    - 30–90 day log retention aligned with compliance needs.
    - Alerts on:
      - Spikes in signature failures.
      - Spikes in replay rejections.
      - Unusual rate-limit activity suggesting DDoS or misconfiguration.

### 7. Error Semantics (Problem+JSON)

All error responses for webhook verification and access control MUST use the unified `application/problem+json` envelope defined in the API core spec.

- Recommended codes (screaming snake case):
  - `NOT_FOUND`:
    - Unknown provider.
  - `INVALID_SIGNATURE`:
    - Missing, malformed, or mismatched signature headers.
  - `UNAUTHORIZED`:
    - Missing required auth (operator token or valid signature) when needed.
  - `RATE_LIMIT_EXCEEDED`:
    - Requests rejected by rate limiting.
- The proposal does not rigidly define all codes but requires:
  - Consistent use of screaming snake case.
  - Clear mapping from each failure path.

### 8. OpenAPI / Documentation Requirements

The OpenAPI specification MUST:

- Document both webhook endpoints:
  - `POST /webhooks/{provider}`
  - `POST /webhooks/{provider}/{tenant_id}`
- For `POST /webhooks/{provider}`:
  - Require operator bearer auth as currently defined.
- For `POST /webhooks/{provider}/{tenant_id}`:
  - Document that:
    - Valid operator bearer auth is accepted (and takes precedence).
    - Otherwise, public access is allowed only with valid provider signature.
    - Provider-specific signature headers are required/conditional.
- Include:
  - Provider-specific header parameter definitions:
    - GitHub: `X-Hub-Signature-256`
    - Slack: `X-Slack-Signature`, `X-Slack-Request-Timestamp`
  - Clear descriptions that:
    - Requests without valid signatures and without valid auth will be rejected with 401.

## Impact

- Specs affected:
  - `api-webhooks`:
    - Public tenant-aware endpoint.
    - Provider-specific verification requirements.
    - Signature-based auth rules and precedence.
  - `auth`:
    - Clarification of public webhook bypass behavior for valid signatures.
    - Preservation of existing public endpoints (`/healthz`, `/readyz`, `/docs`, `/openapi.json`).
  - `config`:
    - New webhook verification secrets and tolerance configuration.
- Code impact (high level):
  - Router:
    - Add `POST /webhooks/{provider}/{tenant_id}`.
    - Centralize provider dispatch and auth/signature decision flow.
  - Handlers:
    - Ensure access to raw body bytes for HMAC.
  - Verification utilities:
    - Implement `verify_github_signature` and `verify_slack_signature` with constant-time comparison.
  - Middleware/layers:
    - Rate limiting before expensive verification.
    - Logging/metrics instrumentation.
  - Documentation:
    - Operational runbook for secrets management, rotation, and monitoring.

## Acceptance Criteria

To be considered complete and compliant (including strict OpenSpec validation and high-quality review):

1. Public webhook support:
   - `POST /webhooks/{provider}/{tenant_id}` implemented.
   - Correct precedence of operator auth vs signature verification.
   - Unsupported providers return HTTP 404 with `NOT_FOUND`.

2. GitHub verification:
   - Uses `X-Hub-Signature-256` with `sha256=<hex>` format.
   - HMAC-SHA256 using configured `POBLYSH_WEBHOOK_GITHUB_SECRET`.
   - Constant-time comparison implemented.
   - Invalid or missing signature → HTTP 401 with problem+json (`INVALID_SIGNATURE` or `UNAUTHORIZED`), metrics/logs updated.

3. Slack verification:
   - Uses `X-Slack-Signature` and `X-Slack-Request-Timestamp`.
   - Constructs `v0:{timestamp}:{raw_body}` and validates with HMAC-SHA256.
   - Enforces `POBLYSH_WEBHOOK_SLACK_TOLERANCE_SECONDS` (default 300s).
   - Missing/malformed timestamp or signature, or outside tolerance → HTTP 401 with problem+json, metrics/logs updated.

4. Config and secrets:
   - New env vars integrated into config system.
   - Behavior matches rules:
     - Secret present → verification enabled.
     - Secret missing → public verification disabled; public requests rejected with 401.
   - Documentation and runbook clearly define secure secret management and rotation.

5. Rate limiting:
   - Public webhook path(s) protected by:
     - Configurable global limiter, and
     - Per-IP or equivalent limiter where feasible.
   - Rate limiting applied before expensive verification.
   - Exceeded limits return HTTP 429 with `RATE_LIMIT_EXCEEDED` and appropriate telemetry.

6. Telemetry:
   - Structured logs and metrics implemented per requirements.
   - No sensitive data or raw secrets in logs.
   - Metric labels bounded to avoid unbounded cardinality.

7. OpenAPI:
   - Endpoints and headers documented.
   - Auth and signature expectations accurately reflected.
   - No contradictions between OpenAPI and this proposal/spec.

8. Tests (OpenSpec-aligned and quality-reviewed):
   - Unit tests:
     - Known test vectors for GitHub HMAC and Slack signatures.
     - Edge cases: missing headers, malformed values, tolerance boundaries, disabled secrets.
   - Integration tests:
     - Valid GitHub/Slack signatures → 202.
     - Invalid/missing signatures → 401.
     - Missing Slack timestamp or out-of-window → 401.
     - Missing provider secret → 401 for unsigned public calls; operator-auth path remains functional.
     - Unknown provider → 404.
     - Rate limit scenarios → 429 with proper envelope.
   - All tests SHALL be deterministic, isolated, and adhere to existing testing conventions.

9. OpenSpec validation:
   - The change directory (`add-webhook-signature-verification`) SHALL pass:
     - Strict OpenSpec validation with no structural or formatting errors.
   - All specs under this change MUST:
     - Use correct `## ADDED|MODIFIED Requirements` headers.
     - Provide at least one `#### Scenario:` per requirement.
     - Use screaming snake case for error codes where specified.
