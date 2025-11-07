## 1. Implementation
- [x] 1.1 Add `gmail` provider metadata and connector stub to registry
- [x] 1.2 Implement `authorize` URL builder for Gmail scopes (offline access)
- [x] 1.3 Implement `exchange_token` and `refresh_token` flows (OAuth2)
- [x] 1.4 Implement webhook handler: verify Pub/Sub OIDC (mandatory), decode base64 `data`, enqueue sync job, return `202 Accepted` quickly; on verification failure return non‑2xx to trigger retry
- [x] 1.5 Implement `sync` using `users.history.list` and `users.messages.get` when needed; advance cursor to latest `historyId`
- [x] 1.6 Emit normalized Signals: `email_updated`, `email_deleted` with stable `dedupe_key`
- [x] 1.7 Add idempotency for Pub/Sub deliveries (prefer `messageId`, fallback `(connection_id, historyId)`)
- [x] 1.8 Align HTTP/TLS deps: `reqwest = { version = "0.12.9", default-features=false, features=["json","rustls-tls"] }`; `oauth2 = { version = "5.0", features=["reqwest","rustls-tls"] }`; remove duplicate/old dev-deps
- [x] 1.9 Add config: document and wire `POBLYSH_PUBSUB_OIDC_AUDIENCE`, `POBLYSH_PUBSUB_OIDC_ISSUERS`, `POBLYSH_PUBSUB_MAX_BODY_KB`

## 2. Validation & QA
- [x] 2.1 Unit tests for base64 decode and envelope parsing
- [x] 2.2 Unit tests for OIDC verification (JWKS happy path and failure cases)
- [x] 2.3 Unit tests for history paging and cursor advancement logic
- [x] 2.4 Golden tests for Signal mapping of history deltas
- [x] 2.5 Tests for rate-limit handling (429 / quota 403 → `SyncError::RateLimited`)

## 3. Docs
- [x] 3.1 Document required scopes and watch lifecycle caveats
- [x] 3.2 Document Pub/Sub push configuration (OIDC audience, service account)
- [x] 3.3 Outline re‑sync behavior when `historyId` is invalid/too old
- [x] 3.4 Add config spec deltas for OIDC verification env vars and webhook body limits
