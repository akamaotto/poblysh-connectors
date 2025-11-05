## 1. Implementation
- [x] 1.1 Add `ExecutorConfig { tick_ms, concurrency, claim_batch, max_run_seconds, max_items_per_run }`.
- [x] 1.2 Implement atomic claim query: select due jobs (`status='queued'` AND (`retry_after IS NULL OR retry_after <= now()`) AND `scheduled_at <= now()`) ordered by `(priority DESC, scheduled_at ASC)` with row locking; update to `running`, set `started_at`, and bump `attempts`.
- [x] 1.3 Enforce single-flight: do not claim jobs where a `running` job exists for the same `connection_id`.
- [x] 1.4 Implement `run_job(job)` calling `Connector::sync(connection, cursor)`; persist signals; capture `SyncResult { signals, next_cursor, has_more }`.
- [x] 1.5 On success: update `connections.metadata.sync.cursor = next_cursor` (if present); set job `status='succeeded'`, `finished_at=now()`.
- [x] 1.6 On partial (`has_more=true`): enqueue a follow-up incremental job with `scheduled_at=now()` carrying `cursor=next_cursor`.
- [x] 1.7 On failure: compute `backoff = min(base * 2^(attempts-1), max)` + jitter; set `status='queued'`, `retry_after=now()+backoff`, and persist structured `error`.
- [x] 1.8 Add tracing/metrics for lifecycle and counters.

## 2. Validation
- [x] 2.1 Unit tests: claim logic excludes `retry_after > now()`, honors priority/time ordering, and enforces single-flight.
- [x] 2.2 Unit tests: backoff calculator bounds and jitter range.
- [x] 2.3 Integration test: run end-to-end with registry integration and verify executor functionality.

## 3. Notes / Non-goals
- No distributed coordination in MVP; single-process executor with DB row locks.
- Idempotent signal writes are best-effort; exact-once may require future dedupe keys and indices.
