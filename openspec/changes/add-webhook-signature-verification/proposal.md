## Why
The webhook ingest endpoint currently requires operator authentication for MVP. To receive real provider callbacks, we must support public delivery with strong request authentication. GitHub and Slack both provide HMAC-based signatures; adding verification enables secure public webhooks while keeping operator-protected mode for local testing.

## What Changes

- Add signature verification for GitHub (`X-Hub-Signature-256`, HMAC-SHA256) and Slack v2 (`X-Slack-Signature` + `X-Slack-Request-Timestamp`) with mandatory timestamp validation that rejects requests outside a 5-minute skew window and prevents replay by default.
- Enable a public webhook path that bypasses operator auth when a valid provider signature is present.
- Introduce provider secrets via a centralized secrets manager (e.g., HashiCorp Vault, cloud KMS/Secrets Manager) that enforces AES-256 encryption at rest, short-lived least-privilege IAM roles, secure retrieval APIs or startup-only environment injection, automated 30–90 day rotation with dual-key grace period and documented rollover procedure, per-environment and per-tenant isolation, and comprehensive audit logging for all access and revocation events.
- Provide a documented local development fallback using an encrypted developer-only secret file with prominent warnings that it is prohibited for production use.
- Recommend and document a tenant-aware path form for public webhooks.
- Update OpenAPI for signature headers and public access notes.

## Impact
- Affected specs: `api-webhooks` (signature verification + public access), `config` (provider secrets), `auth` (bypass rule for signed webhooks)
- Affected code: router (public route) rejects stale Slack requests, webhook handler (raw body access), verification helpers enforcing Slack timestamp replay protection by default
- Dependencies: `hmac`, `sha2`, constant-time comparison utilities

## Acceptance Criteria

- Secrets are stored in a centralized manager with AES-256 encryption at rest.
- Automated rotation (30–90 day window) with dual-key grace period and documented rollover procedure is in place.
- Secrets are isolated per environment and tenant.
- Access uses short-lived, least-privilege credentials delivered only via secure retrieval or startup injection.
- Audit logs capture every secret access and revocation event.
- Local development fallback remains encrypted, developer-only, and clearly marked as unsuitable for production.
