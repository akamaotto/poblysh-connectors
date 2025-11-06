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
- GIVEN a prior cursor timestamp `T_last` and configured dedupe window `W`
- WHEN `sync` runs
- THEN the connector queries messages with `lastModifiedTime >= (T_last - W)` (or equivalent time-range search), orders by last-modified ascending, deduplicates by `(message_id || lastModifiedTime)`, emits Signals, and returns a new cursor set to the maximum `lastModifiedTime` observed

#### Scenario: First sync establishes baseline
- GIVEN no cursor
- WHEN `sync` runs
- THEN no historical backfill occurs; the connector establishes a baseline cursor at `now()` to start future incremental polling

#### Scenario: Stable Signals produced
- WHEN message changes are observed during polling
- THEN Signals are emitted with kinds: `email_received`, `email_updated`, or `email_deleted`, with normalized fields `{ message_id, thread_id?, folder_id?, from, to, subject, occurred_at, raw }`

#### Scenario: Handles rate limit via standardized error
- WHEN Zoho responds with rate limiting (429) or 5xx
- THEN the connector returns `Err(SyncError::RateLimited { retry_after_secs })` or equivalent typed error without emitting partial results and respecting `Retry-After` if present (e.g., parse `Retry-After: 7` or HTTP-date into seconds and propagate as `retry_after_secs = 7`)

#### Scenario: Handles token expiry with refresh
- WHEN Zoho responds with 401 due to expired access token
- THEN the connector attempts a single refresh using the stored refresh token; if refresh succeeds, the polling request is retried once; otherwise, a typed provider error is returned

#### Scenario: Monotonic cursor advancement
- WHEN `sync` completes successfully
- THEN the returned cursor is greater than or equal to the previous cursor timestamp and reflects the highest processed `lastModifiedTime`
