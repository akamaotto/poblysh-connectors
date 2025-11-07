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
- The platform normalizes headers to lower‑case and strips sensitive/signature headers at ingest time; connectors MUST rely only on `payload.headers` for Google header values.

#### Scenario: Valid channel notification triggers sync
- **WHEN** a Calendar Channel notification with `x-goog-resource-state: exists` is received and headers are forwarded under `payload.headers`
- **THEN** the connector webhook handler returns an empty list of Signals, and the platform enqueues a sync job for `(tenant, provider='google-calendar', connection)`

### Requirement: Incremental Event Sync Using Events API
The connector `sync` SHALL incrementally fetch calendar event changes since the last cursor using the Google Calendar `events.list` API and emit normalized Signals.

Details (MVP):
- Endpoint: `GET https://www.googleapis.com/calendar/v3/calendars/{calendarId}/events`
- Scope: primary calendar only in MVP (use `calendarId=primary`). Keep the same parameter set across baseline and incremental calls to ensure token stability.
- Initial baseline: when no cursor is present, call with `timeMin=now()`, `singleEvents=true`, `showDeleted=true` to establish a token-only baseline and ignore returned items. This is acceptable for MVP because subsequent updates (including to older events) are captured by later incremental syncs. A future enhancement may add a full-baseline mode (no `timeMin`, full pagination) when initial completeness is required.
- Incremental: when a cursor exists, call with `syncToken={cursor}` and `showDeleted=true`; follow `nextPageToken` until exhausted; return the `nextSyncToken` as the new cursor. When using `syncToken`, do not provide any of the following parameters: `iCalUID`, `orderBy`, `privateExtendedProperty`, `q`, `sharedExtendedProperty`, `timeMin`, `timeMax`, `updatedMin`. All other parameters SHOULD remain identical to the initial call to avoid undefined behavior. Keep `singleEvents` consistent across calls.
- Paging defaults: set `maxResults` to a reasonable default (e.g., 250; tunable by platform policy) and iterate using `nextPageToken` until exhausted for both baseline and incremental flows.
- Mapping (MVP):
  - If `item.status == 'cancelled'` → emit `event_deleted`
  - Otherwise → emit `event_updated` (creation vs update disambiguation deferred; improvements in a later change)
- Rate limiting: on 429 responses, or 403 responses with rate/quota reasons (e.g., `rateLimitExceeded`, `userRateLimitExceeded`, `dailyLimitExceeded`, `quotaExceeded`), return `SyncError::RateLimited { retry_after_secs }`. Prefer the `Retry-After` header value when present; otherwise use a sensible default backoff (e.g., 60s) or provider guidance.
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
