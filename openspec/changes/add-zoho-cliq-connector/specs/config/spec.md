## ADDED Requirements

### Requirement: Zoho Cliq Webhook Secrets
The configuration layer SHALL support Zoho Cliq webhook verification via either HMAC secret or shared token.

Details:
- `POBLYSH_WEBHOOK_ZOHO_CLIQ_SECRET` (preferred): used for HMAC‑SHA256 verification when Zoho signs requests
- `POBLYSH_WEBHOOK_ZOHO_CLIQ_TOKEN` (fallback): used for constant‑time token comparison when no signature is provided
- If neither is set, public webhook verification for `zoho-cliq` MUST be disabled and the endpoint requires operator auth

#### Scenario: Missing secret/token disables public verification
- **WHEN** neither `POBLYSH_WEBHOOK_ZOHO_CLIQ_SECRET` nor `POBLYSH_WEBHOOK_ZOHO_CLIQ_TOKEN` is set
- **THEN** public requests to `POST /webhooks/zoho-cliq/{tenant}` are rejected unless operator auth is provided

