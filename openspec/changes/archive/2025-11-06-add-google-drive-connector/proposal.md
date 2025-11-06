## Why
Teams frequently rely on Google Drive for file collaboration. We need a first‑class connector that supports push notifications via Google Drive Channels (watch) while also providing a reliable polling fallback to ensure eventual consistency.

## What Changes
- Add `google-drive` provider to the registry with metadata: `auth_type = "oauth2"`, `webhooks = true`, and minimal read scopes.
- Implement Google Drive connector covering the `Connector` trait:
  - `authorize(tenant)` → Google OAuth URL with required scopes and state
  - `exchange_token(code)` → exchange for access/refresh tokens; persist connection metadata
  - `refresh_token(connection)` → standard OAuth2 refresh flow
  - `handle_webhook(payload)` → process Drive Channel change notifications and emit Signals
  - `sync(connection, cursor?)` → incremental polling fallback using `changes` list API
- Define normalized Signals for file change events (created, updated, trashed, moved) mapped from both webhook and polling.

## Impact
- Affected specs: `connectors` (new Google Drive requirements). Optionally `config` for OAuth client/env vars in a future change.
- Affected code: `src/connectors/google_drive.rs` (new), registry seeding in `src/connectors/registry.rs`.
- Dependencies: none for MVP stub (real HTTP flows mocked in later changes).
 - Cross-change dependency: This proposal reuses the existing OAuth start/callback endpoints and persistence wiring introduced in the OAuth flows change. Until that change is merged, the connector MAY stub token exchange/persistence for local testing.

## Naming Consistency
- Provider slug introduced by this change is `google-drive`. A future consolidation may align provider names across the Providers API and the in‑memory registry; this change uses `google-drive` consistently in specs and code.

## Non-Goals (MVP)
- Automatic channel creation/renewal lifecycle management (manual configuration expected for MVP; later change can automate).
- Full Drive event surface; MVP focuses on file‑level changes with minimal mapping.
- Signature verification for Google webhooks; to be covered by the shared webhook verification change.

## Acceptance Criteria
- Provider registry exposes `google-drive` with OAuth2, scopes `["https://www.googleapis.com/auth/drive.readonly"]`, and `webhooks=true`.
- `authorize` returns a Google OAuth URL with configured scopes and unique state.
- `exchange_token` persists a `connections` row with `provider_slug='google-drive'` and token details.
- Webhook ingest converts Google Drive Channel notifications into normalized Signals. The platform MUST forward Google headers into `payload.headers.*` using lower‑case names: `x-goog-channel-id`, `x-goog-resource-id`, `x-goog-resource-state`, `x-goog-message-number` (and `x-goog-resource-uri` when present).
- Polling fallback `sync` produces Signals for changes since the last cursor with a stable, monotonic cursor.
- `openspec validate add-google-drive-connector --strict` passes.
