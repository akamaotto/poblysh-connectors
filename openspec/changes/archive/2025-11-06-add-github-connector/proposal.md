## Why
Many tenants rely on GitHub for day-to-day collaboration. We need a first-class GitHub connector that supports OAuth authorization, secure webhook ingestion, and incremental REST backfill to emit normalized Signals for issues and pull requests.

## What Changes
- Add `github` provider to the registry with metadata: `auth_type = "oauth2"`, `webhooks = true`, and minimal scopes.
- Implement GitHub connector covering the `Connector` trait:
  - `authorize(tenant)` → GitHub OAuth URL with required scopes and state
  - `exchange_token(code)` → exchange for access/refresh tokens; persist connection metadata
  - `refresh_token(connection)` → standard OAuth2 refresh flow
  - `handle_webhook(payload)` → verify HMAC signature, parse events, emit Signals
  - `sync(connection, cursor?)` → incremental REST backfill for Issues and Pull Requests
- Add configuration entries for GitHub OAuth and webhooks: client id/secret, webhook secret, base URLs.
- Define normalized Signals for core GitHub events (issues and pull requests) and map webhook/backfill to these kinds.
- Document rate limiting and cursoring behavior; align with central Rate Limit Policy and Sync Executor.

## Impact
- Affected specs: `connectors` (new GitHub connector requirements), `config` (GitHub env vars). Uses existing `api-webhooks` and `auth` behavior (signature verification defined in separate change).
- Affected code: `src/connectors/github.rs` (new), registry seeding, OAuth handlers reuse `authorize`/`exchange_token`, webhook handler dispatch to connector, sync executor invokes `sync` and persists cursors.
- Dependencies: `reqwest = { version = "0.12", features = ["json"] }` for GitHub REST API calls, `oauth2 = { version = "5.0", default-features = false }` for OAuth2 flows with `reqwest` HTTP client, `hmac = "0.12"` and `sha2 = "0.10"` for webhook signature verification, `serde = { version = "1.0", features = ["derive"] }` for payload serialization.

## Non-Goals (MVP)
- Managing GitHub webhook configuration via API (manual configuration is expected for MVP).
- Full event surface coverage (focus on issues and pull requests).
- GraphQL support (REST only in MVP).

## Acceptance Criteria
- OAuth start returns a valid GitHub authorize URL with configured client ID and scopes.
- OAuth callback persists a `connections` row with `provider_slug='github'`, user identity in metadata, and token details.
- Webhooks at `POST /webhooks/github/{tenant}` accept signed events and emit signals for selected event types.
- Backfill sync produces signals for issues and PRs updated since the last cursor, with pagination and rate-limit awareness.
- `openspec validate add-github-connector --strict` passes.

