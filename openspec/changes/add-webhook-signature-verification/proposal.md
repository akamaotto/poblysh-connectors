## Why
The webhook ingest endpoint currently requires operator authentication for MVP. To receive real provider callbacks, we must support public delivery with strong request authentication. GitHub and Slack both provide HMAC-based signatures; adding verification enables secure public webhooks while keeping operator-protected mode for local testing.

## What Changes

- Add signature verification for GitHub (`X-Hub-Signature-256`, HMAC-SHA256) and Slack v2 (`X-Slack-Signature` + `X-Slack-Request-Timestamp`) with mandatory timestamp validation that rejects requests outside a 5-minute skew window and prevents replay by default.
- Enable a public webhook path that bypasses operator auth when a valid provider signature is present.
- Consume provider secrets via secure configuration (`POBLYSH_WEBHOOK_GITHUB_SECRET`, `POBLYSH_WEBHOOK_SLACK_SIGNING_SECRET`, optional tolerance) with operational guidance for supplying them from the organization's centralized secrets manager (AES-256 encrypted at rest, audited access, documented rotation cadence). Document the rotation playbook and per-environment isolation expectations.
- Provide a documented local development fallback that loads encrypted developer-only secret files with prominent warnings that they are prohibited for production use.
- Recommend and document a tenant-aware path form for public webhooks.
- Update OpenAPI for signature headers and public access notes.

### Security & Observability

- **Rate limiting & DDoS protection:** Enforce per-IP and global thresholds with defined burst limits on the public webhook path, and document integration points for upstream WAF/CDN shields (e.g., IP reputation feeds, request scoring, geo blocks) to absorb volumetric attacks before they reach the service.
- **Signature verification logging & metrics:** Emit structured logs and metrics for every signature verification attempt, including SUCCESS/FAILURE counters, timestamps, request IDs, provider identifiers, and mismatch reasons; capture sample request headers for failed verifications with all secrets and tokens redacted; expose aggregates suitable for dashboarding and alerting.
- **Missing or invalid signatures:** Reject requests with missing or malformed headers using 401 (unauthorized), increment failure metrics, and optionally trigger automated challenge flows or temporary provider/customer blacklisting after repeated failures from the same source.
- **Unauthenticated traffic & abuse mitigation:** Apply request throttling prior to signature evaluation, maintain temporary IP/block lists for abusive sources, and support CAPTCHAs or challenge/response mechanisms when abuse patterns are detected even before signature validation is attempted.
- **Retention, alerting & monitoring SLAs:** Define log retention windows (e.g., 30–90 days) aligned with compliance requirements, maintain dashboards tracking verification success rates and anomaly trends, and configure alerts for spikes in failures, replay attempts, or DDoS indicators to ensure timely operational response.

## Impact
- Affected specs: `api-webhooks` (signature verification + public access), `config` (provider secrets), `auth` (bypass rule for signed webhooks)
- Affected code: router (public route) rejects stale Slack requests, webhook handler (raw body access), verification helpers enforcing Slack timestamp replay protection by default
- Dependencies: `hmac`, `sha2`, constant-time comparison utilities

## Acceptance Criteria

- Secrets are supplied via secure configuration tied to an approved secrets manager with AES-256 encryption at rest and audited access; rotation and per-environment isolation expectations are documented.
- Rotation procedures (30–90 day cadence with dual-key grace period) and local fallback warnings are documented for operators.
- Local development fallback remains encrypted, developer-only, and clearly marked as unsuitable for production.
- Rate limiting, telemetry (logs/metrics), and alerting guidance for public webhook verification are implemented and documented.
