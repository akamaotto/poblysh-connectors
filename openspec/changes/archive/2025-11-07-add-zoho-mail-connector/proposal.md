## Why
Zoho Mail is a common enterprise email provider for teams using the Zoho suite. To cover Zoho tenants and unlock email-derived Signals, we need a Zoho Mail connector that performs reliable polling with a bounded deduplication window to handle eventual consistency and delayed updates. This MVP focuses on OAuth2, incremental polling, and robust dedupe; webhooks are out of scope for now.

## What Changes
- Add `zoho-mail` provider to the registry with metadata: `auth_type = "oauth2"`, `webhooks = false`, and minimal read scopes for message listing.
- Implement Zoho Mail connector covering the `Connector` trait:
  - `authorize(tenant)` → Zoho Accounts OAuth URL with Zoho Mail read scopes and state
  - `exchange_token(code)` → exchange for access/refresh tokens; persist connection metadata including Zoho account/user identifiers
  - `refresh_token(connection)` → standard OAuth2 refresh flow
  - `sync(connection, cursor?)` → incremental polling using Zoho Mail Email Messages API with a dedupe window: query from `(cursor.timestamp - window)` and deduplicate by message identity + last-modified to avoid duplicates
  - `handle_webhook(payload)` → not supported in MVP (explicit unsupported error)
- Define normalized email Signals (MVP): `email_received`, `email_updated`, and `email_deleted`. Align with existing email connectors by including stable fields like `provider_message_id`, `thread_id` (if exposed), `folder_id`, `from`, `to`, `subject`, `occurred_at` (RFC3339, UTC), and a `raw` subset (provider payload excerpt when helpful).

## Impact
- Affected specs: `connectors` (new Zoho Mail requirements), optional `config` for region/DC and scopes.
- Affected code: `src/connectors/zoho_mail.rs` (new), registry setup in `src/connectors/registry.rs`.
- Dependencies: add HTTP client and OAuth/JWT crates; adopt backoff and rate-limit aware retry (details below and in design.md).

## Non-Goals (MVP)
- Webhook ingestion for Zoho Mail (many flows are polling-only; evaluate later if Zoho provides push for Mail).
- Full MIME parsing, attachments download, message body normalization. MVP focuses on metadata and key identifiers.
- IMAP IDLE real-time streams (API-first for consistency and simpler auth).

## Acceptance Criteria
- Provider registry exposes `zoho-mail` with OAuth2, scopes including minimal read scopes for message listing, and `webhooks=false`.
- `authorize` returns a Zoho OAuth URL with configured scopes, region-correct Accounts base URL, and unique state bound to `(tenant, provider)`.
- `exchange_token` persists a `connections` row with `provider_slug='zoho-mail'`, access/refresh tokens, expiry, and stored scopes; metadata includes Zoho account/user identifiers when available.
- `sync` performs incremental polling using a time-based cursor and dedupe window:
  - First sync with empty cursor establishes a baseline at `now()` without backfilling historical messages.
  - Subsequent syncs query messages with `lastModifiedTime >= (cursor.timestamp - window)` (or equivalent time-range search) ordered ascending by last-modified; deduplicate using `(message_id || lastModifiedTime)` and emit Signals.
  - Returns `next cursor` with the highest processed `lastModifiedTime` (RFC3339) to ensure monotonic progress.
- Rate limiting and transient errors are handled with exponential backoff honoring `Retry-After`; 401/invalid token triggers a single refresh attempt before surfacing a typed error.
- Dedupe window default is 5 minutes and configurable via env; dedupe is idempotent across overlapping windows.
- `openspec validate add-zoho-mail-connector --strict` passes.

## Core Technologies and Versions
- From Cargo.toml (existing):
  - axum `0.8.6`, tokio `1.48.0`
  - serde `1.0.217`, serde_json `1.0.138`
  - tracing `0.1.41`, tracing-subscriber `0.3.19`
  - url `2.5.4`, chrono `0.4.38`, uuid `1.11.0`
  - sea-orm `1.1.17` (+ migrator), anyhow `1.0.95`, thiserror `2.0.11`
  - tower `0.5.1`
- New/Promoted runtime crates:
  - reqwest `0.12.9` (promote from dev-deps; features: `json`, `rustls-tls`) — HTTP client for Zoho Mail REST and OAuth tokens
  - oauth2 `5.0.0` — Authorization Code + Refresh Token flows (Zoho Accounts)
  - backoff `0.4.0` — Exponential backoff with jitter for 429/5xx
  - base64 `0.22.1` — Only if specific endpoints return base64-encoded fields (optional)
  - lru `0.16.2` — Optional small caches (scopes, folder map) if needed during polling

Rationale:
- Keep TLS via `rustls` to align with existing runtime; avoid `native-tls`.
- Use general OAuth2 + HTTP client crates for small surface area and faster iteration.
- Use time-based incremental polling with a dedupe window to absorb eventual consistency and out-of-order updates without complex state machines.

## Research Plan (Lightweight Deep Research Algorithm)
Goal: confirm the exact Zoho Mail endpoints, query parameters for time-range incremental fetch, identity fields for dedupe, OAuth scopes, and rate limits; gather best practices from docs and community.

1) Parallel discovery (run concurrently)
   - Web docs: search "Zoho Mail API Email Messages list", "Zoho Mail lastModifiedTime", "Zoho Mail search time range", "Zoho Mail rate limit 429"
   - OAuth docs: "Zoho OAuth 2.0 accounts endpoints", "Zoho DC region base URL (US/EU/IN)"
   - Community: StackOverflow, Zoho Community, GitHub issues for polling patterns, pagination, and dedupe guidance
   - Codebase scan: `rg -n "zoho|mail|poll|cursor|dedupe|window"` to reuse cursoring and dedupe patterns; inspect `connectors` and sync engine tasks

2) Sequential reinforcement (narrow and verify)
   - Identify primary list/search endpoint for messages; prefer server-side filters for `lastModifiedTime >= T` if available; otherwise use search query with time range
   - Confirm identity fields: provider `id`, thread id, RFC822 `Message-Id`, `lastModifiedTime`; pick a stable `dedupe_key`
   - Validate pagination and sort order; prefer ascending by last-modified for monotonic cursoring
   - Check scopes required for read-only listing (e.g., `ZohoMail.messages.READ`); minimize scope footprint
   - Extract rate limit behaviors (`Retry-After`, headers) and recommended backoff; cross-check with multiple sources

3) Synthesis and decisions
   - Choose the exact endpoint + params for incremental polling and document examples
   - Define the dedupe window and idempotency strategy (key shape, window default, and configuration)
   - Finalize OAuth and region handling (Accounts base URL per DC)
   - Capture errors: token expiry, 429/5xx, invalid cursor; specify behavior and retries

## Docs and References (to confirm during implementation)
- Zoho Mail API Index: https://www.zoho.com/mail/help/api/
- Zoho OAuth 2.0: https://www.zoho.com/accounts/protocol/oauth.html
- Email Messages API (List/Details): link from API Index (verify region/DC specifics)

## Security
- OAuth2 Authorization Code with refresh tokens; tokens stored encrypted at rest (uses project crypto facilities).
- Region-aware Accounts domain selection (e.g., `accounts.zoho.com`, `accounts.zoho.eu`); base API host similarly regioned.
- Enforce conservative HTTP timeouts and body limits during sync.

## Error Handling & Resiliency
- Retry 429/5xx with exponential backoff and jitter; honor `Retry-After` when present; do not partially emit.
- On 401/invalid token, attempt a single refresh; if refresh fails, surface a typed provider error.
- Handle invalid cursor (e.g., out-of-range time) by bumping the dedupe window or resetting to a safe baseline with a diagnostic Signal.

## Configuration
- `POBLYSH_ZOHO_MAIL_DC` — Zoho Mail data center/region (e.g., `us`, `eu`, `in`, `au`, `jp`, `ca`, `sa`, `uk`) to select Accounts/API base URLs via a resolver mapping
- `POBLYSH_ZOHO_MAIL_SCOPES` — space/comma-delimited scopes (default: `ZohoMail.messages.READ`)
- `POBLYSH_ZOHO_MAIL_DEDUPE_WINDOW_SECS` — dedupe window in seconds (default: `300`)
- `POBLYSH_ZOHO_MAIL_HTTP_TIMEOUT_SECS` — Zoho Mail HTTP client timeout for polling (default: `15`)

## Data Model Notes
- Per-connection cursor stores `last_processed_ts` as RFC3339 string (UTC); on each successful sync, advance to the max seen `lastModifiedTime`.
- Dedupe uses deterministic `dedupe_key = hash(message_id || lastModifiedTime)`; store on emitted Signals to enable idempotent inserts.

## Alternatives Considered
- IMAP IDLE or IMAP polling: richer body access but heavier auth and state; defer to API-first approach for consistency across providers.
- Generated client libraries: prefer hand-rolled `reqwest` for MVP to reduce dependencies.

## Risks / Trade-offs
- Region handling complexity: mitigate by centralized base-URL resolver and clear config.
- Incomplete time filters on list endpoints: fallback to search-based time range; validate in research step.
- Eventual consistency may cause missing edits outside the dedupe window: monitor and tune the window; add guardrails in ops docs.

## Migration Plan
- Introduce connector behind feature flag; gated rollout per environment/tenant.
- Validate polling behavior against a test mailbox and tune window size.

## Open Questions
- Confirm exact message list/search filters for time-based incremental fetch and recommended server-side sort.
- Validate whether thread/group identifiers exist and are stable for dedupe enrichment.
