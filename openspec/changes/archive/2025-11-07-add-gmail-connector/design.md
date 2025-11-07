## Context
Build a Gmail connector with Pub/Sub push ingest, fast ack, and incremental fetch using Gmail History. The service already uses Axum, Tokio, SeaORM, and Utoipa. We’ll add OAuth2 and JWT/OIDC verification to accept public Pub/Sub pushes securely and enqueue background sync.

## Goals / Non-Goals
- Goals:
  - Securely receive Pub/Sub push for Gmail and ack fast
  - Verify OIDC token on push requests (issuer, audience, signature)
  - Decode Gmail push envelope and trigger incremental sync using History API
  - Emit normalized Signals for updated/deleted emails with a monotonic cursor
- Non-Goals:
  - Automating Gmail `watch` lifecycle (to be handled later)
  - Full MIME parsing/attachment handling (MVP keeps metadata light)

## Tech and Crate Selection

Existing crate versions (from Cargo.toml):
- axum = 0.8.6; tokio = 1.48.0; serde = 1.0.217; serde_json = 1.0.138
- tracing = 0.1.41; tracing-subscriber = 0.3.19
- sea-orm = 1.1.17; utoipa = 5.3.1
- reqwest (runtime) = 0.12.x

Selected additions/alignments (runtime deps):
- reqwest = 0.12.9 (default-features=false, features: `json`, `rustls-tls`) — HTTP client for Gmail REST and JWKS
- oauth2 = 5.0.0 (features: `reqwest`, `rustls-tls`) — Authorization code + refresh token flows for Gmail
- jsonwebtoken = 10.1.0 — Verify OIDC JWT on Pub/Sub push
- jwks = 0.4.0 — Retrieve and parse Google JWKS; cache by `kid` with ETag
- base64 = 0.22.1 — Decode Pub/Sub message `data` field
- lru = 0.16.2 — Cache decoded keys and verification results to minimize JWKS fetches

Rationale:
- Keep TLS stack consistent with existing `rustls` usage; avoid `native-tls`.
- Use general OAuth2 and HTTP crates instead of heavy generated clients to keep dependencies small for MVP.
- OIDC verification uses Google public keys (JWKS) and standard RS256 verification; validate `iss`, `aud`, and token lifetime (`exp`/`iat`) as recommended.
- Reuse the platform's sync-engine backoff (no separate backoff crate in the connector).

Docs and References (accessed Nov 2025):
- Gmail Push Notifications: https://developers.google.com/workspace/gmail/api/guides/push
- Gmail History API: https://developers.google.com/workspace/gmail/api/guides/history
- Pub/Sub Push Subscriptions: https://cloud.google.com/pubsub/docs/push
- Authenticating Pub/Sub Push with OIDC: https://cloud.google.com/pubsub/docs/push#authenticating_push

## Flow
1) Pub/Sub pushes HTTPS POST to our public webhook `POST /webhooks/gmail` with an OIDC JWT in `Authorization: Bearer <JWT>` (required when the push subscription is configured with OIDC).
2) Handler verifies OIDC JWT:
   - Fetch JWKS from Google and cache (validate `kid`, signature, `iss` in {`https://accounts.google.com`, `accounts.google.com`}, `aud` equals configured audience exactly, and token not expired; allow small clock skew for `iat`).
   - Optionally verify `email`/`sub` matches the configured service account principal on the subscription.
3) Decode JSON envelope: the outer body contains `message` (with `messageId`, `publishTime`, optional `attributes`, and base64 `data`) and `subscription`. Decode `message.data` → `{ emailAddress, historyId }`.
4) Resolve the connection via `(tenant, provider='gmail', external_id = emailAddress)` and enqueue a sync job using `message.messageId` as the preferred idempotency key (record `subscription` for diagnostics). Fall back to `(connection_id, historyId)` only if `messageId` is unavailable. Respond `202 Accepted` immediately to ack (push acks on any 2xx; `202` is our standard). If verification fails, return a non‑2xx to trigger Pub/Sub retry.
5) Sync worker calls `users.history.list(startHistoryId=cursor_or_payload)`; walk pages, and for new messages affected, call `users.messages.get` as needed to map normalized Signals.
6) Advance cursor to the highest processed `historyId` to ensure monotonic progress.

## Security
- Public webhook path allowed when OIDC verification succeeds; otherwise reject (non‑2xx) to trigger Pub/Sub retry.
- Validate JWT claims: `iss`, `aud`, `exp`, `iat` (clock skew), and `kid`.
- Cache JWKS and revalidate with ETag; refresh asynchronously on `kid` misses.
- Enforce size limits on request body and timeouts to ensure quick ack path.

## Error Handling & Resiliency
- Ack fast: if enqueue succeeds, return 202; on transient verification/JWKS errors, return 429/503 to trigger Pub/Sub retry.
- Gmail rate limits: on 429/5xx or quota 403 responses, surface `SyncError::RateLimited { retry_after_secs }`, honoring `Retry-After` when present; the sync engine applies exponential backoff and jitter.
- Duplicate deliveries are ignored by idempotent job insert (`UNIQUE(connection_id, historyId)` or `dedupe_key`).
- `historyId` invalid (e.g., too old): HTTP 404 from `users.history.list` indicates an invalid/expired cursor; record condition and schedule re‑sync (bounded backfill) with a distinct job type; emit an internal signal for observability.

## Configuration
- `POBLYSH_GMAIL_SCOPES` (default: `https://www.googleapis.com/auth/gmail.readonly`)
- `POBLYSH_PUBSUB_OIDC_AUDIENCE` — expected OIDC `aud` value configured on the push subscription; verification requires an exact match to this value (commonly a service URL or a custom audience string set on the subscription)
- `POBLYSH_PUBSUB_OIDC_ISSUERS` — allowed issuers (default: `accounts.google.com, https://accounts.google.com`)
- `POBLYSH_PUBSUB_MAX_BODY_KB` — request size cap for webhook ingress (default: 256)
- JWKS endpoint: `https://www.googleapis.com/oauth2/v3/certs` (cache by `kid` and ETag)

## Connection Targeting
- During `exchange_token`, persist `connections.external_id = primary email address` so that webhook resolution can map `emailAddress` to the correct connection per tenant/provider.

## Data Model Notes
- Store per‑connection Gmail cursor: last processed `historyId`.
- Optional idempotency table or unique index for `(connection_id, historyId)` enqueued jobs.

## Alternatives Considered
- Generated Google API crates (`google-apis-rs`): richer typing but heavier and slower iteration for MVP; revisit after proving flows.
- Verifying Pub/Sub via IP allowlists: not recommended; OIDC provides stronger auth.

## Risks / Trade-offs
- JWKS fetch latency during first push after scale‑to‑zero; mitigated by pre-warming and caching.
- History backfill cost if cursor is invalid; mitigated by bounded strategy and alerting.

## Migration Plan
- Introduce new connector and OIDC verification in a feature‑flagged public webhook path.
- Gradually enable for environments and tenants after validation.

## Open Questions
- Should we support additional Gmail scopes (labels, modify) for future features?
- Do we need per‑tenant audience values if endpoints differ per environment?
