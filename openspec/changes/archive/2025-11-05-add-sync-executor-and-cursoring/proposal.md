## Why
- Scheduled jobs exist but are not yet executed; we need a reliable executor to claim, run, and finalize sync work.
- Providers require cursor-based pagination; without persisting cursors per connection, incremental syncs will duplicate or miss data.
- Failures must apply exponential backoff with jitter to avoid hot-loop retries and respect upstream limits.

## What Changes
- Add a sync executor responsible for claiming due `sync_jobs`, invoking the provider `Connector::sync`, persisting emitted Signals, and updating cursors.
- Define cursor persistence at the connection level (`connections.metadata.sync.cursor`) with per-job `cursor` for handoff; write back `next_cursor` on success.
- Implement failure handling with exponential backoff + jitter via `retry_after`, attempts increment, and error recording.
- Introduce worker concurrency and per-connection single-flight to prevent duplicate work.
- Emit tracing and metrics for job lifecycle: claimed, running, succeeded, failed, signals_count, attempt, backoff_seconds.

## Impact
- Specs: `sync-engine` (executor, claiming, backoff, cursor semantics) and `connectors` (modify `sync` result shape to return `next_cursor` and `has_more`).
- Code: background executor workers (Tokio), repository methods for atomic claim/update, connectors `sync` result update, signal insertion pipeline.
- No API surface changes; operational behavior and observability only.
