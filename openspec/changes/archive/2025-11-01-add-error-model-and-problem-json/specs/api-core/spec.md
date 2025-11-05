## ADDED Requirements

### Requirement: Unified Error Envelope

The API SHALL return errors using a single JSON envelope with media type `application/problem+json`.

Shape (MVP):

- `code` (string, machine‑readable; e.g., `VALIDATION_FAILED`, `UNAUTHORIZED`, `NOT_FOUND`, `CONFLICT`, `RATE_LIMITED`, `PROVIDER_ERROR`, `INTERNAL_SERVER_ERROR`)
- `message` (string, human‑readable debug message; safe to show to operators)
- `details` (object, optional; key‑value metadata like field errors)
- `retry_after` (integer, optional; seconds until retry is advised)
- `trace_id` (string, optional; correlation ID for logs/traces)

#### Scenario: Validation error returns 400 with details
- GIVEN a request body fails validation for field `name`
- WHEN the handler rejects the request
- THEN the response is HTTP 400 with `Content-Type: application/problem+json`
- AND the body contains `{ code: "VALIDATION_FAILED", message: "...", details: { "name": "..." }, trace_id: "..." }`

#### Scenario: Not found returns 404
- WHEN a resource is not found
- THEN the response is HTTP 404 with `{ code: "NOT_FOUND", message: "..." }`

#### Scenario: Rate limited returns 429 with Retry-After
- WHEN the request exceeds rate limits
- THEN the response is HTTP 429 with `{ code: "RATE_LIMITED", retry_after: N }`
- AND the `Retry-After` header is present with the same value `N`

#### Scenario: Internal error returns 500 with trace id
- WHEN an unexpected error occurs
- THEN the response is HTTP 500 with `{ code: "INTERNAL_SERVER_ERROR", message: "Internal server error", trace_id: "..." }`

### Requirement: Error Mapping
The system SHALL map common error sources to the unified envelope with appropriate HTTP status codes.

Mappings (MVP):
- Validation errors → 400 `VALIDATION_FAILED` with `details` per field
- Auth failures → 401 `UNAUTHORIZED`; forbidden → 403 `FORBIDDEN`
- Not found → 404 `NOT_FOUND`
- Unique constraint violation → 409 `CONFLICT`
- Rate limit triggers → 429 `RATE_LIMITED` with `retry_after` and header
- Provider upstream HTTP errors → 502 `PROVIDER_ERROR` with provider/status metadata in `details`
- Fallback/unknown → 500 `INTERNAL_SERVER_ERROR` with `trace_id`

#### Scenario: Unique violation maps to 409
- GIVEN a Postgres unique key violation occurs while inserting
- WHEN the error is handled
- THEN the response is HTTP 409 with `{ code: "CONFLICT" }`

#### Scenario: Provider error maps to 502
- GIVEN an upstream provider returns HTTP 503
- WHEN the connector path propagates the error
- THEN the API responds with HTTP 502 `{ code: "PROVIDER_ERROR", details: { provider: "github", status: 503 } }`

### Requirement: OpenAPI Error Schema

The error model SHALL be documented in OpenAPI and referenced by endpoints so clients can rely on the shape.

#### Scenario: Schema present in OpenAPI
- **WHEN** fetching `/openapi.json`
- **THEN** a schema for `ApiError` exists
- **AND** endpoints declare `ApiError` for non‑2xx responses where applicable

#### ApiError Schema Definition

- Declare `components.schemas.ApiError` as an object with machine-readable envelope fields.
- Required properties: `code` (string) and `message` (string).
- Optional properties: `status` (integer HTTP status), `detail` (string), `timestamp` (string, date-time), `trace_id` (string), `errors` (array of objects for field-level issues), and `extensions` (object for vendor-specific metadata).
- Additional optional properties MAY be included provided they remain JSON-serializable and backwards compatible.

Example schema definition:

```yaml
components:
  schemas:
    ApiError:
      type: object
      required:
        - code
        - message
      properties:
        code:
          type: string
          description: Machine-readable error identifier.
        message:
          type: string
          description: Human-readable summary suitable for operators.
        status:
          type: integer
          format: int32
          description: HTTP status associated with the problem.
        detail:
          type: string
          description: Additional context or remediation guidance.
        timestamp:
          type: string
          format: date-time
          description: Moment the error was produced.
        trace_id:
          type: string
          description: Correlation identifier for tracing.
        errors:
          type: array
          description: Structured validation issues keyed by field.
          items:
            type: object
        extensions:
          type: object
          additionalProperties: true
          description: Vendor-specific fields following RFC 7807 extension guidance.
```

#### Details Constraints

- Follow RFC 7807 and RFC 9457 guidance: `detail` (and entries inside `details`) SHOULD help clients remediate the issue without exposing implementation internals or sensitive data.
- Use standardized top-level keys for recurring categories:
  - Validation failures: `details` contains dot-notated field paths mapping to machine-readable error codes, e.g., `{ "user.email": "INVALID_FORMAT" }`.
  - Provider errors: nest provider context under `provider`, e.g., `{ "provider": { "name": "github", "status": 503, "error_code": "UPSTREAM_TIMEOUT" } }`.
- Permit additional error-specific keys when necessary, but keep naming lowercase with dot separators for hierarchical data.
- Mark `details` as operator-visible: responses MAY return `details` to API consumers only when the information is safe for public clients; otherwise, include it in logs with `trace_id` linkage and omit sensitive fields from the public response.
- Never include secrets, stack traces, database identifiers, or internal hostnames. Error payloads MUST contain only data the caller is authorized to view.
- Provide high-level guidance strings in `detail` while keeping verbose diagnostics in server logs to minimize information leakage and satisfy least-privilege principles.

#### Reference Pattern

Endpoints SHOULD reference the shared schema for each non-2xx response:

```yaml
paths:
  /example:
    get:
      responses:
        "400":
          description: Bad Request
          content:
            application/problem+json:
              schema:
                $ref: "#/components/schemas/ApiError"
```

### Requirement: Correlation ID Propagation
Error responses SHALL include a `trace_id` correlating with structured logs for the same request.

#### Scenario: Trace ID included
- WHEN an error occurs during a request
- THEN the error body includes `trace_id`
- AND a matching `trace_id` appears in server logs

