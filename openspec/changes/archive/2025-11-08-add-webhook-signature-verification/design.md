connectors/openspec/changes/add-webhook-signature-verification/design.md
## Context

The MVP webhook ingest endpoint (`POST /webhooks/{provider}`) is currently operator-authenticated and not suitable for public provider callbacks.

This change introduces secure public webhook ingestion with provider-specific signature verification for GitHub and Slack. It must:

- Allow providers to call a public endpoint without operator credentials while preserving strong authentication.
- Cleanly coexist with the existing operator-auth path for local, test, or internal use.
- Provide clear, testable behavior for:
  - Authorization precedence between operator auth and signatures.
  - Signature verification algorithms and header formats.
  - Missing/misconfigured secrets.
  - Replay protection (Slack) and its explicit limitations (GitHub).
  - Rate limiting, abuse protection, and metrics with safe cardinality.
  - OpenAPI documentation and error semantics.

The design aligns with OpenSpec conventions and is structured to pass strict validation and enterprise code review standards.

## Goals / Non-Goals

### Goals

- Add a public tenant-aware webhook path:
  - `POST /webhooks/{provider}/{tenant_id}`.
- Implement provider-specific signature verification:
  - GitHub: HMAC-SHA256 over raw body with `X-Hub-Signature-256` (prefix `sha256=`).
  - Slack: HMAC-SHA256 over `v0:{timestamp}:{raw_body}` with `X-Slack-Signature` (prefix `v0=`) and `X-Slack-Request-Timestamp`.
- Define clear authorization and verification precedence:
  - Operator token takes precedence; valid signature is an alternative when operator token is absent/invalid.
- Enforce behavior when secrets are missing or invalid:
  - Public verification disabled for that provider; unsigned or unverifiable requests rejected.
- Ensure correct handling of raw request bodies:
  - Use unmodified bytes for HMAC computation.
- Integrate rate limiting and basic abuse protections on public endpoints.
- Provide structured telemetry:
  - Logs and metrics for verification outcomes, replay rejections, and rate limiting decisions.
  - Respect bounded metric cardinality.
- Update OpenAPI to fully describe public webhook behavior and headers.
- Maintain backward compatibility with the operator-auth endpoint.

### Non-Goals

- Introducing per-tenant or per-connection secrets (MVP uses provider-scoped secrets).
- Implementing GitHub replay attack storage/deduplication (only Slack gets timestamp-based replay protection).
- Adding new external storage or database migrations.
- Supporting additional providers beyond GitHub and Slack in this change.
- Implementing MTLS for webhooks (may be considered in future changes).

## Dependencies & Technology Choices

We use existing, proven dependencies to keep the implementation simple and robust.

- Web framework:
  - `axum = "0.8.6"`
  - `tokio = "1.48.0"`
  - `tower = "0.5.1"`
  - `tower-http = "0.6.2"` (for middleware and potential rate limiting helpers)
- Crypto and security:
  - `hmac = "0.12.1"` (RustCrypto)
  - `sha2 = "0.10.8"` (RustCrypto)
  - `subtle = "2.6.1"` (for constant-time equality)
  - `hex = "0.4.3"` (for hex decoding/encoding)
- Observability:
  - `tracing`, `tracing-subscriber`, `tracing-log`
  - `metrics = "0.23.0"`
- Config:
  - Existing configuration system and environment layering.
  - New variables:
    - `POBLYSH_WEBHOOK_GITHUB_SECRET` (string, optional)
    - `POBLYSH_WEBHOOK_SLACK_SIGNING_SECRET` (string, optional)
    - `POBLYSH_WEBHOOK_SLACK_TOLERANCE_SECONDS` (int, default 300)

No additional crypto crates are required. Rate limiting can initially rely on `tower`/`tower-http` and simple in-memory strategies; any more advanced or distributed rate limiting will be specified in future changes.

## Authorization & Verification Precedence

All webhook requests follow a deterministic decision order to avoid ambiguity.

### Endpoints

- Operator-auth endpoint (existing behavior preserved):
  - `POST /webhooks/{provider}`
- Public tenant-aware endpoint:
  - `POST /webhooks/{provider}/{tenant_id}`

### Decision Flow (Normalized)

For both endpoints, apply the following logic:

1. Validate provider:
   - If `{provider}` is not recognized:
     - Return HTTP 404 with `application/problem+json`:
       - `code: "NOT_FOUND"`

2. Check operator bearer token:
   - If a valid operator token is present:
     - Accept the request (subject to other request-level validations, e.g., basic syntax).
     - Signature verification is NOT required and MAY be skipped.
     - Return or proceed as `202 Accepted` as per webhook ingest semantics.
   - If operator token is present but invalid:
     - Treat as absent for the purposes of signature fallback.
     - Do NOT leak whether the token was “almost” valid; log internally.

3. Signature verification path (only when no valid operator token):

   - If provider is supported for signatures (GitHub, Slack in this change):
     - If the corresponding provider secret is NOT configured:
       - Public verification is effectively disabled for that provider.
       - Any unauthenticated request MUST be rejected:
         - HTTP 401 with `application/problem+json`:
           - `code: "INVALID_SIGNATURE"` or equivalent auth error as defined in the spec.
     - If the secret is configured:
       - Perform provider-specific verification (see below).
       - If verification succeeds:
         - Accept with HTTP 202.
       - If verification fails (missing headers, malformed, mismatch, timestamp invalid, etc.):
         - Reject with HTTP 401:
           - `code: "INVALID_SIGNATURE"`; do not leak sensitive detail.
   - If provider is not supported for signatures:
     - Public unsigned access is not allowed.
     - Without valid operator auth:
       - Reject with HTTP 401 or 404, consistent with spec (prefer 404 for unknown provider, 401 for known provider lacking supported auth).

4. Rate limiting and abuse controls:
   - For unauthenticated/public requests:
     - Apply IP/global rate limiting BEFORE expensive HMAC computation where feasible.
     - Over-limit requests:
       - Return HTTP 429 with proper `application/problem+json`.
   - Operator-authenticated requests MAY have separate or higher limits.

This ordering ensures:

- Backward compatibility: operator-auth behavior is unaffected.
- Minimal confusion: signature is a capability used only when operator auth is not valid.
- Clear semantics for missing secrets.

## Provider-Specific Verification

### Common Principles

- Always compute HMAC over the raw request body bytes as received.
- Never deserialize or modify the body before computing the HMAC.
- Use constant-time comparison for all signature checks.
- Treat missing or malformed headers as verification failures.
- Do not log secrets or raw signatures; log only safe metadata and coarse-grained reasons.

### GitHub Verification

- Header:
  - `X-Hub-Signature-256: sha256=<hex-digest>`
- Secret:
  - `POBLYSH_WEBHOOK_GITHUB_SECRET`
- Algorithm:
  - `digest = hex(hmac_sha256(secret, raw_body))`
  - Expected header format: `sha256=` + `digest`
- Steps:
  1. Read raw body bytes once.
  2. Compute HMAC-SHA256 with configured secret.
  3. Construct expected string with `sha256=` prefix.
  4. Compare expected vs provided using constant-time equality:
     - Parse/validate format, but ensure final comparison is fixed-time on validated inputs.
- Failure cases:
  - Secret not set:
    - Public verification disabled → 401 for unsigned/unsigned-only requests.
  - Header missing, malformed, or prefix not `sha256=`:
    - 401 (INVALID_SIGNATURE).
- Replay:
  - No timestamp header is mandated by GitHub.
  - MVP:
    - No server-side replay store; document this explicitly.
    - `X-GitHub-Delivery` MAY be logged for future replay/dedup strategies.

### Slack Verification (v2)

- Headers:
  - `X-Slack-Signature: v0=<hex-digest>`
  - `X-Slack-Request-Timestamp: <unix-seconds>`
- Secret:
  - `POBLYSH_WEBHOOK_SLACK_SIGNING_SECRET`
- Tolerance:
  - `POBLYSH_WEBHOOK_SLACK_TOLERANCE_SECONDS` (default 300)
- Algorithm:
  1. Extract timestamp; must be a valid integer.
  2. Validate timestamp freshness:
     - `abs(now - timestamp) <= tolerance_seconds`.
     - If outside this window:
       - Reject as replay: 401; increment `signature_verification_replay_reject`.
  3. Construct base string:
     - `v0:{timestamp}:{raw_body}`
  4. Compute HMAC-SHA256 over base string with signing secret.
  5. Expected signature: `v0=` + hex digest.
  6. Compare to `X-Slack-Signature` using constant-time equality.
- Failure cases (any of):
  - Secret missing.
  - Missing timestamp.
  - Timestamp not an integer.
  - Timestamp too old or too far in the future.
  - Missing/malformed `X-Slack-Signature` or wrong prefix.
  - Signature mismatch.
- All failures:
  - 401 with `INVALID_SIGNATURE`.

## Configuration & Secrets Management

- Provider secrets:
  - MUST be loaded via the existing config system.
  - MUST be treated as sensitive:
    - Never logged.
    - Redacted in errors and traces.
- Missing secrets:
  - For GitHub or Slack:
    - Public signature-based auth is disabled.
    - Requests without valid operator auth MUST be rejected with 401.
- Operational guidance (enforced in docs/tasks):
  - Production secrets:
    - Sourced from an approved secrets manager.
    - AES-256 encryption at rest.
    - Audited access.
    - 30–90 day rotation cadence, with dual-key/grace-period support recommended.
  - Local development:
    - MAY use an encrypted local file or env vars.
    - MUST be clearly documented as non-production only.

## Rate Limiting & Abuse Protections

### Strategy

- Public webhook endpoints are high-risk; mitigate abuse with:
  - Pre-verification rate limiting:
    - Apply inexpensive checks before HMAC computations where possible.
  - Post-verification telemetry:
    - Use metrics and logs to detect anomalies.

### Implementation Notes

- For this change:
  - Implement:
    - A global rate limit for public webhook routes.
    - A best-effort per-IP rate limit using in-memory tracking.
  - Defaults:
    - Document example defaults in specs (e.g., N requests/sec per IP).
    - Actual numbers configurable via env/config.
- Behavior:
  - On limit exceed:
    - Return HTTP 429 with `application/problem+json` and suitable `code` (e.g., `RATE_LIMIT_EXCEEDED`).
- Metrics cardinality:
  - Metrics MUST avoid unbounded label sets.
  - Recommended labels:
    - `provider` (bounded: github, slack, etc).
    - `outcome` (e.g., success, invalid_signature, replay_reject, rate_limited).
  - Do NOT include:
    - Raw IPs.
    - Raw tenant IDs as open-ended labels.
    - Raw request IDs.
  - If tenant dimension is ever needed:
    - Restrict to coarse or sampled usage, or capture in logs rather than as a metric label.

## Telemetry & Logging

### Logging

- For each verification attempt, log a structured event that includes:
  - `provider`
  - `tenant_id` (if path-based; ensure PII policies are respected)
  - `outcome`:
    - `success`
    - `invalid_signature`
    - `missing_secret`
    - `replay_reject`
    - `rate_limited`
  - High-level failure reason category (no secrets, bad header, old timestamp).
- Redaction:
  - NEVER log:
    - Actual secrets.
    - Full signatures.
    - Raw request bodies.
  - MAY log:
    - Truncated hashes or anonymized request IDs if needed for debugging.

### Metrics

- Counters (examples):
  - `signature_verification_success_total{provider, outcome="success"}`
  - `signature_verification_failure_total{provider, outcome="invalid_signature"}` 
  - `signature_verification_replay_reject_total{provider, outcome="replay_reject"}`
  - `webhook_rate_limited_total{provider}`
- Histograms:
  - `signature_verification_latency_seconds{provider}`
- Ensure all names and labels are:
  - Stable.
  - Bounded.
  - Documented in the spec and implementation.

## OpenAPI Documentation

The design requires OpenAPI to clearly represent:

- `POST /webhooks/{provider}`:
  - Secured by operator bearer auth.
  - Document existing headers and behavior.

- `POST /webhooks/{provider}/{tenant_id}`:
  - No bearer auth required when a valid signature is present.
  - Must document:
    - Provider-specific headers:
      - GitHub:
        - `X-Hub-Signature-256` (required for public access when enabled).
      - Slack:
        - `X-Slack-Signature`
        - `X-Slack-Request-Timestamp`
    - Behavior when headers or secrets are missing:
      - 401 with `INVALID_SIGNATURE` (or equivalent defined code).
    - Successful response:
      - 202 with `{ "status": "accepted" }`.

The security schemes and operation objects MUST reflect that public access is allowed with signatures, not with bearer auth, for the tenant-aware endpoint.

## Decision Summary

- Keep operator-authenticated `/webhooks/{provider}` for backwards compatibility.
- Introduce `/webhooks/{provider}/{tenant_id}` for public, signature-verified callbacks.
- Define a strict, documented precedence:
  - Valid operator token → accept (no signature required).
  - Else, verify signature if provider-secret configured:
    - Valid signature → accept.
    - Invalid/missing → 401.
  - Missing secret for supported provider → 401 for unsigned/signed attempts without operator auth.
- Use existing RustCrypto (`hmac`, `sha2`) and `subtle` for HMAC and constant-time comparison.
- Use raw body bytes consistently for HMAC input.
- Enforce:
  - Slack timestamp tolerance.
  - GitHub signature format.
- Implement:
  - Config-driven secrets with strong operational guidance.
  - Pre-verification rate limiting and structured telemetry.
  - Metrics with bounded cardinality.
  - OpenAPI updates aligned with this behavior.
- Explicitly document limitations:
  - No GitHub replay store (MVP).
  - No per-tenant secrets (MVP).
  - No new persistent storage introduced.

## Testing Strategy (OpenSpec & Code Quality Alignment)

The change MUST include tests that:

1. Verify precedence:
   - Operator token + any body/no signature → 202.
   - No operator token + valid GitHub/Slack signature → 202.
   - No operator token + invalid/missing signature → 401.
   - Missing provider secret:
     - No operator token → 401 even if signature headers are present.

2. Provider correctness:
   - GitHub:
     - Valid example vector yields correct `X-Hub-Signature-256` and passes.
     - Missing `sha256=` prefix or malformed header → 401.
   - Slack:
     - Correct base string construction and signature → 202.
     - Timestamp too old/new → 401 (replay_reject counter incremented).
     - Missing/invalid headers → 401.

3. Rate limiting:
   - Surging requests to public endpoints:
     - Confirm 429 is returned when limits are breached.
     - Metrics/logs reflect rate limiting events.

4. Telemetry:
   - Ensure no sensitive values in logs.
   - Ensure metrics use only bounded labels.

5. OpenSpec validation:
   - Specs under this change use:
     - Proper `## ADDED|MODIFIED Requirements` sections.
     - `#### Scenario:` formatting.
     - Screaming snake case for any new problem codes (e.g., `INVALID_SIGNATURE`, `RATE_LIMIT_EXCEEDED`) where defined.

This design text is crafted to be consistent with the proposal, tasks, and specs, resolving all identified gaps from the review and supporting a clean run of strict OpenSpec validation for the `add-webhook-signature-verification` change.