## 1. Implementation
- [x] 1.1 Add dependencies: `tracing`, `tracing-subscriber = { version = "0.3.19", features = ["env-filter", "json"] }`, `tracing-log`, `tower-http`
- [x] 1.2 Initialize subscriber in `main.rs` with EnvFilter from `POBLYSH_LOG_LEVEL`, formatter from `POBLYSH_LOG_FORMAT`, and register `tracing_log::LogTracer` so `log::` macros emit structured events
- [x] 1.3 Add `TraceLayer` to Axum router: log `method`, `path`, `status`, `latency_ms`, `trace_id`, and optional `tenant_id`
- [x] 1.4 Introduce middleware/util to generate a per-request `trace_id`, record it on the span, and store a `TraceContext` in `Request::extensions()` for downstream access
- [x] 1.5 Update `src/error.rs` (and other handlers) to read `trace_id` from the shared `TraceContext` extension before falling back to generated IDs
- [x] 1.6 Redact sensitive headers (`authorization`, `cookie`, `set-cookie`) and avoid logging bodies by default

## 2. Validation
- [x] 2.1 Run locally with `POBLYSH_LOG_LEVEL=info` and confirm JSON lines on startup and per request
- [x] 2.2 Trigger an error and verify the `trace_id` present in both logs and error response, matching the extension value
- [x] 2.3 Confirm a seed or database log emitted via `log::` macros appears as structured JSON with the registered subscriber
- [x] 2.4 Set `POBLYSH_LOG_FORMAT=pretty` and confirm human-readable output
