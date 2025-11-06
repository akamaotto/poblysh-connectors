## ADDED Requirements

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

