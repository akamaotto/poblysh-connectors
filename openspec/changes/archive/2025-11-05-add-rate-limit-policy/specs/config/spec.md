## MODIFIED Requirements
### Requirement: Typed Application Config
The system SHALL expose a typed configuration struct (`AppConfig`) sourced from env with sensible defaults for non-critical settings and include a `RateLimitPolicy` section for backoff control.

Fields (MVP):
- `profile` (string) default `local` from `POBLYSH_PROFILE`
- `api_bind_addr` (string) default `0.0.0.0:8080` from `POBLYSH_API_BIND_ADDR`
- `log_level` (string) default `info` from `POBLYSH_LOG_LEVEL`
- `ratelimit` (object) with:
  - `base_seconds` (integer, default `5`)
  - `max_seconds` (integer, default `900`)
  - `jitter_factor` (float, default `0.1`)
  - `provider_overrides` (map keyed by provider slug) with optional fields per provider:
    - `base_seconds?` (integer)
    - `max_seconds?` (integer)
    - `jitter_factor?` (float)

Environment variables (local profiles):
- `POBLYSH_PROFILE` → `profile`
- `POBLYSH_API_BIND_ADDR` → `api_bind_addr`
- `POBLYSH_LOG_LEVEL` → `log_level`
- `POBLYSH_RATELIMIT_BASE_SECONDS` → `ratelimit.base_seconds`
- `POBLYSH_RATELIMIT_MAX_SECONDS` → `ratelimit.max_seconds`
- `POBLYSH_RATELIMIT_JITTER_FACTOR` → `ratelimit.jitter_factor`
- Provider overrides MAY be supplied as `POBLYSH_RATELIMIT_<PROVIDER>_BASE_SECONDS`, `..._MAX_SECONDS`, `..._JITTER_FACTOR` where `<PROVIDER>` is the provider slug uppercased with hyphens replaced by underscores (e.g., `google-drive` → `GOOGLE_DRIVE`, giving `POBLYSH_RATELIMIT_GOOGLE_DRIVE_BASE_SECONDS=10`).

#### Scenario: Defaults applied when unset
- **WHEN** no relevant `POBLYSH_*` env variables are set
- **THEN** `profile=local`, `api_bind_addr=0.0.0.0:8080`, `log_level=info`, and the rate-limit policy uses `base_seconds=5`, `max_seconds=900`, `jitter_factor=0.1`

#### Scenario: Provider override takes precedence
- **GIVEN** `POBLYSH_RATELIMIT_GITHUB_BASE_SECONDS=10`
- **WHEN** computing backoff for GitHub
- **THEN** the base seconds is 10 for that provider

#### Scenario: Unknown provider falls back to defaults
- **WHEN** computing backoff for a provider without overrides
- **THEN** the global defaults are used
