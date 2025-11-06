## 1. Implementation
- [ ] 1.1 Add provider metadata and register `zoho-cliq` in provider registry
- [ ] 1.2 Implement `src/connectors/zoho_cliq.rs` with `Connector` trait methods (webhook‑only MVP)
- [ ] 1.3 Webhook handler: route `POST /webhooks/zoho-cliq/*` to Zoho Cliq connector; forward raw headers in `payload.headers` with lower‑case keys
- [ ] 1.4 Add config entries: `POBLYSH_WEBHOOK_ZOHO_CLIQ_SECRET` (HMAC) and fallback `POBLYSH_WEBHOOK_ZOHO_CLIQ_TOKEN` (shared token)
- [ ] 1.5 Implement signature/token verification using `hmac`+`sha2` or constant‑time compare via `subtle`
- [ ] 1.6 Map message events to Signals: `message_posted`, `message_updated`, `message_deleted` with normalized payload
- [ ] 1.7 Populate `dedupe_key` using provider message/event identifier when present

## 2. Validation & QA
- [ ] 2.1 Unit tests: verification success/failure, event mapping from sample payloads
- [ ] 2.2 Integration tests (mock): webhook ingest happy path and auth failures
- [ ] 2.3 OpenAPI docs mention public webhook variant and expected headers for Zoho Cliq

## 3. Docs & Ops
- [ ] 3.1 Document manual webhook configuration steps (secret/token) for local
- [ ] 3.2 Document rate limiting expectations and retries behavior
- [ ] 3.3 Add observability: structured logs and metrics on verification outcomes

