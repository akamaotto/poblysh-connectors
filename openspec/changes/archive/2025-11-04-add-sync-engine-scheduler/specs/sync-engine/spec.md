## ADDED Requirements
### Requirement: Interval Scheduler Loop
The sync engine SHALL run a background scheduler tick (default every 60 seconds) that evaluates `connections.status = 'active'`, locks eligible connections, and enqueues incremental `sync_jobs` when due. The scheduler MUST ensure at most one queued or running interval job per connection at a time by enforcing database uniqueness and transactional locking.

#### Scenario: Active connection receives incremental job
- **GIVEN** tenant `T` has an `active` connection `C` to provider `github`
- **AND** the most recent incremental job for `C` finished at 12:00 UTC
- **AND** `metadata.sync.next_run_at = 12:15:00 UTC`
- **AND** no other queued incremental job exists for `C`
- **WHEN** the scheduler tick executes at or after 12:15:00 UTC
- **THEN** it inserts a new row into `sync_jobs` with `job_type = 'incremental'`, `status = 'queued'`, `connection_id = C`, and `scheduled_at` set to the jitter-adjusted next run time computed for `C`
- **AND** it updates `metadata.sync.next_run_at` to the subsequent due time (`> scheduled_at`)

#### Scenario: Duplicate pending job prevented
- **GIVEN** an incremental `sync_job` already exists for connection `C` with `status = 'queued'` or `status = 'running'`
- **WHEN** the scheduler tick executes
- **THEN** it skips enqueuing an additional interval job for `C`

### Requirement: Bootstrap Scheduling
If a connection has never completed an incremental job, the scheduler SHALL enqueue the first incremental job immediately and set `metadata.sync.next_run_at` based on `activation_reference = connections.metadata.sync.first_activated_at` when present, otherwise the connection's `created_at`.

#### Scenario: First incremental job scheduled
- **GIVEN** connection `C` has no prior incremental `sync_job`
- **AND** `metadata.sync.first_activated_at = 2025-01-01T12:00:00Z`
- **WHEN** the scheduler tick evaluates `C`
- **THEN** it enqueues an incremental job for `C` with `scheduled_at <= now() + applied_jitter`
- **AND** it writes `metadata.sync.next_run_at = activation_reference + base_interval + jitter`

### Requirement: Effective Interval Resolution
The scheduler SHALL derive each connection's base interval (seconds) from `connections.metadata.sync.interval_seconds` when present, otherwise use 900 seconds (15 minutes). The computed base interval MUST be added to the last completion time to determine the next run window.

#### Scenario: Metadata overrides default
- **GIVEN** connection `C` has `metadata.sync.interval_seconds = 300`
- **WHEN** the scheduler computes the next run time
- **THEN** it uses 300 seconds as the base interval for `C`

#### Scenario: Default interval applied
- **GIVEN** connection `D` lacks a metadata override
- **WHEN** the scheduler computes the next run time
- **THEN** it uses 900 seconds as the base interval

### Requirement: Positive Jitter Application
The scheduler MUST apply a positive jitter derived from configuration bounds on every scheduled run. For each connection, jitter SHALL be sampled independently from a uniform distribution in the range `[config.jitter_pct_min * base_interval, config.jitter_pct_max * base_interval]` (defaults: 0%–20%). The sampled jitter MUST be added to the base interval before persisting `scheduled_at`, and the scheduler MUST emit debug tracing with the jitter value.

#### Scenario: Jitter within configured bounds
- **GIVEN** the base interval is 900 seconds
- **AND** `config.jitter_pct_min = 0.0` and `config.jitter_pct_max = 0.2`
- **WHEN** the scheduler enqueues the next job
- **THEN** the jitter added is `>= 0` seconds and `<= 180` seconds
- **AND** `scheduled_at = next_due_at + jitter` where `next_due_at` is computed as:
  - **If** `last_run_at + 900 <= now()`: `next_due_at = last_run_at + 900` (normal scheduling)
  - **Else**: `next_due_at` is advanced by multiples of 900 until `next_due_at > now()` for catch-up
- **AND** `metadata.sync.next_run_at = next_due_at + 900` (subsequent run time)
- **AND** tracing output includes the jitter seconds and effective `next_due_at`

#### Scenario: Custom jitter range respected
- **GIVEN** the base interval is 900 seconds
- **AND** `config.jitter_pct_min = 0.05` and `config.jitter_pct_max = 0.3`
- **WHEN** the scheduler enqueues the next job
- **THEN** the jitter added is `>= 45` seconds and `<= 270` seconds
- **AND** the tracing output notes the sampled jitter and configured bounds

### Requirement: Downtime Catch-up
When the scheduler restarts after being offline longer than a connection's base interval, it MUST compute `next_due_at` by repeatedly adding the base interval to the last completion time until `next_due_at > now()`. The scheduler then enqueues exactly one incremental job with `scheduled_at = (next_due_at - base_interval) + jitter` and persists `metadata.sync.next_run_at = next_due_at` to keep future ticks aligned.

#### Scenario: Overdue job scheduled immediately
- **GIVEN** connection `C` last finished an incremental job at 10:00 UTC with a 15-minute interval (900 seconds)
- **AND** the service is down until 10:20 UTC
- **WHEN** the scheduler restarts at 10:20 UTC
- **THEN** it computes `next_due_at = 10:30:00 UTC` (advancing past current time)
- **AND** it enqueues a new incremental job with `scheduled_at = (10:30:00 - 900) + jitter <= 10:20:30 UTC`
- **AND** it sets `metadata.sync.next_run_at = 10:30:00 UTC`
- **AND** the job becomes eligible for workers immediately (since `scheduled_at <= now()`)

#### Scenario: Normal scheduling when current
- **GIVEN** connection `D` last finished at 10:00 UTC with a 15-minute interval
- **AND** the scheduler tick runs at 10:14 UTC (before next due time)
- **WHEN** the scheduler evaluates `D`
- **THEN** it computes `next_due_at = 10:15:00 UTC` (last_run_at + 900, which is <= now())
- **AND** it enqueues a job with `scheduled_at = 10:15:00 + jitter` (future time)
- **AND** it sets `metadata.sync.next_run_at = 10:30:00 UTC`

### Requirement: Scheduler Metadata Contract
Connection metadata SHALL expose a `sync` object containing:
- `interval_seconds` (integer, optional, bounded between 60 and `config.max_overridden_interval_seconds` (default 86400))
- `next_run_at` (timestamp, optional, UTC; persisted by scheduler)
- `last_jitter_seconds` (integer, optional, >= 0)
- `first_activated_at` (timestamp, optional, UTC)

Values outside the allowed range MUST be rejected and replaced with defaults during configuration load. Metadata updates MUST occur atomically with job scheduling.

#### Scenario: Invalid interval override rejected
- **GIVEN** `config.max_overridden_interval_seconds = 3600`
- **AND** connection `C` has `metadata.sync.interval_seconds = 7200`
- **WHEN** the scheduler loads scheduler metadata
- **THEN** it ignores the override, uses the default 900-second interval, and logs a warning referencing the validation failure

### Requirement: Observability Signals
The scheduler SHALL emit tracing spans/fields and metrics with the following identifiers:
- Counter `sync_scheduler_jobs_scheduled_total` with labels `{ provider_slug, tenant_id }`
- Histogram `sync_scheduler_jitter_seconds`
- Gauge `sync_scheduler_backlog_gauge` tracking number of overdue connections
- Histogram `sync_scheduler_tick_duration_ms`

Metrics MUST be recorded on every tick, and tracing logs MUST include `{ jitter_seconds, base_interval_seconds, next_run_at }` per scheduled connection.

#### Scenario: Metrics recorded for scheduled job
- **WHEN** the scheduler enqueues an incremental job
- **THEN** it increments `sync_scheduler_jobs_scheduled_total`, records the jitter value in `sync_scheduler_jitter_seconds`, updates `sync_scheduler_backlog_gauge`, and a tracing span includes the jitter and `next_run_at`

#### Scenario: Jitter persistence validated
- **GIVEN** connection `C` has a 900-second base interval
- **AND** the scheduler computes jitter = 45 seconds for the next run
- **WHEN** the scheduler enqueues the incremental job
- **THEN** it inserts the job with `scheduled_at = next_due_at + 45`
- **AND** it updates `connections.metadata.sync.last_jitter_seconds = 45` in the same transaction
- **AND** it updates `connections.metadata.sync.next_run_at` to the subsequent run time
- **AND** tracing output includes both the applied jitter and the persisted values
- **AND** subsequent scheduler ticks can read the persisted `last_jitter_seconds` for diagnostics

### Requirement: Graceful Shutdown Handling
The scheduler SHALL respect service shutdown signals by cancelling the background tick loop, allowing any in-flight tick to finish its database transaction, and preventing new ticks from starting. Shutdown events MUST be traced so operators can confirm the scheduler exited cleanly without leaving partial work.

#### Scenario: Scheduler stops polling on shutdown
- **GIVEN** the scheduler is mid-way through evaluating due connections
- **AND** a shutdown signal is received
- **WHEN** the scheduler completes the current tick
- **THEN** it commits or rolls back outstanding transactions, emits a shutdown trace event, and refrains from starting any further ticks

### Requirement: Interval Concurrency Guard
The system MUST prevent duplicate interval jobs in multi-instance deployments by combining a Postgres partial unique index on `(connection_id, job_type)` for `status IN ('queued','running')` with `SELECT ... FOR UPDATE SKIP LOCKED` when scanning due connections.

#### Scenario: Parallel schedulers remain race-free
- **GIVEN** two scheduler instances tick simultaneously
- **WHEN** both attempt to enqueue an interval job for the same connection
- **THEN** one obtains the row lock and enqueues the job, while the other observes the uniqueness guard and skips without error
