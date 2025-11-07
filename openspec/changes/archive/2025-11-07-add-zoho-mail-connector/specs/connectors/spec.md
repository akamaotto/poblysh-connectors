## ADDED Requirements

### Requirement: Zoho Mail Connector (Polling + Dedupe Window)
The system SHALL provide a Zoho Mail connector that supports OAuth2 authorization, region-aware Accounts endpoints, and incremental synchronization using polling with a bounded dedupe window.

#### Scenario: Provider metadata registered
- WHEN listing providers
- THEN `zoho-mail` appears with `auth_type="oauth2"`, `webhooks=false`, and scopes including `ZohoMail.messages.READ`

#### Scenario: Authorization URL generated (region-aware)
- WHEN `authorize(tenant)` is called
- THEN a Zoho OAuth URL is returned using the configured Accounts base (`accounts.zoho.{dc}`) with the Mail read scope, `response_type=code`, `access_type=offline`, and a unique `state`

#### Scenario: Token exchange persists connection
- WHEN `exchange_token(code)` succeeds
- THEN a `connections` row is created with `provider_slug='zoho-mail'`, access/refresh tokens, expiry, and stored scopes; metadata includes any available Zoho account/user identifiers

#### Scenario: Incremental polling with dedupe window
- GIVEN a prior cursor timestamp `T_last` and configured dedupe window `W` (from `POBLYSH_ZOHO_MAIL_DEDUPE_WINDOW_SECS`)
- WHEN `sync` runs
- THEN the connector queries messages with `lastModifiedTime >= (T_last - W)` (or equivalent time-range search), orders by last-modified ascending, computes `dedupe_key = hash(message_id || lastModifiedTime)` per message, deduplicates using this `dedupe_key` across overlapping windows, emits Signals, and returns a new cursor set to the maximum `lastModifiedTime` observed

#### Scenario: First sync establishes baseline
- GIVEN no cursor
- WHEN `sync` runs
- THEN no historical backfill occurs; the connector establishes a baseline cursor at `now()` to start future incremental polling

#### Scenario: Stable Signals produced
- WHEN message changes are observed during polling
- THEN Signals are emitted with kinds: `email_received`, `email_updated`, or `email_deleted`, with normalized fields aligned to the normalized email schema:
  - `provider`: `"zoho_mail"`
  - `kind`: one of `email_received`, `email_updated`, `email_deleted`
  - `message_id`: Zoho Mail message identifier
  - `thread_id` (optional): Zoho Mail thread identifier when available
  - `folder_id` (optional): Zoho Mail folder identifier
  - `from`: sender address
  - `to`: primary recipient addresses
  - `subject` (optional)
  - `occurred_at`: RFC3339 UTC timestamp derived from the relevant Zoho Mail time field (e.g., received or last modified)
  - `raw` (optional): minimal provider payload excerpt or identifiers needed for debugging

#### Scenario: Handles rate limit via standardized error
- WHEN Zoho responds with rate limiting (429) or 5xx
- THEN the connector returns `Err(SyncError::RateLimited { retry_after_secs })` without emitting partial results and respecting `Retry-After` if present:
  - IF `Retry-After` is a delta-seconds value (e.g., `7`), propagate `retry_after_secs = 7`
  - IF `Retry-After` is an HTTP-date, compute the delta from current time and propagate as `retry_after_secs`

#### Scenario: Handles token expiry with refresh
- WHEN Zoho responds with 401 due to expired access token
- THEN the connector attempts a single refresh using the stored refresh token; if refresh succeeds, the polling request is retried once; otherwise, a typed provider error (e.g., `SyncError::AuthenticationRequired` or equivalent) is returned and no partial results are emitted

#### Scenario: Webhooks explicitly unsupported
- WHEN `handle_webhook(payload)` is invoked for the Zoho Mail connector in this MVP
- THEN the connector MUST return a typed unsupported error that the API layer maps to the standardized format:
  - Connector error: `SyncError::Unsupported` (or equivalent) with message "WEBHOOKS_NOT_SUPPORTED"
  - API layer maps to HTTP 501 with problem details:
    - HTTP status: `501 Not Implemented`
    - Problem type: `about:blank` (or project standard)
    - Code: `WEBHOOKS_NOT_SUPPORTED`
    - Title: `Webhooks not supported for Zoho Mail connector`
    - Detail: A human-readable explanation indicating that Zoho Mail webhooks are not implemented and polling is required

#### Scenario: Validates required configuration
- WHEN the Zoho Mail connector initializes
- THEN it validates required configuration, including:
  - `POBLYSH_ZOHO_MAIL_DC` is present and maps to a known Zoho Accounts/API base URL
  - `POBLYSH_ZOHO_MAIL_SCOPES` is set or defaults to `ZohoMail.messages.READ`
  - `POBLYSH_ZOHO_MAIL_DEDUPE_WINDOW_SECS` and `POBLYSH_ZOHO_MAIL_HTTP_TIMEOUT_SECS` are valid positive integers
- AND fails fast with a clear configuration error if any required value is invalid or missing
