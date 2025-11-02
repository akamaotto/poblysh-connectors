## 1. Implementation
- [x] 1.1 Add dependencies: `tracing`, `tracing-subscriber`, `tower-http`
- [x] 1.2 Initialize subscriber in `main.rs` with EnvFilter from `POBLYSH_LOG_LEVEL` and formatter from `POBLYSH_LOG_FORMAT`
- [x] 1.3 Add `TraceLayer` to Axum router: log `method`, `path`, `status`, `latency_ms`, `trace_id`, and optional `tenant_id`
- [x] 1.4 Generate/attach `trace_id` per request (span field) and ensure `src/error.rs` reads it into error responses
- [x] 1.5 Redact sensitive headers (`authorization`, `cookie`, `set-cookie`) and avoid logging bodies by default

## 2. Validation
- [x] 2.1 Run locally with `POBLYSH_LOG_LEVEL=info` and confirm JSON lines on startup and per request
- [x] 2.2 Trigger an error and verify the `trace_id` present in both logs and error response
- [x] 2.3 Set `POBLYSH_LOG_FORMAT=pretty` and confirm human-readable output
