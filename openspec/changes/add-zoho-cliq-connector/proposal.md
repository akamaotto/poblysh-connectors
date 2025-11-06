## Why
Teams use Zoho Cliq for chat collaboration. We need a webhook‑only Zoho Cliq connector to ingest message events and emit normalized Signals so downstream automations can react to message creation/updates/deletions across configured channels.

## What Changes
- Add `zoho-cliq` provider metadata to the registry: `auth_type = "custom(webhook)"`, `webhooks = true`, no OAuth in MVP.
- Implement Zoho Cliq connector covering the `Connector` trait surface for MVP:
  - `authorize`/`exchange_token`/`refresh_token` → no‑op or explicit "unsupported" errors in MVP (webhook‑only)
  - `handle_webhook(payload)` → verify signature or shared token, parse message events, emit Signals
  - `sync(connection, cursor?)` → no polling/backfill in MVP (future: channel history API via OAuth)
- Add configuration entries for webhook auth: `POBLYSH_WEBHOOK_ZOHO_CLIQ_SECRET` (HMAC) or `POBLYSH_WEBHOOK_ZOHO_CLIQ_TOKEN` (shared token), choosing HMAC when configured.
- Define normalized message Signal kinds (MVP): `message_posted`, `message_updated`, `message_deleted` with consistent payload mapping.
- Document rate limiting and idempotency guidance for webhook ingest; include `dedupe_key` from message/event identifiers when available.

## Impact
- Affected specs: `connectors` (new connector requirements), `config` (Zoho Cliq webhook secret/token), `api-webhooks` (public path variant usage unchanged).
- Affected code: `src/connectors/zoho_cliq.rs` (new), registry initialization, webhook handler dispatch to connector; optional shared verification helpers reuse.
- Dependencies: reuse existing `axum`, `serde`, `tracing`; add `hmac` and `sha2` for HMAC‑SHA256 verification if Zoho Cliq supports it; otherwise use constant‑time compare via `subtle` for shared token.

## Non-Goals (MVP)
- OAuth‑based API access or historical backfill (future iteration).
- Real‑time socket/bot integrations (e.g., event streams) beyond HTTP webhooks.
- Channel management or webhook provisioning APIs (manual configuration expected in MVP).

## Acceptance Criteria
- `POST /webhooks/zoho-cliq/{tenant}` accepts signed (HMAC) or token‑authenticated requests and returns 202 with body `{ "status": "accepted" }` per public webhook spec.
- Valid message events produce Signals with kinds: `message_posted`, `message_updated`, or `message_deleted`, including normalized fields: `{ message_id, channel_id, user_id, text, occurred_at, raw }`; `payload.headers` keys are lower‑case.
- Invalid or unauthenticated requests are rejected with 401 (no operator auth), following existing signature/token verification flows.
- `openspec validate add-zoho-cliq-connector --strict` passes.

## Core Technologies and Versions
- axum `0.8.6` (router, extractors), tokio `1.48.0` (runtime)
- serde `1.0.217`, serde_json `1.0.138`
- tracing `0.1.41`, tracing-subscriber `0.3.19`
- subtle `2.6.1` for constant‑time comparison (already present)
- NEW: hmac `0.12.1` and sha2 `0.10.8` (HMAC‑SHA256 verification) — align with existing signature verification change
- Optional future backfill: reqwest `0.12.9` (already in dev-deps; promote to runtime dep later when OAuth/API is added)

## Research Plan (Lightweight Deep Research Algorithm)
Goal: confirm Zoho Cliq webhook authentication model, payload shapes for message events, and best‑practice handling; select precise verification method and headers.

1) Parallel discovery (run concurrently)
   - Web docs: search "Zoho Cliq outgoing webhooks", "Zoho Cliq message webhook payload", "Zoho Cliq HMAC signature"
   - API docs: browse Zoho Cliq REST and platform docs for "Outgoing Webhooks", "Incoming Webhooks" and authentication sections
   - Community: scan StackOverflow, GitHub issues, Zoho community for pitfalls (signature headers, timestamps, retries)
   - Codebase scan: `rg -n "webhook|signature|hmac|zoho|cliq"` to reuse helper patterns

2) Sequential reinforcement (narrow and verify)
   - From docs, extract exact header names and algorithms (e.g., `X-Cliq-Signature` HMAC vs static token); confirm shared secret configuration steps
   - Validate payload examples: identify message fields (`message_id`, `channel_id`, `user_id`, `text`, `ts`) and any delivery id
   - Cross‑check multiple sources; if conflicts, prefer primary docs; confirm with test payloads if sample tool available
   - Map to normalized Signal fields; define `dedupe_key` using event/message id or delivery id
   - Capture rate limits and retry behaviors; decide idempotency strategy for duplicate deliveries

3) Synthesis and decisions
   - Choose verification path: HMAC (preferred) or token; define exact header names and tolerance (timestamps/replay if available)
   - Update spec with concrete header names, examples, and error codes
   - Lock crate versions and finalize configuration names: `POBLYSH_WEBHOOK_ZOHO_CLIQ_SECRET|TOKEN`

Reference docs to consult (to be verified during research execution)
- Zoho Cliq REST/Platform: Outgoing webhooks guide and message event payloads
- Zoho Cliq Incoming webhooks (for returning responses to channel) — nice‑to‑have for echo replies
- Community threads: webhook signature verification and retries
