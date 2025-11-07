## Why
Teams rely on Google Calendar for scheduling. We need a first‑class connector that supports push notifications via Google Calendar Channels (watch) and incremental event sync so we can emit normalized Signals for calendar events.

## What Changes
- Add `google-calendar` provider to the registry with metadata: `auth_type = "oauth2"`, `webhooks = true`, and minimal read scopes.
- Implement Google Calendar connector covering the `Connector` trait:
  - `authorize(tenant)` → Google OAuth URL with required scopes and state
  - `exchange_token(code)` → exchange for access/refresh tokens; persist connection metadata
  - `refresh_token(connection)` → standard OAuth2 refresh flow
  - `handle_webhook(payload)` → process Calendar Channel notifications (headers forwarded) and trigger incremental sync
  - `sync(connection, cursor?)` → incremental event sync using `events.list` with `syncToken`
 - Define normalized Signals for calendar events (MVP): `event_updated`, `event_deleted` mapped from sync results. Creation vs update disambiguation is deferred to a later change.

## Impact
- Affected specs: `connectors` (new Google Calendar requirements). Optionally `config` for OAuth client/env vars in a future change.
- Affected code: `src/connectors/google_calendar.rs` (new), registry seeding in `src/connectors/registry.rs`.
- Dependencies: for the MVP stub, none (HTTP flows mocked). For real implementation, use existing crates already in this repo: `reqwest 0.12.x` for HTTP and `oauth2 5.x` for OAuth flows. Avoid generated Google client crates or `yup-oauth2` to keep a consistent stack and minimize dependency surface.

## Non-Goals (MVP)
- Automatic channel lifecycle (create/renew/stop) — manual provisioning expected for MVP; later change can automate.
- Multi‑calendar coverage — MVP focuses on the primary calendar; additional calendars can be added later.
- Signature verification for Google webhooks — covered by the shared webhook verification change.

## Acceptance Criteria
- Provider registry exposes `google-calendar` with OAuth2, scopes `["https://www.googleapis.com/auth/calendar.readonly"]`, and `webhooks=true`.
- `authorize` returns a Google OAuth URL with configured scopes and unique state.
- `exchange_token` persists a `connections` row with `provider_slug='google-calendar'` and token details.
- Webhook ingest accepts Google Calendar Channel notifications and enqueues a sync job for the connection; connector webhook handler may return no direct Signals.
- Incremental sync produces Signals `event_updated`, `event_deleted` since the last cursor, with a stable cursor using `nextSyncToken`.
- `openspec validate add-google-calendar-connector --strict` passes.
