## 1. Implementation
- [x] 1.1 Add provider metadata and register `github` in provider registry
- [x] 1.2 Implement `src/connectors/github.rs` with `Connector` trait methods
- [x] 1.3 Wire OAuth start/callback to call `authorize`/`exchange_token` for `github`
- [x] 1.4 Add config loader entries for GitHub client ID/secret and webhook secret
- [x] 1.5 Webhook handler: route `POST /webhooks/github/*` to GitHub connector and verify signature
- [x] 1.6 Map GitHub events (issues, pull_request) to normalized Signals
- [x] 1.7 Implement REST backfill (issues, pull requests) with cursoring, pagination, and rate-limit handling
- [x] 1.8 Persist connection metadata (user id/login) and sync cursor in `connections.metadata`
- [x] 1.9 Add unit tests: signature verification, event mapping, token exchange/refresh
- [ ] 1.10 Add integration tests: OAuth flow happy path (mock), webhook ingest (mock), backfill sync (wiremock)

## 2. Observability & Docs
- [x] 2.1 Add tracing spans and structured logs for webhook verification and sync/backfill calls
- [x] 2.2 Emit metrics for rate limiting and API errors from GitHub
- [x] 2.3 Document manual webhook configuration and required secrets for local

## 3. Validation
- [x] 3.1 Run `openspec validate add-github-connector --strict`
- [x] 3.2 Ensure QA review items (if any) are resolved

