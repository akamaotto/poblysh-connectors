# config Specification

## Purpose
This specification defines the configuration system for the Poblysh Connectors API, including environment variable loading, validation, crypto key management, and provider-specific configuration settings. It targets developers and operators who need to configure the service for different environments (local, test, production) and understand the validation requirements and security considerations for sensitive configuration values.
## Requirements
### Requirement: Env Prefix And Layered Loading
The system SHALL load configuration from environment variables using the `POBLYSH_` prefix and support layered `.env` files for local development. Later items in the precedence list override earlier ones.

Load order (first → last): `.env`, `.env.local`, `.env.<profile>`, `.env.<profile>.local`, then process environment.

#### Scenario: Base .env loads
- WHEN `.env` contains `POBLYSH_API_BIND_ADDR=127.0.0.1:3000`
- AND no other overrides are present
- THEN the server uses `127.0.0.1:3000`

#### Scenario: Profile override wins
- GIVEN `.env` sets `POBLYSH_API_BIND_ADDR=127.0.0.1:3000`
- AND `.env.local` sets `POBLYSH_API_BIND_ADDR=0.0.0.0:8081`
- WHEN the service starts
- THEN it binds to `0.0.0.0:8081`

#### Scenario: OS environment has highest precedence
- GIVEN `.env.<profile>.local` sets `POBLYSH_API_BIND_ADDR=0.0.0.0:8082`
- AND the OS environment sets `POBLYSH_API_BIND_ADDR=0.0.0.0:9090`
- WHEN the service starts
- THEN it binds to `0.0.0.0:9090`

### Requirement: Local Profiles
The system SHALL support a `POBLYSH_PROFILE` variable to select local profiles. Valid profiles for MVP are `local` (default) and `test`; additional profiles may be added later (e.g., `dev`, `prod`).

#### Scenario: Default profile is local
- WHEN `POBLYSH_PROFILE` is not set
- THEN the effective profile is `local`

#### Scenario: Profile-specific file loads
- GIVEN `POBLYSH_PROFILE=test`
- AND `.env.test` defines `POBLYSH_LOG_LEVEL=debug`
- WHEN the service starts
- THEN the effective log level is `debug`

### Requirement: Typed Application Config
The system SHALL expose a typed configuration struct (`AppConfig`) sourced from env with sensible defaults for non-critical settings.

Fields (MVP):
- `profile` (string) default `local` from `POBLYSH_PROFILE`
- `api_bind_addr` (string) default `0.0.0.0:8080` from `POBLYSH_API_BIND_ADDR`
- `log_level` (string) default `info` from `POBLYSH_LOG_LEVEL`

#### Scenario: Defaults applied when unset
- WHEN no `POBLYSH_*` env variables are set
- THEN `api_bind_addr` defaults to `0.0.0.0:8080`
- AND `log_level` defaults to `info`

### Requirement: Validation And Fail Fast
The config loader MUST validate required fields and value formats on startup, aggregating errors and exiting with a non-zero code if invalid.

Validation (MVP):
- `api_bind_addr` MUST parse as a valid socket address (host:port)

#### Scenario: Invalid bind address causes startup failure
- GIVEN `POBLYSH_API_BIND_ADDR=not-an-addr`
- WHEN the service starts
- THEN startup fails with an error explaining the invalid address

### Requirement: Redacted Config Logging
The system SHALL log the loaded configuration at debug level with sensitive fields redacted.

#### Scenario: Secrets redacted
- GIVEN the config contains secret-like fields (e.g., `*_KEY`, `*_SECRET`, `*_TOKEN`, `DATABASE_URL` password)
- WHEN debug logging is enabled
- THEN those values are redacted in logs (e.g., `****`)

### Requirement: Logging Format Configuration
The system SHALL support a `POBLYSH_LOG_FORMAT` variable to control log output format.

Details (MVP):
- Accepted values: `json` (default) and `pretty`
- Unknown values MUST fall back to `json`

#### Scenario: Default format is JSON
- WHEN `POBLYSH_LOG_FORMAT` is unset
- THEN logs are emitted in JSON format

#### Scenario: Pretty format selected
- GIVEN `POBLYSH_LOG_FORMAT=pretty`
- WHEN the service starts
- THEN logs are emitted in a human-readable text format

### Requirement: Sync Engine Scheduler Configuration
The system SHALL support configuration for the sync engine scheduler through environment variables prefixed with `POBLYSH_SYNC_SCHEDULER_`.

Configuration fields:
- `TICK_INTERVAL_SECONDS` (integer, default 60) - Interval between scheduler ticks, range 10-300 seconds
- `DEFAULT_INTERVAL_SECONDS` (integer, default 900) - Default sync interval for connections without metadata override, range 60-86400 seconds  
- `JITTER_PCT_MIN` (float, default 0.0) - Minimum jitter percentage (0.0 = no minimum jitter)
- `JITTER_PCT_MAX` (float, default 0.2) - Maximum jitter percentage as fraction of interval (0.2 = 20% of interval)
- `MAX_OVERRIDDEN_INTERVAL_SECONDS` (integer, default 86400) - Maximum allowed interval override in connection metadata

All percentage values must be between 0.0 and 1.0. The jitter range MUST be validated to ensure `JITTER_PCT_MIN <= JITTER_PCT_MAX`.

#### Scenario: Default scheduler configuration
- WHEN no `POBLYSH_SYNC_SCHEDULER_*` variables are set
- THEN the scheduler uses: 60-second tick interval, 900-second default sync interval, 0-20% jitter range, 86400-second max override

#### Scenario: Custom scheduler intervals
- GIVEN `POBLYSH_SYNC_SCHEDULER_TICK_INTERVAL_SECONDS=30`
- AND `POBLYSH_SYNC_SCHEDULER_DEFAULT_INTERVAL_SECONDS=1800`
- WHEN the scheduler starts
- THEN it ticks every 30 seconds and uses 30-minute default sync intervals

#### Scenario: Jitter range customization
- GIVEN `POBLYSH_SYNC_SCHEDULER_JITTER_PCT_MIN=0.05`
- AND `POBLYSH_SYNC_SCHEDULER_JITTER_PCT_MAX=0.3`
- WHEN the scheduler computes jitter for a 900-second interval
- THEN jitter is sampled uniformly from 45-270 seconds (5%-30% of 900)

#### Scenario: Invalid configuration rejected
- GIVEN `POBLYSH_SYNC_SCHEDULER_JITTER_PCT_MAX=1.5` (invalid: > 1.0)
- WHEN the service starts
- THEN startup fails with validation error explaining the invalid percentage range

#### Scenario: Jitter range validation
- GIVEN `POBLYSH_SYNC_SCHEDULER_JITTER_PCT_MIN=0.3`
- AND `POBLYSH_SYNC_SCHEDULER_JITTER_PCT_MAX=0.1` (invalid: min > max)
- WHEN the service starts
- THEN startup fails with validation error about inverted jitter range

#### Scenario: Max override enforcement
- GIVEN `POBLYSH_SYNC_SCHEDULER_MAX_OVERRIDDEN_INTERVAL_SECONDS=3600`
- AND a connection has `metadata.sync.interval_seconds = 7200` (exceeds max)
- WHEN the scheduler loads the connection's metadata
- THEN it ignores the override, uses the default interval, and logs a warning about the validation failure

### Requirement: Scheduler Configuration Validation
The scheduler configuration MUST be validated on startup with the following rules:

Validation rules:
- `TICK_INTERVAL_SECONDS`: Must be >= 10 and <= 300
- `DEFAULT_INTERVAL_SECONDS`: Must be >= 60 and <= `MAX_OVERRIDDEN_INTERVAL_SECONDS`
- `JITTER_PCT_MIN`: Must be >= 0.0 and <= 1.0
- `JITTER_PCT_MAX`: Must be >= 0.0 and <= 1.0
- `JITTER_PCT_MIN` must be <= `JITTER_PCT_MAX`
- `MAX_OVERRIDDEN_INTERVAL_SECONDS`: Must be >= 60 and <= 604800 (7 days)

Invalid configurations MUST cause startup failure with descriptive error messages.

#### Scenario: Tick interval validation
- GIVEN `POBLYSH_SYNC_SCHEDULER_TICK_INTERVAL_SECONDS=5` (below minimum)
- WHEN the service starts
- THEN startup fails with error: "SYNC_SCHEDULER_TICK_INTERVAL_SECONDS must be between 10 and 300 seconds, got 5"

### Requirement: Crypto Key Configuration
The system SHALL require a symmetric crypto key provided via `POBLYSH_CRYPTO_KEY` to encrypt/decrypt tokens at rest using AES‑256‑GCM.

Details:
- `POBLYSH_CRYPTO_KEY` MUST be a base64‑encoded 32‑byte value (256‑bit key) using standard base64 encoding with proper padding.
- Validation MUST check for: missing environment variable, invalid base64 characters, incorrect padding, and exact 32-byte decoded length.
- Startup MUST fail with a clear error if the key is missing or invalid in any profile.
- The key value MUST be treated as a secret and redacted in logs and diagnostics.
- Error messages MUST indicate the specific validation failure without exposing partial key material.

#### Scenario: Missing key causes startup failure
- **GIVEN** `POBLYSH_CRYPTO_KEY` is unset
- **WHEN** the service starts
- **THEN** startup fails with an actionable error indicating the crypto key is required

#### Scenario: Invalid base64 length rejected
- **GIVEN** `POBLYSH_CRYPTO_KEY=Zm9v` (base64 for 3 bytes)
- **WHEN** the service starts
- **THEN** startup fails indicating the key must decode to exactly 32 bytes

#### Scenario: Invalid base64 characters rejected
- **GIVEN** `POBLYSH_CRYPTO_KEY=invalid!@#$%^&*()`
- **WHEN** the service starts
- **THEN** startup fails with a base64 format validation error

#### Scenario: Invalid base64 padding rejected
- **GIVEN** `POBLYSH_CRYPTO_KEY=YWJjZGVm` (missing padding)
- **WHEN** the service starts
- **THEN** startup fails indicating invalid base64 padding

### Requirement: GitHub OAuth Configuration
The system SHALL read GitHub OAuth configuration from environment variables.

Env vars (MVP):
- `GITHUB_CLIENT_ID` (required)
- `GITHUB_CLIENT_SECRET` (required; secret)
- `GITHUB_OAUTH_BASE` (optional; default `https://github.com`)
- `GITHUB_API_BASE` (optional; default `https://api.github.com`)

#### Scenario: Missing client id fails startup
- **WHEN** `POBLYSH_GITHUB_CLIENT_ID` is not set
- **THEN** startup validation fails with a clear error

#### Scenario: Defaults applied for base URLs
- **WHEN** base URLs are not set
- **THEN** the effective values are `https://github.com` and `https://api.github.com`

### Requirement: GitHub Webhook Secret
The system SHALL read the webhook signing secret for GitHub HMAC verification from an environment variable.

Env var:
- `GITHUB_WEBHOOK_SECRET` (required for public webhook mode; optional for operator-protected local mode)

#### Scenario: Public mode requires secret
- **WHEN** public webhook access is enabled
- **THEN** `POBLYSH_WEBHOOK_GITHUB_SECRET` MUST be set, otherwise startup fails with a clear error

### Requirement: Jira OAuth Configuration
The system SHALL read Jira OAuth configuration from environment variables. In local/test profiles, placeholder values MAY be used.

Env vars (MVP):
- `JIRA_CLIENT_ID` (required; empty values MUST fail fast outside local/test profiles)
- `JIRA_CLIENT_SECRET` (required; secret; MUST be stored in secret manager or env vault)
- `JIRA_OAUTH_BASE` (optional; default `https://auth.atlassian.com`)
- `JIRA_API_BASE` (optional; default `https://api.atlassian.com`)

#### Scenario: Local/test profiles may use placeholders
- **WHEN** running with `POBLYSH_PROFILE` in `local` or `test`
- **THEN** missing Jira OAuth settings MAY fall back to placeholder values suitable for development

#### Scenario: Defaults applied for base URLs
- **WHEN** base URLs are not set
- **THEN** the effective values are `https://auth.atlassian.com` and `https://api.atlassian.com`

#### Scenario: Missing required OAuth env vars fails in prod profiles
- **WHEN** `POBLYSH_PROFILE` is not `local` or `test`
- **THEN** initialization MUST reject startup if `JIRA_CLIENT_ID` or `JIRA_CLIENT_SECRET` are missing or empty

### Requirement: Jira Webhook Secret (Optional)
The system SHALL support an optional webhook verification secret for Jira webhooks in public mode. Note: Jira webhooks use basic authentication with user:password or API tokens rather than HMAC signature verification.

Env var:
- `WEBHOOK_JIRA_SECRET` (optional; shared secret for webhook verification when configured)

#### Scenario: Basic authentication verification when secret configured
- **WHEN** `WEBHOOK_JIRA_SECRET` is set
- **THEN** webhook handlers MUST verify the request using the configured secret as a shared authentication token, and reject requests with HTTP 401 when verification fails

#### Scenario: Webhook verification optional for development
- **WHEN** running with `POBLYSH_PROFILE` in `local` or `test`
- **THEN** Jira webhook verification MAY be disabled for development convenience

### Requirement: Pub/Sub OIDC Verification Configuration
The system SHALL provide configuration for verifying Google Cloud Pub/Sub push OIDC tokens and limiting webhook ingress size.

Details:
- `POBLYSH_PUBSUB_OIDC_AUDIENCE` (string) — required; the JWT `aud` claim MUST exactly match this value. Common patterns include the public webhook URL or a custom audience string set on the subscription.
- `POBLYSH_PUBSUB_OIDC_ISSUERS` (comma‑separated list) — optional; defaults to `accounts.google.com, https://accounts.google.com`. The JWT `iss` claim MUST exactly match one of these values.
- `POBLYSH_PUBSUB_MAX_BODY_KB` (integer) — optional; default `256`. Requests exceeding this size MUST be rejected before verification to preserve fast ack behavior.
- JWKS endpoint: `https://www.googleapis.com/oauth2/v3/certs` SHALL be used to validate the JWT signature by `kid`, and keys SHOULD be cached with ETag support.

#### Scenario: Audience must match exactly
- **WHEN** a push request is received with an OIDC JWT where `aud != POBLYSH_PUBSUB_OIDC_AUDIENCE`
- **THEN** the request is rejected with a non‑2xx status code to trigger Pub/Sub retry and a verification error is logged

#### Scenario: Issuer must be allowed
- **WHEN** a push request is received with an OIDC JWT where `iss` is not in `POBLYSH_PUBSUB_OIDC_ISSUERS`
- **THEN** the request is rejected with a non‑2xx status code to trigger Pub/Sub retry and a verification error is logged

#### Scenario: Expired or not‑yet‑valid token rejected
- **WHEN** the OIDC JWT is expired or has an `iat` outside an acceptable skew
- **THEN** the request is rejected with a non‑2xx status code to trigger Pub/Sub retry

#### Scenario: Webhook request size limited
- **WHEN** a push request body exceeds `POBLYSH_PUBSUB_MAX_BODY_KB`
- **THEN** the request is rejected with a non‑2xx status code and no work is enqueued

### Requirement: Zoho Cliq Webhook Secrets
The configuration layer SHALL support Zoho Cliq webhook verification via a shared token (MVP).

Details:
- `POBLYSH_WEBHOOK_ZOHO_CLIQ_TOKEN` (required for public route): used for constant‑time token comparison with `Authorization: Bearer`.
- HMAC‑based verification MAY be added in a follow‑up change once official docs confirm header names and signature construction.
- If token is not set, public webhook verification for `zoho-cliq` MUST be disabled (401 on public route).

#### Scenario: Missing token disables public verification
- **WHEN** `POBLYSH_WEBHOOK_ZOHO_CLIQ_TOKEN` is not set
- **THEN** `POST /webhooks/zoho-cliq/{tenant}` MUST return 401 (public route), and operators MAY use the protected route `POST /webhooks/zoho-cliq` with operator auth for manual testing

### Requirement: Crypto Key Documentation Coverage
The documentation SHALL describe the `POBLYSH_CRYPTO_KEY` configuration, including the base64-encoded 32-byte requirement, and provide a clear pointer to the local crypto rotation runbook for rotation steps.

#### Scenario: Developer understands key format
- **WHEN** a developer reads `docs/configuration.md`
- **THEN** they learn that `POBLYSH_CRYPTO_KEY` must be a standard base64 string that decodes to 32 bytes

#### Scenario: Developer locates rotation procedure
- **WHEN** a developer needs to rotate their local crypto key
- **THEN** the documentation links to `docs/runbooks/local-crypto-rotation.md`, providing the step-by-step process

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

