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

