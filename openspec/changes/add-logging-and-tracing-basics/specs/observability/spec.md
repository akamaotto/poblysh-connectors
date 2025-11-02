## ADDED Requirements

### Requirement: Tracing Initialization
The service SHALL initialize structured logging using the `tracing` ecosystem with a JSON formatter and environment-based filtering.

Details (MVP):
- Formatter: JSON output with fields including `timestamp`, `level`, `message`, `target`, and span fields.
- Filtering: Level configured via `POBLYSH_LOG_LEVEL` (e.g., `info`, `debug`, `trace`).
- Startup MUST succeed even if no external exporters are configured.

#### Scenario: JSON log record emitted on startup
- WHEN the service boots with `POBLYSH_LOG_LEVEL=info`
- THEN a log line is emitted in JSON format including keys `level` and `message`

### Requirement: HTTP Request Spans
Incoming HTTP requests MUST be wrapped in `tracing` spans recording method, path, status, and latency.

Fields (MVP):
- `method`, `path`, `status`, `latency_ms`, `trace_id`
- Optional `tenant_id` if present in request context

#### Scenario: Request finished log contains standard fields
- WHEN calling `GET /providers`
- THEN a JSON log is emitted at request completion including `method`, `path`, `status`, and `latency_ms`

### Requirement: Correlation ID (trace_id)
Each request MUST have a correlation identifier (`trace_id`) attached to the span and exposed to handlers. Error responses MUST include the same `trace_id` per the error model.

#### Scenario: Error response `trace_id` matches logs
- GIVEN a request causes a handled error
- WHEN the server responds with an error envelope
- THEN the body includes `trace_id`
- AND a log entry for that request contains the same `trace_id`

### Requirement: Redaction And Sensitive Data Handling
The system MUST redact or omit sensitive values from logs.

Redactions (MVP):
- HTTP headers: `authorization`, `cookie`, `set-cookie`
- Body payloads: tokens and secrets (do not log bodies by default)

#### Scenario: Authorization header not logged
- WHEN calling an endpoint with `Authorization: Bearer secret`
- THEN no log line contains the token value and sensitive headers are omitted or redacted

### Requirement: Local-Friendly Pretty Format (Optional)
The system SHALL support an optional human-readable text format for local debugging, enabled via `POBLYSH_LOG_FORMAT=pretty`. JSON output SHALL remain the default when unspecified.

#### Scenario: Pretty format enabled locally
- GIVEN `POBLYSH_LOG_FORMAT=pretty`
- WHEN the service starts
- THEN logs are human-readable (not JSON) with span context
