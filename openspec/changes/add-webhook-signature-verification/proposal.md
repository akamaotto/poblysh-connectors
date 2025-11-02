## Why
The webhook ingest endpoint currently requires operator authentication for MVP. To receive real provider callbacks, we must support public delivery with strong request authentication. GitHub and Slack both provide HMAC-based signatures; adding verification enables secure public webhooks while keeping operator-protected mode for local testing.

## What Changes
- Add signature verification for GitHub (`X-Hub-Signature-256`, HMAC-SHA256) and Slack v2 (`X-Slack-Signature` + `X-Slack-Request-Timestamp`).
- Enable a public webhook path that bypasses operator auth when a valid provider signature is present.
- Introduce provider secrets via configuration for verification.
- Recommend and document a tenant-aware path form for public webhooks.
- Update OpenAPI for signature headers and public access notes.

## Impact
- Affected specs: `api-webhooks` (signature verification + public access), `config` (provider secrets), `auth` (bypass rule for signed webhooks)
- Affected code: router (public route), webhook handler (raw body access), verification helpers, optional replay window for Slack
- Dependencies: `hmac`, `sha2`, constant-time comparison utilities

