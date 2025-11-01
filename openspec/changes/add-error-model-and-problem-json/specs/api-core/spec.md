## ADDED Requirements

### Requirement: Unified Error Envelope
The API SHALL return errors using a single JSON envelope with media type `application/problem+json`.

Shape (MVP):
- `code` (string, machine‑readable; e.g., `validation_failed`, `unauthorized`, `not_found`, `conflict`, `rate_limited`, `provider_error`, `internal_error`)
- `message` (string, human‑readable debug message; safe to show to operators)
- `details` (object, optional; key‑value metadata like field errors)
- `retry_after` (integer, optional; seconds until retry is advised)
- `trace_id` (string, optional; correlation ID for logs/traces)

#### Scenario: Validation error returns 400 with details
- GIVEN a request body fails validation for field `name`
- WHEN the handler rejects the request
- THEN the response is HTTP 400 with `Content-Type: application/problem+json`
- AND the body contains `{ code: "validation_failed", message: "...", details: { "name": "..." }, trace_id: "..." }`

#### Scenario: Not found returns 404
- WHEN a resource is not found
- THEN the response is HTTP 404 with `{ code: "not_found", message: "..." }`

#### Scenario: Rate limited returns 429 with Retry-After
- WHEN the request exceeds rate limits
- THEN the response is HTTP 429 with `{ code: "rate_limited", retry_after: N }`
- AND the `Retry-After` header is present with the same value `N`

#### Scenario: Internal error returns 500 with trace id
- WHEN an unexpected error occurs
- THEN the response is HTTP 500 with `{ code: "internal_error", message: "Internal server error", trace_id: "..." }`

### Requirement: Error Mapping
The system SHALL map common error sources to the unified envelope with appropriate HTTP status codes.

Mappings (MVP):
- Validation errors → 400 `validation_failed` with `details` per field
- Auth failures → 401 `unauthorized`; forbidden → 403 `forbidden`
- Not found → 404 `not_found`
- Unique constraint violation → 409 `conflict`
- Rate limit triggers → 429 `rate_limited` with `retry_after` and header
- Provider upstream HTTP errors → 502 `provider_error` with provider/status metadata in `details`
- Fallback/unknown → 500 `internal_error` with `trace_id`

#### Scenario: Unique violation maps to 409
- GIVEN a Postgres unique key violation occurs while inserting
- WHEN the error is handled
- THEN the response is HTTP 409 with `{ code: "conflict" }`

#### Scenario: Provider error maps to 502
- GIVEN an upstream provider returns HTTP 503
- WHEN the connector path propagates the error
- THEN the API responds with HTTP 502 `{ code: "provider_error", details: { provider: "github", status: 503 } }`

### Requirement: OpenAPI Error Schema
The error model SHALL be documented in OpenAPI and referenced by endpoints so clients can rely on the shape.

#### Scenario: Schema present in OpenAPI
- WHEN fetching `/openapi.json`
- THEN a schema for `ApiError` exists
- AND endpoints declare `ApiError` for non‑2xx responses where applicable

### Requirement: Correlation ID Propagation
Error responses SHOULD include a `trace_id` correlating with structured logs for the same request.

#### Scenario: Trace ID included
- WHEN an error occurs during a request
- THEN the error body includes `trace_id`
- AND a matching `trace_id` appears in server logs

