## MODIFIED Requirements
### Requirement: Token Refresh Contract
Connector implementations SHALL provide `refresh_token(connection)` to exchange the refresh token for a new access token and updated expiry.

Return shape (MVP):
- On success: `{ access_token: string, refresh_token?: string, expires_at?: RFC3339 }`
- On error:
  - Permanent (e.g., invalid_grant/revoked): return a typed permanent error (implementation-defined) used to mark connection status accordingly
  - Transient (network/5xx): return transient error; executor may retry later using Rate Limit Policy

#### Scenario: Refresh returns updated tokens
- **WHEN** refresh is successful
- **THEN** the connector returns a new `access_token`, optionally a new `refresh_token`, and an `expires_at` if provided by the provider

#### Scenario: Invalid refresh token is permanent
- **WHEN** the provider returns `invalid_grant`
- **THEN** the connector signals a permanent error so the system can mark the connection status as `error` or `revoked`

### Requirement: Error Mapping For 401 And 5xx
Connectors MUST map provider authentication and availability errors to standardized `SyncError` variants to enable on-demand refresh and retry semantics.

Mappings (MVP):
- Provider `401 Unauthorized` → `SyncError::Unauthorized { message?: string, details?: object }`
- Provider 5xx or network failures → `SyncError::Transient { message: string, details?: object }`
- Provider rate-limit responses → `SyncError::RateLimited { retry_after_secs?: number, details?: object }` (see Rate Limit Policy)

#### Scenario: 401 maps to Unauthorized
- **WHEN** the provider responds with HTTP 401 during a sync or webhook call
- **THEN** the connector returns `Err(SyncError::Unauthorized { ... })`

#### Scenario: 5xx maps to Transient
- **WHEN** the provider responds with HTTP 503 or a network timeout occurs
- **THEN** the connector returns `Err(SyncError::Transient { ... })`

#### Scenario: 429 maps to RateLimited
- **WHEN** the provider responds with HTTP 429 (Too Many Requests)
- **THEN** the connector returns `Err(SyncError::RateLimited { retry_after_secs: <value>, details?: object, message?: string })` so executors can respect the provider-imposed backoff window
