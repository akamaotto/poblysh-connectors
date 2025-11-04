## Why
- Incremental synchronization currently depends on manual triggers or webhooks, leaving most providers without continuous polling.
- Running all connections on the same cadence would create thundering-herd spikes and trip upstream rate limits.
- We need a predictable background scheduler that can recover after downtime while staggering work automatically.

## What Changes
- Introduce a sync engine scheduler service that polls active connections and enqueues incremental `sync_jobs` on a steady cadence, including a bootstrap path for connections that have never run.
- Derive each connection's base interval (default 15 minutes) with an optional per-connection override stored in connection metadata and validate override bounds (60–86400 seconds).
- Apply a 0–20% positive jitter sampled from a uniform distribution to every scheduled run, persist `{ next_run_at, last_jitter_seconds }` in `connections.metadata.sync`, and advance `next_run_at` past `now()` during catch-up so only one interval job is queued at a time.
- Add tracing/metrics hooks (`sync_scheduler_jobs_scheduled_total`, `sync_scheduler_jitter_seconds`, `sync_scheduler_backlog_gauge`, `sync_scheduler_tick_duration_ms`) so operators can observe scheduled runs, jitter applied, backlog depth, and tick health.
- Enforce “at most one pending interval job per connection” with a Postgres partial unique index and transactional `SELECT ... FOR UPDATE SKIP LOCKED` guard so multiple scheduler instances remain safe.

## Impact
- Specs: new `sync-engine` capability detailing bootstrap, interval resolution, jitter persistence, downtime catch-up semantics, observability, and concurrency guard (touches `database.sync_jobs` and `connections.metadata.sync`).
- Code: background scheduler module (Tokio task), configuration struct for scheduler defaults, repository helpers for selecting due connections/jobs, metadata persistence helpers, and coverage tests (first-run, jitter bounds, downtime catch-up, concurrency guard).
- External APIs remain unchanged; only background behavior and observability evolve.
