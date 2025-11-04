## ADDED Requirements

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