## ADDED Requirements

### Requirement: Webhook Verification Secrets
The system SHALL use provider-specific secrets from configuration to verify webhook signatures.

Variables (MVP):
- `POBLYSH_WEBHOOK_GITHUB_SECRET` (string, optional): enables GitHub signature verification when set
- `POBLYSH_WEBHOOK_SLACK_SIGNING_SECRET` (string, optional): enables Slack v2 signature verification when set
- `POBLYSH_WEBHOOK_SLACK_TOLERANCE_SECONDS` (int, default 300): maximum allowed clock skew for Slack timestamps

Behavior:
- If a provider secret is set, the corresponding public webhook verification MUST be enabled.
- If no secret is set for a provider, public verification for that provider MUST be disabled and requests without operator auth MUST be rejected.
- Secrets MUST be treated as sensitive and redacted in logs.
- Operational guidance MUST document that production secrets are sourced from an approved centralized secrets manager that enforces AES-256 encryption at rest, audited access, and a 30–90 day rotation cadence with optional dual-key grace period.
- The team MUST provide a documented and encrypted local development fallback file and clearly warn that it is prohibited for production use.

#### Scenario: Missing provider secret disables public verification
- GIVEN `POBLYSH_WEBHOOK_GITHUB_SECRET` is unset
- WHEN a GitHub webhook arrives without operator auth
- THEN the request is rejected with HTTP 401

#### Scenario: Slack tolerance can be configured
- GIVEN `POBLYSH_WEBHOOK_SLACK_TOLERANCE_SECONDS=120`
- WHEN a Slack webhook arrives with timestamp 2 minutes old
- THEN the request is accepted if the signature matches

#### Scenario: Secrets manager operational guidance documented
- WHEN onboarding production, operators consult the webhook verification runbook
- THEN it specifies the centralized secrets manager integration, rotation cadence, and audit logging expectations

#### Scenario: Local fallback clearly marked non-production
- WHEN a developer uses the encrypted local secret file
- THEN the documentation states it is for local use only and MUST NOT be deployed to production
