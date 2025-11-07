## 1. Implementation
- [x] 1.1 Add provider metadata and register `zoho-cliq` in provider registry
- [x] 1.2 Implement `src/connectors/zoho_cliq.rs` with `Connector` trait methods (webhook‑only MVP)
- [x] 1.3 Webhook handler: route `POST /webhooks/zoho-cliq/*` to Zoho Cliq connector; forward raw headers in `payload.headers` with lower‑case keys
- [x] 1.4 Add config entry: `POBLYSH_WEBHOOK_ZOHO_CLIQ_TOKEN` (Authorization: Bearer token for public route)
- [x] 1.5 Implement token verification using constant‑time compare via `subtle`; defer HMAC until docs confirm header names
- [x] 1.6 Map message events to Signals: `message_posted`, `message_updated`, `message_deleted` with normalized payload
- [x] 1.7 Populate `dedupe_key` using provider message/event identifier when present
 - [x] 1.8 Add `zoho-cliq` support to public webhook verification middleware (token‑only) to mirror Jira pattern
 - [x] 1.9 Update `/providers` static list (or migrate to registry) to include `zoho-cliq` metadata

## 2. Validation & QA
- [x] 2.1 Unit tests: verification success/failure, event mapping from sample payloads
- [ ] 2.2 Integration tests (mock): webhook ingest happy path and auth failures
- [ ] 2.3 OpenAPI docs mention public webhook variant and expected headers for Zoho Cliq

## 3. Docs & Ops
- [ ] 3.1 Document manual webhook configuration steps (token-only) for local
- [ ] 3.2 Document rate limiting expectations and retries behavior
- [ ] 3.3 Add observability: structured logs and metrics on verification outcomes
