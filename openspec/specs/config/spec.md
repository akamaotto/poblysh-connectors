# config Specification

## Purpose
TBD - created by archiving change add-config-and-env-loading. Update Purpose after archive.
## Requirements
### Requirement: Env Prefix And Layered Loading
The system SHALL load configuration from environment variables using the `POBLYSH_` prefix and support layered `.env` files for local development. Later items in the precedence list override earlier ones.

Load order (first → last): `.env`, `.env.local`, `.env.<profile>`, `.env.<profile>.local`, then process environment.

#### Scenario: Base .env loads
- WHEN `.env` contains `POBLYSH_API_BIND_ADDR=127.0.0.1:3000`
- AND no other overrides are present
- THEN the server uses `127.0.0.1:3000`

#### Scenario: Profile override wins
- GIVEN `.env` sets `POBLYSH_API_BIND_ADDR=127.0.0.1:3000`
- AND `.env.local` sets `POBLYSH_API_BIND_ADDR=0.0.0.0:8081`
- WHEN the service starts
- THEN it binds to `0.0.0.0:8081`

#### Scenario: OS environment has highest precedence
- GIVEN `.env.<profile>.local` sets `POBLYSH_API_BIND_ADDR=0.0.0.0:8082`
- AND the OS environment sets `POBLYSH_API_BIND_ADDR=0.0.0.0:9090`
- WHEN the service starts
- THEN it binds to `0.0.0.0:9090`

### Requirement: Local Profiles
The system SHALL support a `POBLYSH_PROFILE` variable to select local profiles. Valid profiles for MVP are `local` (default) and `test`; additional profiles may be added later (e.g., `dev`, `prod`).

#### Scenario: Default profile is local
- WHEN `POBLYSH_PROFILE` is not set
- THEN the effective profile is `local`

#### Scenario: Profile-specific file loads
- GIVEN `POBLYSH_PROFILE=test`
- AND `.env.test` defines `POBLYSH_LOG_LEVEL=debug`
- WHEN the service starts
- THEN the effective log level is `debug`

### Requirement: Typed Application Config
The system SHALL expose a typed configuration struct (`AppConfig`) sourced from env with sensible defaults for non-critical settings.

Fields (MVP):
- `profile` (string) default `local` from `POBLYSH_PROFILE`
- `api_bind_addr` (string) default `0.0.0.0:8080` from `POBLYSH_API_BIND_ADDR`
- `log_level` (string) default `info` from `POBLYSH_LOG_LEVEL`

#### Scenario: Defaults applied when unset
- WHEN no `POBLYSH_*` env variables are set
- THEN `api_bind_addr` defaults to `0.0.0.0:8080`
- AND `log_level` defaults to `info`

### Requirement: Validation And Fail Fast
The config loader MUST validate required fields and value formats on startup, aggregating errors and exiting with a non-zero code if invalid.

Validation (MVP):
- `api_bind_addr` MUST parse as a valid socket address (host:port)

#### Scenario: Invalid bind address causes startup failure
- GIVEN `POBLYSH_API_BIND_ADDR=not-an-addr`
- WHEN the service starts
- THEN startup fails with an error explaining the invalid address

### Requirement: Redacted Config Logging
The system SHALL log the loaded configuration at debug level with sensitive fields redacted.

#### Scenario: Secrets redacted
- GIVEN the config contains secret-like fields (e.g., `*_KEY`, `*_SECRET`, `*_TOKEN`, `DATABASE_URL` password)
- WHEN debug logging is enabled
- THEN those values are redacted in logs (e.g., `****`)

### Requirement: Logging Format Configuration
The system SHALL support a `POBLYSH_LOG_FORMAT` variable to control log output format.

Details (MVP):
- Accepted values: `json` (default) and `pretty`
- Unknown values MUST fall back to `json`

#### Scenario: Default format is JSON
- WHEN `POBLYSH_LOG_FORMAT` is unset
- THEN logs are emitted in JSON format

#### Scenario: Pretty format selected
- GIVEN `POBLYSH_LOG_FORMAT=pretty`
- WHEN the service starts
- THEN logs are emitted in a human-readable text format

### Requirement: Sync Engine Scheduler Configuration
The system SHALL support configuration for the sync engine scheduler through environment variables prefixed with `POBLYSH_SYNC_SCHEDULER_`.

Configuration fields:
- `TICK_INTERVAL_SECONDS` (integer, default 60) - Interval between scheduler ticks, range 10-300 seconds
- `DEFAULT_INTERVAL_SECONDS` (integer, default 900) - Default sync interval for connections without metadata override, range 60-86400 seconds  
- `JITTER_PCT_MIN` (float, default 0.0) - Minimum jitter percentage (0.0 = no minimum jitter)
- `JITTER_PCT_MAX` (float, default 0.2) - Maximum jitter percentage as fraction of interval (0.2 = 20% of interval)
- `MAX_OVERRIDDEN_INTERVAL_SECONDS` (integer, default 86400) - Maximum allowed interval override in connection metadata

All percentage values must be between 0.0 and 1.0. The jitter range MUST be validated to ensure `JITTER_PCT_MIN <= JITTER_PCT_MAX`.

#### Scenario: Default scheduler configuration
- WHEN no `POBLYSH_SYNC_SCHEDULER_*` variables are set
- THEN the scheduler uses: 60-second tick interval, 900-second default sync interval, 0-20% jitter range, 86400-second max override

#### Scenario: Custom scheduler intervals
- GIVEN `POBLYSH_SYNC_SCHEDULER_TICK_INTERVAL_SECONDS=30`
- AND `POBLYSH_SYNC_SCHEDULER_DEFAULT_INTERVAL_SECONDS=1800`
- WHEN the scheduler starts
- THEN it ticks every 30 seconds and uses 30-minute default sync intervals

#### Scenario: Jitter range customization
- GIVEN `POBLYSH_SYNC_SCHEDULER_JITTER_PCT_MIN=0.05`
- AND `POBLYSH_SYNC_SCHEDULER_JITTER_PCT_MAX=0.3`
- WHEN the scheduler computes jitter for a 900-second interval
- THEN jitter is sampled uniformly from 45-270 seconds (5%-30% of 900)

#### Scenario: Invalid configuration rejected
- GIVEN `POBLYSH_SYNC_SCHEDULER_JITTER_PCT_MAX=1.5` (invalid: > 1.0)
- WHEN the service starts
- THEN startup fails with validation error explaining the invalid percentage range

#### Scenario: Jitter range validation
- GIVEN `POBLYSH_SYNC_SCHEDULER_JITTER_PCT_MIN=0.3`
- AND `POBLYSH_SYNC_SCHEDULER_JITTER_PCT_MAX=0.1` (invalid: min > max)
- WHEN the service starts
- THEN startup fails with validation error about inverted jitter range

#### Scenario: Max override enforcement
- GIVEN `POBLYSH_SYNC_SCHEDULER_MAX_OVERRIDDEN_INTERVAL_SECONDS=3600`
- AND a connection has `metadata.sync.interval_seconds = 7200` (exceeds max)
- WHEN the scheduler loads the connection's metadata
- THEN it ignores the override, uses the default interval, and logs a warning about the validation failure

### Requirement: Scheduler Configuration Validation
The scheduler configuration MUST be validated on startup with the following rules:

Validation rules:
- `TICK_INTERVAL_SECONDS`: Must be >= 10 and <= 300
- `DEFAULT_INTERVAL_SECONDS`: Must be >= 60 and <= `MAX_OVERRIDDEN_INTERVAL_SECONDS`
- `JITTER_PCT_MIN`: Must be >= 0.0 and <= 1.0
- `JITTER_PCT_MAX`: Must be >= 0.0 and <= 1.0
- `JITTER_PCT_MIN` must be <= `JITTER_PCT_MAX`
- `MAX_OVERRIDDEN_INTERVAL_SECONDS`: Must be >= 60 and <= 604800 (7 days)

Invalid configurations MUST cause startup failure with descriptive error messages.

#### Scenario: Tick interval validation
- GIVEN `POBLYSH_SYNC_SCHEDULER_TICK_INTERVAL_SECONDS=5` (below minimum)
- WHEN the service starts
- THEN startup fails with error: "SYNC_SCHEDULER_TICK_INTERVAL_SECONDS must be between 10 and 300 seconds, got 5"

### Requirement: Crypto Key Configuration
The system SHALL require a symmetric crypto key provided via `POBLYSH_CRYPTO_KEY` to encrypt/decrypt tokens at rest using AES‑256‑GCM.

Details:
- `POBLYSH_CRYPTO_KEY` MUST be a base64‑encoded 32‑byte value (256‑bit key) using standard base64 encoding with proper padding.
- Validation MUST check for: missing environment variable, invalid base64 characters, incorrect padding, and exact 32-byte decoded length.
- Startup MUST fail with a clear error if the key is missing or invalid in any profile.
- The key value MUST be treated as a secret and redacted in logs and diagnostics.
- Error messages MUST indicate the specific validation failure without exposing partial key material.

#### Scenario: Missing key causes startup failure
- **GIVEN** `POBLYSH_CRYPTO_KEY` is unset
- **WHEN** the service starts
- **THEN** startup fails with an actionable error indicating the crypto key is required

#### Scenario: Invalid base64 length rejected
- **GIVEN** `POBLYSH_CRYPTO_KEY=Zm9v` (base64 for 3 bytes)
- **WHEN** the service starts
- **THEN** startup fails indicating the key must decode to exactly 32 bytes

#### Scenario: Invalid base64 characters rejected
- **GIVEN** `POBLYSH_CRYPTO_KEY=invalid!@#$%^&*()`
- **WHEN** the service starts
- **THEN** startup fails with a base64 format validation error

#### Scenario: Invalid base64 padding rejected
- **GIVEN** `POBLYSH_CRYPTO_KEY=YWJjZGVm` (missing padding)
- **WHEN** the service starts
- **THEN** startup fails indicating invalid base64 padding

### Requirement: GitHub OAuth Configuration
The system SHALL read GitHub OAuth configuration from environment variables.

Env vars (MVP):
- `GITHUB_CLIENT_ID` (required)
- `GITHUB_CLIENT_SECRET` (required; secret)
- `GITHUB_OAUTH_BASE` (optional; default `https://github.com`)
- `GITHUB_API_BASE` (optional; default `https://api.github.com`)

#### Scenario: Missing client id fails startup
- **WHEN** `POBLYSH_GITHUB_CLIENT_ID` is not set
- **THEN** startup validation fails with a clear error

#### Scenario: Defaults applied for base URLs
- **WHEN** base URLs are not set
- **THEN** the effective values are `https://github.com` and `https://api.github.com`

### Requirement: GitHub Webhook Secret
The system SHALL read the webhook signing secret for GitHub HMAC verification from an environment variable.

Env var:
- `GITHUB_WEBHOOK_SECRET` (required for public webhook mode; optional for operator-protected local mode)

#### Scenario: Public mode requires secret
- **WHEN** public webhook access is enabled
- **THEN** `POBLYSH_WEBHOOK_GITHUB_SECRET` MUST be set, otherwise startup fails with a clear error

