## ADDED Requirements
### Requirement: Token Refresh Service
The system SHALL run a background token refresh service that periodically scans active connections and refreshes tokens nearing expiry.

Scheduling and selection (MVP):
- Tick interval default: 3600 seconds (configurable via `TokenRefreshConfig.tick_seconds`)
- Lead time default: 600 seconds before `expires_at` (configurable via `TokenRefreshConfig.lead_time_seconds`)
- Select active connections where `expires_at IS NOT NULL` AND `expires_at <= now() + lead_time`
- Apply per-connection positive jitter in `[0, jitter_factor * tick_seconds]` when scheduling refreshes within each tick to avoid bursts
- Limit concurrent refresh operations to `concurrency` (default 4), global across providers
- Enforce single-flight: at most one refresh may run per `connection_id` at a time

#### Scenario: Connection nearing expiry gets refreshed
- **GIVEN** a connection with `expires_at = now() + 5 minutes`
- **WHEN** the refresh tick runs with `lead_time_seconds = 600`
- **THEN** the system invokes connector `refresh_token` and updates the connection tokens and `expires_at`

#### Scenario: Long-lived tokens are skipped
- **GIVEN** a connection with `expires_at IS NULL`
- **WHEN** the refresh tick runs
- **THEN** the connection is not refreshed by the background service

### Requirement: On-demand Refresh On 401
When a provider returns 401 Unauthorized during connector operations (sync or webhook), the system MUST attempt a token refresh and retry the operation once.

Behavior:
- Detect `SyncError::Unauthorized` (provider 401) from connector error mapping
- Call `refresh_token` for the connection; on success, persist updated tokens and `expires_at`
- Retry the original operation exactly once; if it fails again with 401, propagate error without further retries

#### Scenario: Refresh then retry succeeds
- **WHEN** sync returns 401 for a connection with an expired token
- **THEN** the system refreshes the token, persists it, retries the sync, and succeeds

#### Scenario: Refresh fails permanently
- **WHEN** refresh returns a permanent error (e.g., invalid_grant)
- **THEN** the connection `status` is set to `error` (or `revoked` if determinable), and the original operation fails without retry

### Requirement: Metrics And Tracing (Refresh)
The system SHALL emit metrics and logs for refresh behavior.

Metrics (MVP):
- Counter: `token_refresh_attempt_total{provider}` increments per attempt
- Counter: `token_refresh_success_total{provider}` and `token_refresh_failure_total{provider}`
- Histogram: `token_refresh_latency_seconds` duration of refresh calls
- Gauge: `connections_expiring_soon{provider}` number of connections within lead time window; avoid per-tenant labels to limit cardinality

#### Scenario: Metrics updated on refresh
- **WHEN** a refresh attempt occurs
- **THEN** counters and latency histogram update accordingly
