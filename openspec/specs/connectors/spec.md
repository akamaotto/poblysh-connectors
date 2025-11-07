# connectors Specification

## Purpose
TBD - created by archiving change add-connector-trait-and-registry. Update Purpose after archive.
## Requirements
### Requirement: Connector Trait
The `sync` method signature and semantics SHALL support cursor advancement and pagination.

#### Scenario: Sync returns next cursor and has_more
- **WHEN** `sync(connection, cursor?)` is invoked
- **THEN** it returns a `SyncResult` object with fields:
  - `signals: [Signal]` (normalized events)
  - `next_cursor?: Cursor` (opaque provider cursor for the next call)
  - `has_more: bool` (true if additional pages remain)

#### Scenario: Cursor is opaque and serializable
- **WHEN** `next_cursor` is provided
- **THEN** it is an opaque value serializable to JSON and safe to store under `connections.metadata.sync.cursor`

### Requirement: Provider Metadata Structure
The system SHALL define a provider metadata structure for discovery and documentation.

#### Scenario: Metadata fields
- **WHEN** metadata is retrieved for a provider
- **THEN** it includes `name` (string), `auth_type` (enum string), `scopes` (string array), and `webhooks` (boolean)

### Requirement: In-memory Provider Registry
The system SHALL provide a read-only, in-memory registry mapping provider `name -> { connector, metadata }`.

#### Scenario: Resolve connector by name
- **WHEN** `get(name)` is called for a known provider
- **THEN** the registry returns a handle to the connector instance

#### Scenario: Unknown provider returns error
- **WHEN** `get(name)` is called for an unknown provider
- **THEN** the registry returns a typed error indicating unknown provider

#### Scenario: List provider metadata
- **WHEN** `list_metadata()` is called
- **THEN** the registry returns a list of metadata entries sorted by `name` ascending

### Requirement: Seed Registry With Stub Connector
The system SHALL include at least one stub connector registered to validate wiring.

#### Scenario: Stub connector registration
- **WHEN** the system initializes the registry
- **THEN** at least one provider (e.g., `example`) exists with non-empty metadata and a no-op connector implementation

### Requirement: Provider Metadata (GitHub)
The registry SHALL expose a `github` provider with metadata describing OAuth and webhook support.

#### Scenario: Metadata fields populated
- **WHEN** listing providers
- **THEN** `github` appears with `{ name: "github", auth_type: "oauth2", scopes: ["repo", "read:org"], webhooks: true }`

### Requirement: GitHub OAuth Authorization
The connector SHALL generate a GitHub OAuth authorize URL using configured client credentials and requested scopes.

Details:
- Base: `https://github.com/login/oauth/authorize`
- Params: `client_id`, `redirect_uri`, `scope` (space- or comma-separated), `state`
- State MUST be cryptographically random and bound to `(tenant, provider)` with expiration.

#### Scenario: Returns authorize URL
- **WHEN** `authorize(tenant)` is called
- **THEN** it returns a URL to GitHub including `client_id`, `redirect_uri`, `scope`, and a unique `state`

### Requirement: GitHub Token Exchange And Connection Persistence
The connector SHALL exchange the authorization `code` for tokens and persist a connection with the authenticated user identity.

Details:

- Token endpoint: `POST https://github.com/login/oauth/access_token`
- Headers: `Accept: application/json`
- Response JSON SHALL include:
  - `access_token` – bearer token issued by GitHub (persist exactly as returned)
  - `token_type`
  - `refresh_token` when provided (GitHub may omit for classic OAuth; handle absence)
  - `expires_in` (seconds) when provided. The connector MUST convert this to an absolute `expires_at` in UTC using `expires_at = current_time_utc + expires_in` (per OAuth2 RFC 6749) before persisting. When GitHub omits `expires_in`, the connector SHALL persist `expires_at = null` and document that clients MUST treat the token as non-expiring until a later refresh provides an expiry. Clients MUST apply a safety window of 10–60 seconds when evaluating token freshness (i.e., consider the token expired if `current_time_utc + safety_margin >= expires_at`).
- Metadata stored: `{ user: { id, login }, primary: true (if first github connection for tenant) }`

#### Scenario: Successful exchange creates connection
- **WHEN** `exchange_token(code)` succeeds
- **THEN** a `connections` row exists with `provider_slug='github'`, tokens persisted, and metadata includes user id/login

### Requirement: GitHub Token Refresh
The connector SHALL call `POST https://github.com/login/oauth/access_token` with `grant_type=refresh_token` whenever a stored `refresh_token` exists, and SHALL return the exact token payload GitHub issues so callers can update credentials deterministically. The response MUST include:
- `access_token` – new value issued by GitHub (callers MUST persist and use this value)
- `token_type` – typically `bearer`
- `scope` – scopes GitHub associates with the refreshed token
- `expires_in` OR `expires_at` – absolute or relative expiry; if GitHub omits both, the connector SHALL omit `expires_at` and document that callers MUST treat the token as non-expiring until a subsequent refresh succeeds
- `refresh_token` – the new refresh token when GitHub rotates it; if GitHub omits this field the connector SHALL:
  - return the previous refresh token under `refresh_token`
  - set `refresh_token_status` to `"unchanged"`
  - document that callers MUST retain the existing refresh token and may retry refresh when approaching access token expiry
- `refresh_token_expires_in` – if present from GitHub, surface the numeric value; otherwise omit

If GitHub explicitly indicates the refresh token is invalid or revoked, the connector SHALL return an error and MUST NOT return stale tokens.

#### Scenario: Refresh returns new credentials
- **WHEN** GitHub issues a new access token and refresh token
- **THEN** the connector returns the new `access_token`, `token_type`, `scope`, `expires_in`/`expires_at`, the rotated `refresh_token`, `refresh_token_expires_in` when provided, and sets `refresh_token_status` to `"rotated"`

#### Scenario: Refresh token reused without rotation
- **WHEN** GitHub refresh succeeds but omits `refresh_token`
- **THEN** the connector returns the previous refresh token, surface GitHub’s `access_token`, `token_type`, `scope`, `expires_in`/`expires_at`, and sets `refresh_token_status` to `"unchanged"` so callers keep using the existing refresh token

#### Scenario: No refresh token available
- **WHEN** the stored connection lacks a refresh token (e.g., legacy OAuth apps)
- **THEN** the connector SHALL return `Err(RefreshError::Unsupported)` and document that callers must re-authorize via the OAuth start flow

### Requirement: GitHub Webhook Handling (Issues, Pull Requests)
The connector SHALL handle GitHub webhook payloads for `issues` and `pull_request` event families and emit normalized Signals.

Details:
- Signature: Verify `X-Hub-Signature-256` HMAC (see public webhook verification change)
- Event header: `X-GitHub-Event`
- Signal kinds (MVP): `issue_opened`, `issue_closed`, `issue_reopened`, `pr_opened`, `pr_closed`, `pr_merged`, `issue_comment`, `pr_review`
- Mapping examples:
  - `issues` `opened|closed|reopened` → `issue_*`
  - `pull_request` `opened|closed` (+ `merged=true`) → `pr_opened|pr_closed|pr_merged`
- Connection selection (MVP): choose the tenant’s primary GitHub connection; if none, reject with not found.

#### Scenario: Valid `issues` event produces a Signal
- **WHEN** a signed webhook with `X-GitHub-Event: issues` and action `opened` is received for a tenant
- **THEN** one `issue_opened` Signal is emitted with `occurred_at` from payload and normalized payload fields

#### Scenario: Valid `pull_request` merged produces a Signal
- **WHEN** a signed `pull_request` event with `action=closed` and `pull_request.merged=true`
- **THEN** one `pr_merged` Signal is emitted

### Requirement: GitHub REST Backfill (Issues, Pull Requests)
The connector `sync` SHALL backfill and incrementally fetch issues and pull requests updated since the last cursor.

Details (MVP):
- Scope: items updated for the authenticated user across accessible repositories
- Issues endpoint: `GET https://api.github.com/issues?filter=all&state=all&since={since}`
- PRs endpoint: `GET https://api.github.com/search/issues?q=type:pr+involves:{login}+updated:>={since}` (fallback when needed)
- Pagination: use `Link` headers; page size default 50–100
- Cursor: `{ "since": RFC3339 }` advanced to the max `updated_at` observed
- Rate limiting: when GitHub returns `403`/`429` **with** rate-limit headers (`x-ratelimit-remaining`, `x-ratelimit-reset`, `Retry-After`), the connector MUST surface `SyncError::RateLimited { retry_after_secs }` using the `Retry-After` header when present or the delta between `x-ratelimit-reset` and current time; the connector MUST NOT emit partial results in this case.
- Authentication recovery: when GitHub returns `401 Unauthorized`, the connector MUST attempt exactly one refresh flow using stored credentials (per the token refresh requirement). If refresh succeeds, retry the failing request once; if the refresh fails or the retried request still returns 401, return `SyncError::AuthenticationRequired` (or equivalent) and instruct callers to re-authorize. No further retries or partial results are allowed.
- Permission failures: when GitHub returns `403` **without** rate-limit headers, treat it as a scope/permission error and return `SyncError::PermissionDenied` (or equivalent) with guidance to adjust scopes. Do not retry or emit partial results.
- Transient server errors: when GitHub returns `5xx` (HTTP 500–503), the connector SHALL retry with exponential backoff and jitter (e.g., base 1s, multiplier 2x, random jitter ±20%) up to a configurable maximum (default 3 attempts, configurable up to 5). After exhausting retries, return `SyncError::UpstreamFailure` including diagnostic metadata; no partial results should be emitted.

#### Scenario: Returns Signals and next cursor
- **WHEN** `sync` runs with `cursor.since = T`
- **THEN** it emits Signals for items with `updated_at >= T`, sets `next_cursor.since` to the max `updated_at`, and sets `has_more` when pagination continues

#### Scenario: Handles rate limit via standardized error
- **WHEN** GitHub responds with rate limit exceeded
- **THEN** the connector returns `Err(SyncError::RateLimited { retry_after_secs })` without emitting partial results

#### Scenario: Recovers from expired access token once
- **WHEN** GitHub responds `401 Unauthorized`
- **THEN** the connector attempts a single token refresh, retries the request once, and if the retry still fails it returns `Err(SyncError::AuthenticationRequired)` with no partial results

#### Scenario: Surfaces permission errors without retries
- **WHEN** GitHub responds `403` without rate-limit headers
- **THEN** the connector returns `Err(SyncError::PermissionDenied)` explaining required scopes and emits no partial results

#### Scenario: Retries transient 5xx failures then surfaces error
- **WHEN** GitHub responds with a `5xx` status
- **THEN** the connector retries with exponential backoff (max default 3 attempts, configurable up to 5) using jitter, and if failures persist it returns `Err(SyncError::UpstreamFailure)` without emitting partial results

### Requirement: Provider Metadata (Google Drive)
The registry SHALL expose a `google-drive` provider with metadata describing OAuth and webhook support.

#### Scenario: Metadata fields populated
- **WHEN** listing providers
- **THEN** `google-drive` appears with `{ name: "google-drive", auth_type: "oauth2", scopes: ["https://www.googleapis.com/auth/drive.readonly"], webhooks: true }`

### Requirement: Google Drive OAuth Authorization
The connector SHALL generate a Google OAuth authorize URL using configured client credentials and requested scopes.

Details:
- Base: `https://accounts.google.com/o/oauth2/v2/auth`
- Params: `client_id`, `redirect_uri`, `scope` (space‑delimited), `response_type=code`, `access_type=offline` (to obtain refresh token), `state`
- State MUST be cryptographically random and bound to `(tenant, provider)` with expiration.

#### Scenario: Returns authorize URL
- **WHEN** `authorize(tenant)` is called
- **THEN** it returns a URL to Google including `client_id`, `redirect_uri`, `scope`, `response_type=code`, and a unique `state`

### Requirement: Google Drive Token Exchange And Connection Persistence
The connector SHALL exchange the authorization `code` for tokens and persist a connection with token details and minimal identity in metadata.

Details:
- Token endpoint: `POST https://oauth2.googleapis.com/token`
- Request: `code`, `client_id`, `client_secret`, `redirect_uri`, `grant_type=authorization_code`
- Response: `access_token`, `refresh_token` (when `access_type=offline`), `expires_in`
- Metadata stored: `{ user: { email? }, drive: { enabled_scopes: [...] } }` when available

#### Scenario: Successful exchange creates connection
- **WHEN** `exchange_token(code)` succeeds
- **THEN** a `connections` row exists with `provider_slug='google-drive'`, tokens persisted, and metadata includes any available identity hints

### Requirement: Google Drive Channel Webhook Handling
The connector SHALL handle Google Drive Channel notifications and emit normalized Signals.

Details:
- Notification headers include: `X-Goog-Channel-ID`, `X-Goog-Resource-ID`, `X-Goog-Resource-State`, `X-Goog-Message-Number`
- Bodies MAY be empty; the platform MUST forward these Google headers into `payload.headers.*` using canonicalized, lower‑case names: `x-goog-channel-id`, `x-goog-resource-id`, `x-goog-resource-state`, `x-goog-message-number` (and `x-goog-resource-uri` when present). The `Connector::handle_webhook(payload)` SHALL read headers from `payload.headers`.
- Signal kinds (MVP): `file_created`, `file_updated`, `file_trashed` (reserve `file_moved` for a later change)
- Mapping guidance:
  - `X-Goog-Resource-State: add` → `file_created`
  - `X-Goog-Resource-State: update` → `file_updated`
  - `X-Goog-Resource-State: trash` → `file_trashed`
  - Moves require comparing parent IDs; in MVP, all potential moves SHALL be mapped to `file_updated`. The distinct `file_moved` kind is reserved for a future change once parent comparison is implemented.

#### Scenario: Valid channel update produces a Signal
- **WHEN** a Drive Channel notification with `X-Goog-Resource-State: update` is received
- **THEN** one `file_updated` Signal is emitted with `occurred_at` set to receipt time and headers captured in payload

### Requirement: Polling Fallback Using Drive Changes API
The connector `sync` SHALL use the Drive Changes API to backfill and incrementally fetch file changes since the last cursor.

Details (MVP):
- Endpoint: `GET https://www.googleapis.com/drive/v3/changes?pageSize=100&pageToken={cursor.startPageToken|nextPageToken}`
- Initial cursor: obtain `startPageToken`; for MVP, allow empty cursor to use current `startPageToken` (yields no historical events but establishes a baseline)
- Pagination: follow `nextPageToken` until exhausted
- Cursor: opaque Google tokens; store and return latest `nextPageToken` or a new `startPageToken` when list is done
- Rate limiting: on 403/429, return `SyncError::RateLimited { retry_after_secs }`

#### Scenario: Returns Signals and next cursor
- **WHEN** `sync` runs with a valid page token
- **THEN** it emits Signals for items in that page and returns the `nextPageToken` as cursor (or a new `startPageToken` when complete)

#### Scenario: Handles rate limit via standardized error
- **WHEN** Google responds with rate limiting
- **THEN** the connector returns `Err(SyncError::RateLimited { retry_after_secs })` without emitting partial results

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

### Requirement: Jira Connector
The system SHALL provide a Jira connector implementing OAuth2 authorization, selective webhook handling, and incremental sync for issues while emitting normalized Signals that follow the platform taxonomy.

#### Scenario: OAuth authorize URL includes mandatory parameters
- **WHEN** `authorize(tenant)` is called
- **THEN** the returned URL has host `auth.atlassian.com` and includes `response_type=code`, `client_id` from configuration, `audience=api.atlassian.com`, `prompt=consent`, `access_type=offline`, a non-empty `state`, and a `redirect_uri` that matches an allow-listed callback configured for the tenant

#### Scenario: Token exchange persists refreshable connection
- **WHEN** `exchange_token(code)` succeeds
- **THEN** a `connections` row exists with `provider_slug='jira'`, encrypted access and refresh tokens, token expiry timestamps, the Atlassian cloud/site identifier, and account identity metadata persisted in connection settings

#### Scenario: Webhook filters non-issue events
- **WHEN** `handle_webhook(payload)` receives a non-issue webhook event
- **THEN** it returns an empty signal list and records a trace indicating the event type was ignored

#### Scenario: Webhook emits normalized issue Signals
- **WHEN** `handle_webhook(payload)` receives an issue create or update event
- **THEN** it emits one or more Signals with kinds `issue_created` or `issue_updated`, each containing normalized fields `{ issue_id, issue_key, project_key, summary, status, assignee, url, occurred_at }` and the original payload

#### Scenario: Incremental sync paginates by updated timestamp
- **GIVEN** Jira REST returns multiple pages of issues updated since a cursor
- **WHEN** `sync(connection, cursor?)` is called
- **THEN** it iterates through all pages, emits Signals ordered by `updated` ascending, and advances the stored cursor to the greatest `updated` timestamp processed

#### Scenario: Sync deduplicates against recent webhooks
- **GIVEN** an issue already emitted via webhook since the current cursor
- **WHEN** `sync(connection, cursor?)` is called
- **THEN** it emits at most one Signal per issue per `updated` timestamp, avoiding duplicates across webhook and sync pathways

#### Scenario: Dedupe key generation for consistency
- **WHEN** processing Jira webhook or sync events
- **THEN** dedupe keys SHALL be generated using the format `jira:{signal_kind}:{issue_id}:{updated_timestamp}` where:
  - `signal_kind` is one of `issue_created` or `issue_updated`
  - `issue_id` is the numeric Jira issue identifier
  - `updated_timestamp` is the ISO 8601 timestamp from the issue's `updated` field
- **AND** this ensures consistent deduplication across webhook and sync data sources

### Requirement: Gmail Connector
The system SHALL provide a Gmail connector that supports OAuth2 authorization, Pub/Sub push webhook ingestion with OIDC verification, and incremental synchronization using Gmail History.

#### Scenario: Provider metadata registered
- **WHEN** listing providers
- **THEN** `gmail` appears with `auth_type="oauth2"`, `webhooks=true`, and scopes including `https://www.googleapis.com/auth/gmail.readonly`

#### Scenario: Authorization URL generated
- **WHEN** `authorize(tenant)` is called
- **THEN** a Google OAuth URL is returned with the Gmail scope, `response_type=code`, and `access_type=offline` when supported

#### Scenario: Token exchange persists connection
- **WHEN** `exchange_token(code)` succeeds
- **THEN** a `connections` row is created with `provider_slug='gmail'`, access/refresh tokens, expiry, and stored scopes

#### Scenario: Webhook push verified and acked fast
- **WHEN** a Pub/Sub push is received for Gmail
- **THEN** the request is verified via Google OIDC (issuer, audience, signature). On success, the handler decodes the base64 `data` envelope and responds with `202 Accepted` within one second while enqueueing sync (any 2xx acks Pub/Sub; `202` is our required standard). On verification failure, the handler responds with a non‑2xx to trigger Pub/Sub retry.

#### Scenario: Incremental sync via history
- **WHEN** `sync(connection, cursor?)` runs
- **THEN** it calls `users.history.list` starting from the cursor or payload `historyId`, emits Signals for updates/deletes, and advances the cursor to the highest processed `historyId`

#### Scenario: Idempotent delivery
- **WHEN** duplicate Pub/Sub deliveries occur
- **THEN** the system avoids duplicate work using idempotent enqueueing keyed by Pub/Sub `messageId` or `(connection_id, historyId)`

#### Scenario: Invalid history cursor recovery
- **WHEN** Gmail returns an invalid/too‑old `historyId`
- **THEN** the connector records the condition and initiates a documented bounded re‑sync strategy rather than failing silently. HTTP 404 from `users.history.list` indicates an invalid/expired cursor.

#### Scenario: Connection targeting by email
- **WHEN** the webhook payload `message.data` contains `{ emailAddress, historyId }`
- **THEN** the system resolves the connection by `(tenant, provider='gmail', external_id=emailAddress)` and enqueues a sync job for that connection; if multiple matches exist, respond 409; if none, log and respond 202 with no‑op (or 404 if strict mode is adopted and documented)

#### Scenario: Rate limiting behavior
- **WHEN** Gmail APIs respond with `429` or `403` quota errors (e.g., `rateLimitExceeded`, `userRateLimitExceeded`, `quotaExceeded`, `dailyLimitExceeded`)
- **THEN** the connector returns `SyncError::RateLimited { retry_after_secs }`, preferring the `Retry-After` header when present; the sync engine applies exponential backoff and jitter

