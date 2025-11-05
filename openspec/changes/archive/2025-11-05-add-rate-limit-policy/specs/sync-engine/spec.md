## ADDED Requirements
### Requirement: Central Rate Limit Policy
The sync engine SHALL implement a centralized policy to handle provider rate limits when executing `sync_jobs`.

Policy inputs and precedence:
- Connector error type `RateLimited { retry_after_secs? }` (if present)
- Provider hints (e.g., HTTP `Retry-After` header parsed by the connector)
- Static backoff formula using `base_seconds`, `max_seconds` and exponential growth by attempts

Computation (MVP):
- Defaults: `base_seconds = 5`, `max_seconds = 900`, `jitter_factor = 0.1`.
- `attempts` is the number of prior failures for the job (increments by 1 only when re-queuing on failure).
- `exp_backoff = min(base_seconds * 2^(attempts), max_seconds)`
- `hint = retry_after_secs` when provided by connector, else `None`
- `effective = max(hint.unwrap_or(0), exp_backoff)`
- Apply jitter: `jitter = uniform(0, jitter_factor * effective)`; `final_backoff = effective + jitter`
- Set job `retry_after = now() + final_backoff`

#### Scenario: Uses provider hint when greater than policy
- **GIVEN** attempts = 2, base = 5 → `exp_backoff = 20`
- **AND** connector returns `RateLimited { retry_after_secs = 60 }`
- **WHEN** computing backoff
- **THEN** `effective = 60` and `retry_after = now() + 60 + jitter`

#### Scenario: Falls back to exponential when no hint
- **GIVEN** attempts = 1, base = 5, max = 900
- **WHEN** computing backoff
- **THEN** `retry_after >= now() + 5` and `< now() + 5 + (jitter_factor * 5)`

### Requirement: Provider Overrides
The policy MUST support provider-specific overrides for `base_seconds`, `max_seconds`, and `jitter_factor` keyed by provider slug.

#### Scenario: Override applied by provider
- **GIVEN** default base = 5, but `github.base_seconds = 10`
- **WHEN** a GitHub job is rate-limited on its 1st retry
- **THEN** `exp_backoff` uses 10 seconds instead of 5

- If no override exists for a provider slug, the policy MUST fall back to the global defaults.

### Requirement: Telemetry For Rate Limits
The system SHALL emit structured logs and metrics when rate limits occur.

Metrics (MVP):
- Counter: `rate_limited_total{provider}` increments on each rate-limit event
- Histogram: `rate_limited_backoff_seconds` records `final_backoff` in seconds.
  - Recommended buckets: exponential buckets covering 1s → 900s (e.g., 1, 2, 4, 8, ..., 512, 900).

#### Scenario: Metrics updated on rate limit
- **WHEN** a rate-limited error occurs
- **THEN** the counter increments and the histogram observes the backoff seconds
