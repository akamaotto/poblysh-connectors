## ADDED Requirements

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
