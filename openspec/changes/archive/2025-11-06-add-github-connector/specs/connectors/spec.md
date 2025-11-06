## ADDED Requirements

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

