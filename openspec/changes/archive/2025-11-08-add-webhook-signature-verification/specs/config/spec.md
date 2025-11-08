connectors/openspec/changes/add-webhook-signature-verification/specs/config/spec.md
#L1-200
## ADDED Requirements

### Requirement: Webhook Verification Secrets
The system SHALL use provider-specific secrets from configuration to verify webhook signatures for public webhook endpoints.

Variables (MVP):
- `POBLYSH_WEBHOOK_GITHUB_SECRET` (string, optional)
- `POBLYSH_WEBHOOK_SLACK_SIGNING_SECRET` (string, optional)
- `POBLYSH_WEBHOOK_SLACK_TOLERANCE_SECONDS` (int, optional, default `300`)

Behavior:
- If `POBLYSH_WEBHOOK_GITHUB_SECRET` is set:
  - GitHub public webhook verification for `POST /webhooks/github/{tenant_id}` MUST be enabled.
- If `POBLYSH_WEBHOOK_SLACK_SIGNING_SECRET` is set:
  - Slack public webhook verification for `POST /webhooks/slack/{tenant_id}` MUST be enabled using the configured tolerance.
- If a provider-specific secret is not set:
  - Public verification for that provider MUST be disabled.
  - Any unauthenticated request for that provider to `POST /webhooks/{provider}/{tenant_id}` MUST be rejected with HTTP 401 using the unified `application/problem+json` error envelope.
- Secrets MUST be treated as sensitive values:
  - Secrets MUST NEVER be logged.
  - Secrets MUST NOT appear in metrics, traces, panic messages, or error details.
  - Only derived values (e.g., truncated key identifiers explicitly documented in runbooks) MAY be logged for operational debugging and MUST NOT allow reconstruction of the secret.

#### Scenario: Missing provider secret disables public verification
- GIVEN `POBLYSH_WEBHOOK_GITHUB_SECRET` is unset
- WHEN a GitHub webhook request is sent to `POST /webhooks/github/{tenant_id}` without operator authentication
- THEN the request is rejected with HTTP 401 using the unified problem+json envelope

#### Scenario: Slack tolerance can be configured
- GIVEN `POBLYSH_WEBHOOK_SLACK_TOLERANCE_SECONDS=120`
- WHEN a Slack webhook arrives with `X-Slack-Request-Timestamp` equal to 120 seconds in the past and all signatures are valid
- THEN the request is accepted

#### Scenario: Secrets are never logged
- WHEN webhook verification is enabled and logs are emitted for verification attempts
- THEN no log line contains the full value of any `POBLYSH_WEBHOOK_*` secret

### Requirement: Secret Source, Rotation, and Environment Isolation
Production webhook verification secrets MUST be sourced from an approved centralized secrets management solution.

Details:
- Secrets management:
  - Production and staging webhook secrets MUST be stored in a centralized secrets manager that provides:
    - AES-256 (or stronger) encryption at rest
    - Strict access control and audit logging
    - Versioning and rotation support
- Rotation:
  - There MUST be documented operational guidance describing:
    - A rotation cadence of 30–90 days for each webhook secret.
    - A dual-key or grace-period strategy (e.g., accept new + old secret for a bounded window) to enable safe rotation without downtime.
  - The implementation MUST support loading updated secrets without requiring code changes (e.g., via environment reload or redeploy with new secret values).
- Environment isolation:
  - Each environment (local, dev, staging, production) MUST use distinct secrets.
  - It is PROHIBITED to reuse production secrets in non-production environments.

#### Scenario: Secrets manager operational guidance documented
- WHEN operators consult the webhook verification operations runbook
- THEN it specifies the use of the centralized secrets manager, the rotation cadence, dual-key/grace expectations, audit requirements, and environment isolation

#### Scenario: Rotation without downtime is supported
- GIVEN a new secret is provisioned for GitHub webhooks
- WHEN the deployment is updated according to the documented rotation procedure
- THEN GitHub webhook validation continues to succeed throughout the rotation window without requiring code changes

### Requirement: Local Development Fallback (Non-Production Only)
The system SHALL support a local-only encrypted fallback mechanism for webhook verification secrets to simplify development, subject to strict constraints, and this mechanism MUST be clearly restricted to non-production environments.

Details:
- Any local secret file:
  - MUST be clearly documented as non-production-only.
  - MUST be encrypted or otherwise protected at rest on the developer machine.
  - MUST NOT be committed to version control or distributed to shared environments.
- The configuration documentation:
  - MUST explicitly state that using the local fallback mechanism in production is PROHIBITED.

#### Scenario: Local fallback clearly marked non-production
- WHEN a developer uses the local encrypted secret file for webhook testing
- THEN the documentation states that this mechanism is for local use only and MUST NOT be used in production or shared environments

### Requirement: Configuration for Webhook Verification Behavior
Webhook verification-related configuration MUST integrate with the global configuration system and be discoverable and testable.

Details:
- The configuration loader:
  - MUST expose typed accessors for:
    - `POBLYSH_WEBHOOK_GITHUB_SECRET`
    - `POBLYSH_WEBHOOK_SLACK_SIGNING_SECRET`
    - `POBLYSH_WEBHOOK_SLACK_TOLERANCE_SECONDS` (defaulting to `300` seconds when unset)
- The effective configuration:
  - MUST be validated at startup:
    - If a tolerance value is provided, it MUST be a positive integer.
- The behavior:
  - If a provider’s secret is set:
    - The corresponding verification logic MUST be enabled for that provider.
  - If a provider’s secret is unset:
    - Public requests for that provider without valid operator auth MUST be rejected with HTTP 401.

#### Scenario: Invalid tolerance rejected at startup
- GIVEN `POBLYSH_WEBHOOK_SLACK_TOLERANCE_SECONDS` is set to `0` or a negative value
- WHEN the service starts
- THEN configuration validation fails with a clear, non-secret-leaking error message and the process does not start

#### Scenario: Enabled provider verification uses configured secret
- GIVEN `POBLYSH_WEBHOOK_GITHUB_SECRET` is set to a known value
- WHEN a GitHub webhook with a correctly computed `X-Hub-Signature-256` is received on the public endpoint
- THEN verification uses the configured secret and the request is accepted with HTTP 202

### Requirement: Rate Limiting and Abuse Protection Configuration
The system SHALL support configuration for rate limiting and abuse protections applied to public webhook endpoints prior to signature verification.

Details:
- The implementation:
  - MUST support global rate limiting for public webhook endpoints.
  - SHOULD support per-IP or equivalent source-based rate limiting for public webhook endpoints.
- Configuration:
  - Default limits MAY be hard-coded but MUST be documented and SHOULD be overridable via configuration.
- Behavior:
  - Requests exceeding configured rate limits:
    - MUST be rejected with HTTP 429 using the unified problem+json envelope.
    - MUST NOT proceed to signature verification once identified as rate-limited.

#### Scenario: Rate limit blocks excessive unsigned traffic
- GIVEN rate limiting is configured for public webhook endpoints
- WHEN a single source exceeds the allowed request rate without valid signatures
- THEN excess requests are rejected with HTTP 429 before signature verification

#### Scenario: Rate limit configuration is documented
- WHEN operators review the webhook configuration documentation
- THEN it describes how to tune global and per-source rate limits and clarifies that upstream WAF/CDN integration is recommended for additional protection

### Requirement: Metrics and Logging Configuration Constraints
Configuration and implementation of metrics and logs for webhook verification MUST follow safe, low-cardinality, and privacy-preserving practices.

Details:
- Metrics:
  - MUST include counters for:
    - `signature_verification_success`
    - `signature_verification_failure`
    - `signature_verification_replay_reject`
  - SHOULD include latency histograms for signature verification.
  - MUST avoid unbounded-cardinality labels (e.g., raw IP addresses, unbounded request IDs, raw secrets).
  - Provider identifiers MAY be used as labels if they are from a bounded, documented set.
- Logging:
  - MUST emit structured logs for verification outcomes (e.g., provider, high-level reason).
  - MUST NOT log raw secrets, raw signatures, or full request bodies for successful verifications.
  - MAY log redacted or sampled diagnostic details for failures, provided no sensitive material is exposed.

#### Scenario: Metrics avoid unbounded cardinality
- WHEN metrics for webhook verification are scraped
- THEN no metric label contains raw secrets, raw IP addresses, or other unbounded sensitive identifiers

#### Scenario: Verification failures are observable without leaking secrets
- WHEN signature verification fails for a webhook request
- THEN logs record the provider and a high-level failure reason (e.g., "missing header", "invalid format", "mismatch") without including the secret or full signature value