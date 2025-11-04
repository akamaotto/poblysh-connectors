## 1. Implementation
> **Note**: This is a proposed change awaiting implementation. All tasks represent work to be completed.
- [x] 1.1 Add `SchedulerConfig` (tick interval, default interval seconds, jitter pct range, max override bounds) to application configuration.
- [x] 1.2 Implement a scheduler loop that lists active connections, locks each row (`FOR UPDATE SKIP LOCKED`), and computes the next incremental `sync_job` window from `last_incremental_finished_at` or a bootstrap `activated_at`.
- [x] 1.3 Enqueue incremental sync jobs with `scheduled_at = next_due_at + jitter` where `next_due_at` is derived from the last completion time and advanced past `now()` during catch-up while keeping at most one queued/running interval job per connection (enforced by a partial unique index).
- [x] 1.4 Persist `{ next_run_at, last_jitter_seconds }` in `connections.metadata.sync`, update `sync_jobs` rows, and emit tracing for jitter, chosen interval, and backlog evaluations.
- [x] 1.5 Record metrics counters/gauges (`sync_scheduler_jobs_scheduled_total`, `sync_scheduler_jitter_seconds`, `sync_scheduler_backlog_gauge`, `sync_scheduler_tick_duration_ms`) aligned with tracing labels.
- [x] 1.6 Add a Postgres partial unique index preventing more than one `(connection_id, job_type='incremental', status IN ('queued','running'))` row and extend repository helpers to surface violations as no-op skips.
- [x] 1.7 Extend connection metadata accessors to read/write `metadata.sync.interval_seconds`, `metadata.sync.next_run_at`, and `metadata.sync.last_jitter_seconds` with validation (60–86400 seconds).
- [x] 1.8 Handle shutdown signals gracefully so the scheduler stops polling without leaving partial work (cancellation token + task tracker).

## 2. Validation
- [x] 2.1 Unit tests for jitter generation (bounds, distribution), next-run calculations (bootstrap, steady state, multi-interval catch-up), and metadata persistence.
- [x] 2.2 Integration test that simulates downtime (> interval) and asserts the scheduler enqueues a catch-up job immediately, advances stored `next_run_at` beyond `now()`, and respects the uniqueness guard across concurrent scheduler tasks.

## 3. Notes / Non-goals
- No queue/worker replacement; uses existing Postgres-backed `sync_jobs` table.
- No provider-specific rate limit logic beyond default interval + jitter; per-provider policies can extend later.
