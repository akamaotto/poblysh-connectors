## Why
Email is a primary source of work signals (conversations, threads, and actionable messages). We need a first‑class Gmail connector that ingests near‑real‑time updates via Google Cloud Pub/Sub push, acknowledges quickly, and performs reliable incremental fetches using Gmail History to produce normalized Signals.

## What Changes
- Add `gmail` provider to the registry with metadata: `auth_type = "oauth2"`, `webhooks = true`, minimal read scopes.
- Implement Gmail connector covering the `Connector` trait:
  - `authorize(tenant)` → Google OAuth URL with Gmail scopes and state
  - `exchange_token(code)` → exchange for access/refresh tokens; persist connection metadata
  - `refresh_token(connection)` → standard OAuth2 refresh flow
  - `handle_webhook(payload)` → accept Pub/Sub push, verify OIDC token, decode base64 `data`, enqueue incremental sync by `historyId`, respond immediately (ack)
  - `sync(connection, cursor?)` → incremental fetch via `users.history.list` and per‑message fetch where needed; emit normalized Signals; maintain cursor as last processed `historyId`
- Define normalized Signals for email updates (MVP): `email_updated`, `email_deleted` mapped from history deltas; creation vs update is inferred by first‑seen state.

## Impact
- Affected specs: `connectors` (new Gmail requirements). Optionally `config` for OIDC verification/env vars in a follow‑up.
- Affected code: `src/connectors/gmail.rs` (new), add to registry in `src/connectors/registry.rs`, webhook routing uses existing `POST /webhooks/{provider}`.
- Dependencies: add HTTP client, OAuth2, JWT verification, JWKS retrieval, base64, backoff (see design.md for exact versions and rationale).

## Non-Goals (MVP)
- Automatic Gmail `watch` lifecycle (create/renew/stop). MVP assumes the Pub/Sub Topic + push Subscription and per‑user `watch` are provisioned out‑of‑band; a later change will automate it.
- Full MIME parsing or attachment download; MVP focuses on metadata and minimal payload for updated/deleted events.
- Thread‑level normalization; MVP emits email‑level Signals only.

## Acceptance Criteria
- Provider registry exposes `gmail` with OAuth2, scopes `["https://www.googleapis.com/auth/gmail.readonly"]`, and `webhooks=true`.
- `authorize` returns a Google OAuth URL with configured scopes and unique state, using `access_type=offline` to obtain refresh tokens where allowed.
- `exchange_token` persists a `connections` row with `provider_slug='gmail'` and token details.
- Webhook ingest accepts Pub/Sub push for Gmail, verifies Google OIDC token on the request, decodes the base64 `data` envelope containing `{ emailAddress, historyId }`, and enqueues a sync job; handler responds 202 Accepted within 1s when healthy (ack target <1s; 2xx within Pub/Sub’s ack window <10s).
- Incremental sync processes history starting from the stored cursor using `users.history.list(startHistoryId=...)`, produces `email_updated` and `email_deleted` Signals, and advances the cursor to the highest processed `historyId`.
- Duplicate Pub/Sub deliveries are safely ignored via idempotent enqueueing keyed by message `messageId` or `(connection_id, historyId)`.
- When Gmail returns `historyId` too old/invalid, connector performs a bounded backfill strategy and emits a specific error signal or schedules a full re‑sync task (documented behavior in design.md).
- `openspec validate add-gmail-connector --strict` passes.
