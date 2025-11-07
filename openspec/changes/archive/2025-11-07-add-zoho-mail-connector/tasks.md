## 1. Implementation
- [x] 1.1 Add provider metadata and register `zoho-mail` in provider registry
- [x] 1.2 Implement `src/connectors/zoho_mail.rs` with `Connector` trait methods
- [ ] 1.3 Implement OAuth `authorize` (region-aware Accounts URL) and `exchange_token`/`refresh_token`
- [ ] 1.4 Implement `sync` polling with time-based cursor and dedupe window
- [x] 1.5 Promote `reqwest` to runtime deps and add `oauth2`, `backoff` crates
- [ ] 1.6 Map Zoho message changes to Signals: `email_received`, `email_updated`, `email_deleted`
- [ ] 1.7 Populate `dedupe_key` using `hash(message_id || lastModifiedTime)`

## 2. Validation & QA
- [ ] 2.1 Unit tests: OAuth URL builder (DC-aware), token exchange/refresh (mock), polling window & dedupe logic
- [ ] 2.2 Integration tests: sync cursor advancement and idempotence across overlapping windows
- [ ] 2.3 Error handling tests: 401 triggers refresh, 429/5xx backoff respects Retry-After
- [ ] 2.4 OpenSpec validation: `openspec validate add-zoho-mail-connector --strict`

## 3. Docs & Ops
- [ ] 3.1 Document region/DC config and base URL resolver
- [ ] 3.2 Document default dedupe window and operational tuning guidance
- [ ] 3.3 Observability: structured logs for rate limiting, token refresh, and cursor progression
