## Context
To accept real provider callbacks, the webhook endpoint must be public and authenticate requests using provider signatures. GitHub and Slack provide HMAC‑SHA256 signatures over the raw body (Slack includes a timestamp). We will verify signatures and bypass operator auth for valid requests.

## Goals / Non-Goals
- Goals: Public webhook path with signature verification for GitHub and Slack; maintain operator‑auth path for local/testing
- Non‑Goals: Multi‑tenant per‑connection secrets, replay id de‑duplication, additional providers (Google, Zoho)

## Decisions
- Public route: `POST /webhooks/{provider}/{tenant_id}` conveys tenant context without headers
- GitHub: verify `X-Hub-Signature-256` using HMAC‑SHA256 of raw body, prefix `sha256=`
- Slack v2: verify `X-Slack-Signature` over `v0:{timestamp}:{raw_body}` and enforce ±`tolerance` window (default 300s)
- Config secrets: enable verification when env vars are set and document integration with the centralized secrets manager (AES-256 encrypted, audited access, rotation playbook) plus encrypted local fallback guidance
- Constant‑time compare for all signature equality checks using `subtle::ConstantTimeEq`
- Dependencies: `hmac 0.12`, `sha2 0.10`, `hex 0.4`, and existing `subtle 2.6` for deterministic digest computation and comparison
- Security telemetry: emit structured verification logs/metrics and enforce per-IP plus global rate limiting before signature evaluation

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

## Research Plan
- **Parallel discovery:** Simultaneously query Slack verification docs, GitHub webhook signature validation guidance, RustCrypto `hmac`/`sha2` best practices, and in-repo usages of `subtle` to reuse constant-time idioms.
- **Sequential reinforcement:** Follow each source with focused searches (e.g., “Slack timestamp tolerance rust”, “Rust hmac sha256 example github”) and cross-check community discussions for replay mitigation patterns.
- **Synthesis:** Capture verified header formats, tolerance defaults, and logging/rate-limiting recommendations in implementation notes before coding; update specs/tasks when new constraints surface.
