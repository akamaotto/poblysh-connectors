## Context
To accept real provider callbacks, the webhook endpoint must be public and authenticate requests using provider signatures. GitHub and Slack provide HMAC‑SHA256 signatures over the raw body (Slack includes a timestamp). We will verify signatures and bypass operator auth for valid requests.

## Goals / Non-Goals
- Goals: Public webhook path with signature verification for GitHub and Slack; maintain operator‑auth path for local/testing
- Non‑Goals: Multi‑tenant per‑connection secrets, replay id de‑duplication, additional providers (Google, Zoho)

## Decisions
- Public route: `POST /webhooks/{provider}/{tenant_id}` conveys tenant context without headers
- GitHub: verify `X-Hub-Signature-256` using HMAC‑SHA256 of raw body, prefix `sha256=`
- Slack v2: verify `X-Slack-Signature` over `v0:{timestamp}:{raw_body}` and enforce ±`tolerance` window (default 300s)
- Config secrets: enable verification when env vars are set; otherwise public verification disabled for that provider
- Constant‑time compare for all signature equality checks

## Risks / Trade-offs
- Single global secret per provider vs per‑tenant/connection secrets; MVP chooses global secret for simplicity
- Replay protection: Slack window enforced; GitHub lacks timestamp → rely on HMAC only in MVP (mitigate by short processing and future de‑duplication via `X-GitHub-Delivery`)

## Migration Plan
- Keep `/webhooks/{provider}` path with operator auth for local/test
- Add public `{tenant_id}` path and verification; document provider configuration steps
- No database changes required

## Open Questions
- Should we accept both path forms for all providers or gradually migrate?
- When to introduce per‑tenant/connection secrets and delivery de‑duplication

