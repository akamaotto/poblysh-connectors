connectors/openspec/changes/add-webhook-signature-verification/tasks.md
# add-webhook-signature-verification/tasks.md

## 0. Research & Validation Readiness

- [ ] 0.1 Execute the deep research workflow for this change:
  - [ ] 0.1.1 Review official GitHub webhook security docs for `X-Hub-Signature-256` and HMAC-SHA256 validation semantics.
  - [ ] 0.1.2 Review official Slack “Verifying requests from Slack” docs for `X-Slack-Signature`, `X-Slack-Request-Timestamp`, `v0:{ts}:{body}` base string, and 5-minute replay window.
  - [ ] 0.1.3 Review RustCrypto `hmac`, `sha2`, and `subtle` usage patterns to confirm correct and constant-time verification approaches.
  - [ ] 0.1.4 Review existing project configuration, logging, metrics, and routing patterns to align helpers and endpoints with current conventions.
  - [ ] 0.1.5 Capture concise implementation notes (header formats, algorithms, error semantics, rate limiting expectations, and telemetry patterns) and confirm alignment with the `proposal.md`, `design.md`, and spec deltas.

- [ ] 0.2 Cross-check the change set:
  - [ ] 0.2.1 Ensure `proposal.md` includes:
        - Clear Why/What/Impact.
        - Public webhook, signature verification, security, and observability scope.
  - [ ] 0.2.2 Ensure `design.md` documents:
        - Verification algorithms.
        - Route precedence rules.
        - Config and secrets usage.
        - Rate limiting and metrics strategy.
  - [ ] 0.2.3 Ensure all delta spec files under `specs/`:
        - Use `## ADDED|MODIFIED Requirements`.
        - Include at least one `#### Scenario:` per requirement.
        - Use consistent error code naming (SCREAMING_SNAKE_CASE).
        - Reflect the refined decision flow and missing-secret behavior.

## 1. Configuration & Secrets

- [ ] 1.1 Extend configuration to support webhook verification secrets:
  - [ ] 1.1.1 Add `POBLYSH_WEBHOOK_GITHUB_SECRET` (string, optional).
  - [ ] 1.1.2 Add `POBLYSH_WEBHOOK_SLACK_SIGNING_SECRET` (string, optional).
  - [ ] 1.1.3 Add `POBLYSH_WEBHOOK_SLACK_TOLERANCE_SECONDS` (int, default 300).
- [ ] 1.2 Behavior for missing secrets:
  - [ ] 1.2.1 If a provider’s secret is unset:
        - Public verification for that provider MUST be disabled.
        - Requests to `POST /webhooks/{provider}/{tenant_id}` without valid operator auth MUST be rejected with HTTP 401.
  - [ ] 1.2.2 Ensure `/webhooks/{provider}` (operator-auth path) remains fully functional and unaffected when secrets are absent.
- [ ] 1.3 Security handling:
  - [ ] 1.3.1 Ensure secrets are loaded via the existing configuration system and never logged.
  - [ ] 1.3.2 Document that production secrets MUST come from an approved secrets manager (AES-256 at rest, audited access).
  - [ ] 1.3.3 Document rotation guidance:
        - 30–90 day rotation cadence.
        - Support for dual-key grace period during rotation.
  - [ ] 1.3.4 Document a local, encrypted developer-only fallback:
        - MUST be clearly marked as non-production.
        - MUST NOT be used in production.

## 2. Webhook Verification Helpers

- [ ] 2.1 Implement GitHub verification helper:
  - [ ] 2.1.1 Signature:
        - Function: `verify_github_signature(secret: &str, raw_body: &[u8], header_value: &str) -> Result<bool, VerificationError>`.
  - [ ] 2.1.2 Logic:
        - Require header `X-Hub-Signature-256`.
        - Validate it starts with `sha256=`.
        - Compute `hex(hmac_sha256(secret, raw_body))`.
        - Compare computed vs provided digest using constant-time comparison.
        - On malformed header or mismatch, return a structured error (mapped to HTTP 401 with `INVALID_SIGNATURE`).
- [ ] 2.2 Implement Slack verification helper:
  - [ ] 2.2.1 Signature:
        - Function: `verify_slack_signature(secret: &str, tolerance_secs: i64, raw_body: &[u8], ts_header: &str, sig_header: &str) -> Result<bool, VerificationError>`.
  - [ ] 2.2.2 Logic:
        - Require `X-Slack-Signature` and `X-Slack-Request-Timestamp`.
        - Parse timestamp as integer seconds; if parse fails → treat as invalid.
        - Enforce tolerance: `abs(now - ts) <= tolerance_secs`; else reject as replay (distinct error).
        - Build base string: `v0:{timestamp}:{raw_body}`.
        - Compute `v0=` + hex(HMAC-SHA256(secret, base_string)).
        - Compare using constant-time comparison.
        - On missing/malformed headers, tolerance failure, or mismatch, return structured errors mapped to:
          - HTTP 401 with `INVALID_SIGNATURE` or `REPLAY_ATTACK_DETECTED` as appropriate.
- [ ] 2.3 Use RustCrypto + subtle correctly:
  - [ ] 2.3.1 Use existing `hmac`, `sha2`, and `subtle::ConstantTimeEq` crates.
  - [ ] 2.3.2 Ensure all comparisons are constant-time on equal-length byte slices.
  - [ ] 2.3.3 Ensure raw request body bytes are used without mutation (no pre-parse before HMAC).

## 3. Routing & Authorization Behavior

- [ ] 3.1 Public webhook route:
  - [ ] 3.1.1 Add `POST /webhooks/{provider}/{tenant_id}` route.
  - [ ] 3.1.2 Ensure this route:
        - Accepts requests without operator bearer auth only when a valid provider signature is present for a supported provider with a configured secret.
        - Uses `tenant_id` path parameter as tenant context.
- [ ] 3.2 Operator-auth route compatibility:
  - [ ] 3.2.1 Confirm existing `POST /webhooks/{provider}` route:
        - Remains operator-auth only (no behavior regression).
        - Still requires `Authorization: Bearer <token>` and `X-Tenant-Id` per auth spec.
- [ ] 3.3 Decision flow (normative precedence):
  - [ ] 3.3.1 Implement in this order for webhook requests:
        - a) If a valid operator bearer token is present:
             - Accept (subject to provider/handler success); do NOT require or depend on signatures.
        - b) Else, if `provider` is supported AND corresponding secret is configured AND signature is present and valid:
             - Accept with HTTP 202.
        - c) Else, if `provider` is unsupported:
             - Return HTTP 404 with appropriate `NOT_FOUND` problem code.
        - d) Else (provider supported but missing/malformed/invalid signature, or secret not configured):
             - Return HTTP 401 with appropriate problem code (e.g., `INVALID_SIGNATURE` or `UNAUTHORIZED`).
  - [ ] 3.3.2 Ensure this decision logic is reflected in:
        - Implementation.
        - `api-webhooks` and `auth` spec deltas.
        - Tests.

## 4. OpenAPI & Documentation

- [ ] 4.1 Update OpenAPI for webhook endpoints:
  - [ ] 4.1.1 Document `POST /webhooks/{provider}`:
        - Requires operator bearer auth and tenant header.
  - [ ] 4.1.2 Document `POST /webhooks/{provider}/{tenant_id}`:
        - Does NOT require bearer auth when a valid signature is provided.
        - Includes provider-specific signature headers:
          - GitHub: `X-Hub-Signature-256`.
          - Slack: `X-Slack-Signature`, `X-Slack-Request-Timestamp`.
- [ ] 4.2 Ensure security schemes:
  - [ ] 4.2.1 Do not attach bearer auth security requirement to the public signed path in OpenAPI.
  - [ ] 4.2.2 Clearly indicate that unsigned or invalidly signed requests are rejected with 401.
- [ ] 4.3 Operational documentation:
  - [ ] 4.3.1 Add or update runbooks / ops docs (in project docs or spec comments) to cover:
        - Secrets manager usage and rotation procedures.
        - Public webhook configuration steps for GitHub and Slack.
        - Recommended use of `/webhooks/{provider}/{tenant_id}` in provider settings.
        - Production vs local-secret constraints.

## 5. Telemetry, Metrics & Rate Limiting

- [ ] 5.1 Structured logging:
  - [ ] 5.1.1 Log each verification attempt with:
        - Provider slug.
        - Tenant identifier (where available).
        - Outcome: success, invalid signature, missing header, replay rejected, rate-limited.
        - Non-sensitive context (e.g., request id, `X-GitHub-Delivery`, high-level reason).
  - [ ] 5.1.2 Ensure:
        - No secrets or raw signatures are logged.
        - Logs follow existing `tracing` conventions.

- [ ] 5.2 Metrics:
  - [ ] 5.2.1 Add counters:
        - `signature_verification_success`
        - `signature_verification_failure`
        - `signature_verification_replay_reject`
  - [ ] 5.2.2 Add latency histogram:
        - e.g., `signature_verification_latency`.
  - [ ] 5.2.3 Labeling guidelines:
        - Use low-cardinality labels only (e.g., `provider`, `outcome`).
        - MUST NOT use unbounded-high-cardinality labels (raw IPs, arbitrary IDs).
        - Be cautious with `tenant_id` as a label; if included, ensure it aligns with monitoring standards and does not explode cardinality.

- [ ] 5.3 Rate limiting:
  - [ ] 5.3.1 Implement public endpoint rate limiting:
        - Apply per-IP and global rate limits before signature verification for unauthenticated requests.
  - [ ] 5.3.2 Behavior:
        - On limit exceeded, return HTTP 429 with a standard problem+json error (e.g., `RATE_LIMIT_EXCEEDED`).
  - [ ] 5.3.3 Implementation notes:
        - Use existing tower/axum-friendly patterns (e.g., middleware/layers).
        - Make thresholds configurable and document defaults.
  - [ ] 5.3.4 Document integration expectations:
        - Encourage upstream WAF/CDN protections for volumetric attacks.

## 6. Testing (OpenSpec & CodeRabbit Grade)

- [ ] 6.1 Unit tests: verification helpers
  - [ ] 6.1.1 GitHub:
        - Valid known vector: correct `X-Hub-Signature-256` → verification succeeds.
        - Invalid HMAC → returns failure mapped to 401/`INVALID_SIGNATURE`.
        - Missing header or wrong prefix → failure.
  - [ ] 6.1.2 Slack:
        - Valid signature with timestamp inside tolerance → success.
        - Timestamp too old/new → replay failure mapped to 401/`REPLAY_ATTACK_DETECTED` (or similar).
        - Non-integer or missing timestamp → invalid, 401.
        - Invalid signature value or missing `v0=` prefix → invalid, 401.

- [ ] 6.2 Integration tests: routing and auth precedence
  - [ ] 6.2.1 Public signed path:
        - With valid Slack or GitHub signature and configured secret:
          - `POST /webhooks/{provider}/{tenant_id}` → HTTP 202, no bearer auth required.
        - With invalid or missing signature and no operator auth:
          - For supported provider with secret configured → HTTP 401.
        - For provider with no secret configured:
          - Unsigned/signed requests without operator auth → HTTP 401.
  - [ ] 6.2.2 Operator-auth path:
        - `POST /webhooks/{provider}` with valid operator token and tenant header:
          - Still returns 202 as before.
        - Confirm that signature presence does not interfere with operator-auth success.
  - [ ] 6.2.3 Unsupported provider:
        - `POST /webhooks/unknown/{tenant_id}` → HTTP 404 with `NOT_FOUND`.
  - [ ] 6.2.4 Auth precedence:
        - Requests with valid operator token but invalid/missing signature:
          - MUST be accepted (202) based solely on operator auth.

- [ ] 6.3 Rate limiting & telemetry tests:
  - [ ] 6.3.1 Simulate exceeding per-IP/global rate limits:
        - Verify 429 responses and no signature verification performed after limit is hit.
  - [ ] 6.3.2 Verify logs:
        - Ensure outcomes are logged without secrets.
  - [ ] 6.3.3 Verify metrics:
        - Ensure success, failure, replay, and rate-limit counters/histograms increment as expected.

- [ ] 6.4 Spec compliance checks:
  - [ ] 6.4.1 Confirm all new/modified requirements have at least one scenario.
  - [ ] 6.4.2 Confirm all problem codes are SCREAMING_SNAKE_CASE.
  - [ ] 6.4.3 Ensure examples and behaviors in tests match OpenSpec delta requirements exactly.

## 7. OpenSpec Strict Validation

- [ ] 7.1 Run strict validation for this change:
  - [ ] 7.1.1 Execute: `openspec validate add-webhook-signature-verification --strict`.
- [ ] 7.2 Resolve all reported issues:
  - [ ] 7.2.1 If any structural/spec-format errors are reported:
        - Fix headings, requirement sections, or scenarios until validation passes.
  - [ ] 7.2.2 If any missing deltas or mismatched requirement names are reported:
        - Align delta specs with base specs as per OpenSpec rules.
  - [ ] 7.2.3 Re-run `openspec validate add-webhook-signature-verification --strict` until it returns success.

## 8. Final Review

- [ ] 8.1 Ensure no behavior regressions for existing endpoints.
- [ ] 8.2 Ensure implementation, tests, and docs:
      - Match `proposal.md`, `design.md`, and all spec deltas.
      - Follow coding standards suitable for senior-level and CodeRabbit-quality review.
- [ ] 8.3 Mark all tasks complete only when:
      - Implementation is in place.
      - Tests are green.
      - `openspec validate --strict` passes for this change.