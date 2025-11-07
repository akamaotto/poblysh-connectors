## Why
Many tenants track work in Jira Cloud. We need a first‑class Jira connector that supports OAuth authorization, selective webhook ingestion, and incremental REST backfill to emit normalized Signals for issues.

## What Changes
- Add `jira` provider to the registry with metadata: `auth_type = "oauth2"`, `webhooks = true`, and minimal scopes.
- Implement Jira connector covering the `Connector` trait:
  - `authorize(tenant)` → Atlassian OAuth URL with standard params and state
  - `exchange_token(code)` → exchange for access/refresh tokens; persist connection metadata
  - `refresh_token(connection)` → standard OAuth2 refresh flow
  - `handle_webhook(payload)` → filter events to issues; emit normalized Signals
  - `sync(connection, cursor?)` → incremental REST backfill for issues updated since cursor
- Add configuration entries for Jira OAuth and optional webhook secret: client id/secret, base URLs.
- Define normalized Signals for core Jira issue events and map webhook/backfill to these kinds.

## Impact
- Affected specs: `connectors` (new Jira connector requirements), `config` (Jira env vars). Uses existing `api-webhooks` and `auth` behavior (public mode + signature verification defined in separate change).
- Affected code: `src/connectors/jira.rs` (new), registry seeding, OAuth handlers reuse `authorize`/`exchange_token`, webhook handler dispatch to connector, sync executor invokes `sync` with cursors.
- Dependencies: `reqwest` (Jira REST), `oauth2` or manual token exchange later, `serde` payload models later. MVP uses minimal stubs with realistic URLs and shapes.

## Non-Goals (MVP)
- Managing Jira webhook configuration via API (manual configuration is expected for MVP).
- Full event surface coverage (focus on issue events).
- Multi‑tenant app installation flows (single tenant per connection in MVP).

## Acceptance Criteria
- OAuth start returns an Atlassian authorize URL with expected parameters and tenant‑supplied `redirect_uri` and `state`.
- OAuth callback persists a `connections` row with `provider_slug='jira'`, identity in metadata, and token fields set.
 - Webhooks at `POST /webhooks/jira` (tenant via `X-Tenant-Id` header) accept events (operator‑protected MVP) and emit signals for issue events while ignoring unrelated events.
- Backfill sync produces signals for issues updated since the last cursor.
- `openspec validate add-jira-connector --strict` passes.
