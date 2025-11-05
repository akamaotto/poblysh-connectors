## Why
We need consistent, structured logs and basic tracing to debug and operate the service. JSON logs with request spans enable correlation in local/test and scale to future exporters. A minimal `tracing` setup with span fields and redaction satisfies MVP observability without introducing external systems.

## What Changes
- Initialize `tracing` with a JSON formatter and environment filter sourced from config (`POBLYSH_LOG_LEVEL`).
- Add HTTP request/response spans (method, path, status, latency) using `tower-http` `TraceLayer`.
- Bridge existing `log::` macros into the structured pipeline via `tracing-log::LogTracer` so legacy callsites keep working.
- Generate/propagate a `trace_id` per request, store it in request extensions for handler access, and include it in logs and error responses (aligning with the error model spec).
- Redact sensitive headers/fields (e.g., `authorization`, `cookie`) and never log token values.
- Optional `POBLYSH_LOG_FORMAT` to toggle `json` (default) vs `pretty` for local debugging.

## Impact
- Affected specs: `observability` (new), `config` (logging format)
- Affected code: `src/main.rs` (init subscriber), `src/server.rs` (TraceLayer), `src/error.rs` (ensure `trace_id` uses current span)
- Dependencies: `tracing`, `tracing-subscriber` (with `env-filter`, `json`), `tracing-log`, `tracing-error` (optional), `tower-http`

## Research Plan
- Run parallel searches on docs.rs for `tower-http::trace::TraceLayer`, `tracing-subscriber` JSON formatter usage, and `tracing-log::LogTracer`, while querying crates.io for compatible versions.
- Reinforce with sequential deep dives: review repository usage of `log::` macros (`rg 'log::' src`) and `trace_id` handling (`rg 'trace_id'`) to map required migration points.
- Capture best practices from official tracing documentation and community discussions on header redaction to inform implementation details and future-proofing.
