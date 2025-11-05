## ADDED Requirements
### Requirement: Job Claiming And Concurrency
The executor SHALL claim due jobs atomically and process them with a bounded worker pool. A job is due when `status='queued'`, `scheduled_at <= now()`, and `(retry_after IS NULL OR retry_after <= now())`. The executor MUST prevent multiple jobs from running for the same `connection_id` concurrently.

#### Scenario: Claim ordered by priority and time
- **GIVEN** queued jobs with varying `priority` and `scheduled_at`
- **WHEN** the executor claims jobs
- **THEN** it selects in order of highest `priority` then earliest `scheduled_at`

#### Scenario: Skip jobs not yet eligible due to retry_after
- **GIVEN** a queued job with `retry_after > now()`
- **WHEN** the executor claims jobs
- **THEN** it is not claimed until `retry_after <= now()`

#### Scenario: Single-flight per connection
- **GIVEN** a job `J1` is `running` for `connection_id = C`
- **WHEN** another queued job `J2` exists for the same `connection_id`
- **THEN** the executor does not claim `J2` until `J1` finishes

### Requirement: Execute Connector Sync
The executor SHALL invoke the provider connector's `sync` method with the effective cursor and persist emitted Signals.

Effective cursor resolution:
- Prefer the `cursor` on the job row when present
- Otherwise use the last persisted cursor from `connections.metadata.sync.cursor`

#### Scenario: Signals persisted
- **WHEN** the connector returns signals
- **THEN** they are inserted into the `signals` table with correct `tenant_id`, `provider_slug`, `connection_id`, `occurred_at`, and `payload`

### Requirement: Cursor Persistence
On successful execution, the executor MUST persist the next cursor for the connection at `connections.metadata.sync.cursor` when provided by the connector. The job row MAY also store the cursor for handoff to follow-up jobs.

#### Scenario: Next cursor stored on success
- **GIVEN** the connector returns `next_cursor = { "since": "2024-10-30T00:00:00Z" }`
- **WHEN** the job completes successfully
- **THEN** `connections.metadata.sync.cursor` updates to that value

### Requirement: Pagination Continuation
If the connector indicates more data remains, the executor MUST enqueue a follow-up incremental job immediately with `scheduled_at = now()` and set the job `cursor` to the returned `next_cursor`.

#### Scenario: Follow-up job enqueued on has_more
- **GIVEN** `has_more = true` and `next_cursor` is present
- **WHEN** the job completes
- **THEN** a new `incremental` job is inserted with `scheduled_at <= now()` and `cursor = next_cursor`

### Requirement: Failure Handling With Backoff
On error, the executor SHALL compute backoff using the centralized Rate Limit Policy and requeue the job.

Backoff policy (MVP):
- Apply the central policy defined in the Rate Limit Policy change (defaults: `base_seconds = 5`, `max_seconds = 900`, `jitter_factor = 0.1`).
- Compute `exp_backoff = min(base_seconds * 2^(attempts), max_seconds)` where `attempts` is the number of prior failures for the job.
- Set `retry_after = now() + exp_backoff + jitter` where `jitter` is uniform in `[0, (jitter_factor * exp_backoff)]`.
- Job is updated to `status='queued'`, `retry_after`, `attempts = attempts + 1`, and `error` recorded

#### Scenario: Backoff bounded with jitter
- **GIVEN** `attempts = 3`
- **WHEN** a failure occurs
- **THEN** `retry_after` is set to a time within the computed backoff window and `attempts` increments by 1

### Requirement: Job Finalization
The executor MUST set `status='succeeded'` and `finished_at` on success; on failure it MUST preserve `started_at` and update error details while re-queuing as described.

#### Scenario: Success updates status and timestamps
- **WHEN** a job completes without error
- **THEN** `status='succeeded'`, `started_at` remains set, and `finished_at` is set to `now()`
