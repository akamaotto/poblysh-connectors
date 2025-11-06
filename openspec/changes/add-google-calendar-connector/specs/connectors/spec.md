## ADDED Requirements

### Requirement: Provider Metadata (Google Calendar)
The registry SHALL expose a `google-calendar` provider with metadata describing OAuth and webhook support.

#### Scenario: Metadata fields populated
- **WHEN** listing providers
- **THEN** `google-calendar` appears with `{ name: "google-calendar", auth_type: "oauth2", scopes: ["https://www.googleapis.com/auth/calendar.readonly"], webhooks: true }`

### Requirement: Google Calendar OAuth Authorization
The connector SHALL generate a Google OAuth authorize URL using configured client credentials and requested scopes.

Details:
- Base: `https://accounts.google.com/o/oauth2/v2/auth`
- Params: `client_id`, `redirect_uri`, `scope` (space‑delimited), `response_type=code`, `access_type=offline` (to obtain refresh token), `state`
- State MUST be cryptographically random and bound to `(tenant, provider)` with expiration.

#### Scenario: Returns authorize URL
- **WHEN** `authorize(tenant)` is called
- **THEN** it returns a URL to Google including `client_id`, `redirect_uri`, `scope`, `response_type=code`, and a unique `state`

### Requirement: Google Calendar Token Exchange And Connection Persistence
The connector SHALL exchange the authorization `code` for tokens and persist a connection with token details and minimal identity in metadata.

Details:
- Token endpoint: `POST https://oauth2.googleapis.com/token`
- Request: `code`, `client_id`, `client_secret`, `redirect_uri`, `grant_type=authorization_code`
- Response: `access_token`, `refresh_token` (when `access_type=offline`), `expires_in`
- Metadata stored: `{ user: { email? }, calendar: { primary_calendar_id? } }` when available

#### Scenario: Successful exchange creates connection
- **WHEN** `exchange_token(code)` succeeds
- **THEN** a `connections` row exists with `provider_slug='google-calendar'`, tokens persisted, and metadata includes any available identity hints

### Requirement: Google Calendar Channel Webhook Handling (Watch)
The connector SHALL handle Google Calendar Channel notifications and treat them as sync triggers. Bodies MAY be empty; required Google headers MUST be forwarded into `payload.headers` by the platform with lower‑case keys so the connector can read them. The connector MAY return zero Signals from the webhook handler; the platform MUST enqueue a sync job for the connection.

Details:
- Notification headers include (case-insensitive; forwarded as lower‑case keys): `x-goog-channel-id`, `x-goog-message-number`, `x-goog-resource-id`, `x-goog-resource-state`, `x-goog-resource-uri`, `x-goog-channel-token`.
- Resource state values commonly include `exists` and `sync`; neither contains event details.
- On receipt, the platform SHALL enqueue a `sync_jobs` row with `job_type='webhook'` referencing `(tenant, provider='google-calendar', connection)`.

#### Scenario: Valid channel notification triggers sync
- **WHEN** a Calendar Channel notification with `x-goog-resource-state: exists` is received and headers are forwarded under `payload.headers`
- **THEN** the connector webhook handler returns an empty list of Signals, and the platform enqueues a sync job for `(tenant, provider='google-calendar', connection)`

### Requirement: Incremental Event Sync Using Events API
The connector `sync` SHALL incrementally fetch calendar event changes since the last cursor using the Google Calendar `events.list` API and emit normalized Signals.

Details (MVP):
- Endpoint: `GET https://www.googleapis.com/calendar/v3/calendars/{calendarId}/events`
- Scope: primary calendar only in MVP (use `calendarId=primary`).
- Initial baseline: when no cursor is present, call with `timeMin=now()`, `singleEvents=true`, `showDeleted=true` to establish a baseline and ignore items; store the returned `nextSyncToken` as the cursor.
- Incremental: when a cursor exists, call with `syncToken={cursor}` and `showDeleted=true`; follow `nextPageToken` until exhausted; return the `nextSyncToken` as the new cursor.
- Mapping (MVP):
  - If `item.status == 'cancelled'` → emit `event_deleted`
  - Otherwise → emit `event_updated` (creation vs update disambiguation deferred; improvements in a later change)
- Rate limiting: on 403/429, return `SyncError::RateLimited { retry_after_secs }`.
- Token invalidation: on HTTP 410 (`Gone`) for a `syncToken`, drop the cursor and re‑establish the baseline on next run.

#### Scenario: Returns Signals and next cursor
- **WHEN** `sync` runs with a valid `syncToken` cursor
- **THEN** it emits Signals for changed items in that window and returns the `nextSyncToken` as the new cursor

#### Scenario: Handles rate limit via standardized error
- **WHEN** Google responds with 429 or a `quotaExceeded` response
- **THEN** the connector returns `Err(SyncError::RateLimited { retry_after_secs })` without emitting partial results

#### Scenario: Handles invalidated sync token
- **WHEN** Google responds HTTP 410 to a `syncToken`
- **THEN** the connector reports a reset condition and the platform clears the cursor so the next run re‑establishes the baseline
