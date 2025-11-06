## 1. Implementation
- [ ] 1.1 Add `gmail` provider metadata and connector stub to registry
- [ ] 1.2 Implement `authorize` URL builder for Gmail scopes (offline access)
- [ ] 1.3 Implement `exchange_token` and `refresh_token` flows (OAuth2)
- [ ] 1.4 Implement webhook handler: verify Pub/Sub OIDC, decode base64 `data`, enqueue sync job, return 2xx quickly
- [ ] 1.5 Implement `sync` using `users.history.list` and `users.messages.get` when needed; advance cursor to latest `historyId`
- [ ] 1.6 Emit normalized Signals: `email_updated`, `email_deleted` with stable `dedupe_key`
- [ ] 1.7 Add idempotency and retry with exponential backoff for Gmail API calls
- [ ] 1.8 Move `reqwest = { version = "0.12.9", features = ["json", "rustls-tls"] }` to runtime dependencies

## 2. Validation & QA
- [ ] 2.1 Unit tests for base64 decode and envelope parsing
- [ ] 2.2 Unit tests for OIDC verification (JWKS happy path and failure cases)
- [ ] 2.3 Unit tests for history paging and cursor advancement logic
- [ ] 2.4 Golden tests for Signal mapping of history deltas

## 3. Docs
- [ ] 3.1 Document required scopes and watch lifecycle caveats
- [ ] 3.2 Document Pub/Sub push configuration (OIDC audience, service account)
- [ ] 3.3 Outline re‑sync behavior when `historyId` is invalid/too old
