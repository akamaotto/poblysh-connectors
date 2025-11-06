## Context
Build a Zoho Mail connector with OAuth2 and incremental polling using a bounded dedupe window. The service already uses Axum, Tokio, SeaORM, and Utoipa. We’ll add OAuth2 and a resilient HTTP client with backoff to fetch message changes and produce normalized Signals.

## Goals / Non-Goals
- Goals:
  - OAuth2 authorize/exchange/refresh with Zoho Accounts (region-aware)
  - Incremental polling using time-based cursoring and a dedupe window
  - Emit normalized Signals for received/updated/deleted messages with stable keys
  - Handle rate limits with backoff and a monotonic cursor
- Non-Goals:
  - Webhook push integrations for Zoho Mail (MVP is polling-only)
  - Full MIME/body parsing, attachments download
  - IMAP protocol implementation

## Tech and Crate Selection

Existing crate versions (from Cargo.toml):
- axum = 0.8.6; tokio = 1.48.0; serde = 1.0.217; serde_json = 1.0.138
- tracing = 0.1.41; tracing-subscriber = 0.3.19
- sea-orm = 1.1.17; utoipa = 5.3.1; url = 2.5.4
- chrono = 0.4.38; uuid = 1.11.0; anyhow = 1.0.95; thiserror = 2.0.11
- tower = 0.5.1

Selected additions (runtime deps):
- reqwest = 0.12.9 (features: `json`, `rustls-tls`) — HTTP client for Zoho APIs
- oauth2 = 5.0.0 — Authorization code + refresh token flows
- backoff = 0.4.0 — Exponential backoff for 429/5xx
- base64 = 0.22.1 — Optional (include if API responses require it)
- lru = 0.16.2 — Optional small cache for folder/label lookups

Rationale:
- Keep TLS stack consistent with project runtime (rustls) and avoid native-tls.
- Favor general-purpose OAuth2 and HTTP libraries; generated clients are unnecessary for MVP.
- Backoff and idempotent dedupe window provide reliability without complex state machines.

## OAuth2 and Region Handling
- Accounts base URL varies by data center:
  - US: `https://accounts.zoho.com`
  - EU: `https://accounts.zoho.eu`
  - IN: `https://accounts.zoho.in`
  - Others (e.g., AU, JP): follow Zoho docs; expose via `POBLYSH_ZOHO_DC` and resolver
- Authorization endpoint: `${ACCOUNTS_BASE}/oauth/v2/auth`
  - Params: `client_id`, `redirect_uri`, `scope`, `response_type=code`, `access_type=offline`, `state`
- Token endpoint: `${ACCOUNTS_BASE}/oauth/v2/token`
  - Grant types: `authorization_code`, `refresh_token`
- Scopes (MVP): `ZohoMail.messages.READ` (confirm exact scope in research step)

## Polling and Dedupe Window
- Cursor type: RFC3339 timestamp (UTC) representing last processed `lastModifiedTime`
- Dedupe window: `window_secs` (default 300) to absorb late-arriving modifications
- Query strategy:
  - If the Email Messages API supports server-side `lastModifiedTime >= T` filters, use that with ascending sort by `lastModifiedTime`
  - Else, use Search API with a time-range query (e.g., `after: <ts> before: <now>`), and filter client-side on `lastModifiedTime`
- Dedupe key: `hash(message_id || lastModifiedTime)` to avoid duplicate Signals across overlapping windows
- Advance cursor to the maximum `lastModifiedTime` observed to ensure monotonic progress

## HTTP and Retry Policy
- Timeouts: default `POBLYSH_HTTP_TIMEOUT_SECS` (e.g., 15s)
- Retries: 3x with exponential backoff + jitter on 429/5xx; honor `Retry-After` if present
- Token refresh: on 401, attempt one refresh; if still failing, return typed provider error

## Configuration
- `POBLYSH_ZOHO_DC` — `us|eu|in|...` controls Accounts/API base URLs
- `POBLYSH_ZOHO_MAIL_SCOPES` — default `ZohoMail.messages.READ`
- `POBLYSH_ZOHO_MAIL_POLL_WINDOW_SECS` — default `300`
- `POBLYSH_HTTP_TIMEOUT_SECS` — default `15`

## Module Structure and Integration
- `src/connectors/zoho_mail.rs` implements `Connector`
  - `authorize` builds Accounts URL with configured DC + scopes
  - `exchange_token` calls token endpoint and persists connection
  - `refresh_token` rotates tokens via refresh grant
  - `sync` executes polling with cursor and dedupe window, returning Signals
- Add `register_zoho_mail_connector` to `src/connectors/registry.rs` with metadata

## Normalized Signals (MVP)
- `email_received` — first time a message is seen (or status indicates newly received)
- `email_updated` — message metadata/labels/folder changed; include `last_modified` (mapped from provider `lastModifiedTime`)
- `email_deleted` — message deleted; include identifiers and occurred_at
Fields: `{ message_id, thread_id?, folder_id?, from, to, subject, occurred_at, raw }`

## Docs and References (accessed Nov 2025)
- Zoho Mail API Index: https://www.zoho.com/mail/help/api/
- Zoho OAuth 2.0: https://www.zoho.com/accounts/protocol/oauth.html
- Email Messages API (List/Details): linked from the Index; confirm exact list/search endpoints and filters

## Risks / Trade-offs
- If list endpoint lacks precise time filters, increased client-side filtering cost; mitigate by using search API with narrowed time windows and pagination
- Region/DC misconfiguration can cause token or API failures; mitigate with clear config and logs

## Open Questions
- Confirm exact time filter param names and supported sort order for messages list vs search
- Validate whether thread identifiers are stable across moves/edits
