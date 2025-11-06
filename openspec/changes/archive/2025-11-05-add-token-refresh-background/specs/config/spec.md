## MODIFIED Requirements
### Requirement: Typed Application Config
The configuration SHALL expose a typed `AppConfig` sourced from environment variables and include a nested `TokenRefreshConfig` section controlling background token refresh behavior.

Fields (MVP):
- `profile` (string) default `local` from `POBLYSH_PROFILE`
- `api_bind_addr` (string) default `0.0.0.0:8080` from `POBLYSH_API_BIND_ADDR`
- `log_level` (string) default `info` from `POBLYSH_LOG_LEVEL`
- `refresh` (`TokenRefreshConfig`) defaulted when individual refresh env vars are unset

`TokenRefreshConfig` (MVP):
- `tick_seconds` (integer, default `3600`)
- `lead_time_seconds` (integer, default `600`)
- `concurrency` (integer, default `4`)
- `jitter_factor` (float, default `0.1`, MUST satisfy `0.0 <= jitter_factor <= 1.0`)

Environment variables (local profiles):
- `POBLYSH_TOKEN_REFRESH_TICK_SECONDS`
- `POBLYSH_TOKEN_REFRESH_LEAD_TIME_SECONDS`
- `POBLYSH_TOKEN_REFRESH_CONCURRENCY`
- `POBLYSH_TOKEN_REFRESH_JITTER_FACTOR`

#### Scenario: Defaults applied when unset
- **WHEN** no `POBLYSH_*` env variables are set
- **THEN** the config loader yields `profile=local`, `api_bind_addr=0.0.0.0:8080`, `log_level=info`
- **AND** the refresher uses `tick_seconds=3600`, `lead_time_seconds=600`, `concurrency=4`, `jitter_factor=0.1`

#### Scenario: Invalid refresh jitter factor rejected
- **GIVEN** `POBLYSH_TOKEN_REFRESH_JITTER_FACTOR=1.5`
- **WHEN** the service loads configuration
- **THEN** startup fails with a validation error explaining that `jitter_factor` must be between 0.0 and 1.0
