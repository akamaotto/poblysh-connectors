## Why
We need consistent, structured logs and basic tracing to debug and operate the service. JSON logs with request spans enable correlation in local/test and scale to future exporters. A minimal `tracing` setup with span fields and redaction satisfies MVP observability without introducing external systems.

## What Changes
- Initialize `tracing` with a JSON formatter and environment filter sourced from config (`POBLYSH_LOG_LEVEL`).
- Add HTTP request/response spans (method, path, status, latency) using `tower-http` `TraceLayer`.
- Generate/propagate a `trace_id` per request and include it in logs and error responses (aligning with the error model spec).
- Redact sensitive headers/fields (e.g., `authorization`, `cookie`) and never log token values.
- Optional `POBLYSH_LOG_FORMAT` to toggle `json` (default) vs `pretty` for local debugging.

## Impact
- Affected specs: `observability` (new), `config` (logging format)
- Affected code: `src/main.rs` (init subscriber), `src/server.rs` (TraceLayer), `src/error.rs` (ensure `trace_id` uses current span)
- Dependencies: `tracing`, `tracing-subscriber`, `tracing-error` (optional), `tower-http`

