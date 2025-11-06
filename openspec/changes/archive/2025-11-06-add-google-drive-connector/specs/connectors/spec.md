## ADDED Requirements

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
