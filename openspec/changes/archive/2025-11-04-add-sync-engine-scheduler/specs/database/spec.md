## MODIFIED Requirements
### Requirement: Connection Entity Schema
The `metadata` JSONB column MUST support a `sync` object with the following scheduler fields:
- `interval_seconds` (integer, optional, bounded between 60 and `config.max_overridden_interval_seconds` (default 86400))
- `next_run_at` (TIMESTAMPTZ, optional, UTC)
- `last_jitter_seconds` (integer, optional, >= 0)
- `first_activated_at` (TIMESTAMPTZ, optional, UTC)

Scheduler updates MUST persist `sync.next_run_at` and `sync.last_jitter_seconds` atomically with job scheduling, and values outside the allowed range MUST be rejected.

#### Scenario: Scheduler metadata persisted atomically
- **WHEN** a scheduler tick enqueues an incremental job
- **THEN** the corresponding `connections.metadata.sync` object is updated in the same transaction with the computed `next_run_at` and `last_jitter_seconds`

#### Scenario: Interval override exceeds configuration
- **GIVEN** `config.max_overridden_interval_seconds = 3600`
- **AND** `connections.metadata.sync.interval_seconds = 7200`
- **WHEN** the scheduler persists metadata for the connection
- **THEN** it rejects the override value, stores the default interval instead, and logs a validation warning

### Requirement: Sync Jobs Partial Unique Index
The system MUST create a partial unique index on `sync_jobs` to prevent duplicate interval jobs per connection. The index enforces uniqueness across `(connection_id, job_type)` only for rows with `status IN ('queued', 'running')`.

Index definition:
```sql
CREATE UNIQUE INDEX idx_sync_jobs_connection_type_status 
ON sync_jobs (connection_id, job_type) 
WHERE status IN ('queued', 'running');
```

This index MUST be created in a migration script and ensures that:
- Only one interval job can be queued/running per connection at a time
- Failed/completed jobs don't count toward the uniqueness constraint
- Multiple different job types (e.g., 'incremental', 'full') can coexist for the same connection

#### Scenario: Duplicate interval job prevented by index
- **GIVEN** an incremental sync_job exists for connection `C` with `status = 'queued'`
- **WHEN** the scheduler attempts to enqueue another incremental job for `C`
- **THEN** the database rejects the insert with a unique constraint violation
- **AND** the scheduler treats this as a no-op and continues without error

#### Scenario: Different job types allowed
- **GIVEN** connection `C` has a running incremental sync_job
- **WHEN** the scheduler attempts to enqueue a full sync job for `C`
- **THEN** the insert succeeds because job_type differs ('incremental' vs 'full')
- **AND** both jobs can proceed concurrently

#### Scenario: Completed jobs don't block new scheduling
- **GIVEN** connection `C` has an incremental sync_job with `status = 'completed'`
- **WHEN** the scheduler attempts to enqueue a new incremental job for `C`
- **THEN** the insert succeeds because the completed job doesn't match the index filter
- **AND** the scheduler proceeds with normal scheduling logic
