## ADDED Requirements

### Requirement: Zoho Cliq Webhook Secrets
The configuration layer SHALL support Zoho Cliq webhook verification via a shared token (MVP).

Details:
- `POBLYSH_WEBHOOK_ZOHO_CLIQ_TOKEN` (required for public route): used for constant‑time token comparison with `Authorization: Bearer`.
- HMAC‑based verification MAY be added in a follow‑up change once official docs confirm header names and signature construction.
- If token is not set, public webhook verification for `zoho-cliq` MUST be disabled (401 on public route).

#### Scenario: Missing token disables public verification
- **WHEN** `POBLYSH_WEBHOOK_ZOHO_CLIQ_TOKEN` is not set
- **THEN** `POST /webhooks/zoho-cliq/{tenant}` MUST return 401 (public route), and operators MAY use the protected route `POST /webhooks/zoho-cliq` with operator auth for manual testing
