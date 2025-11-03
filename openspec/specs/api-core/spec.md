# api-core Specification

## Purpose
This specification defines the core REST surface shared by all Poblysh connectors, detailing the foundational contracts such as error envelopes, authentication headers, and pagination primitives that other capabilities build upon. It targets backend service developers, integration partners, and API reviewers who rely on a stable definition of cross-cutting behaviors. The document aims to preserve long-term compatibility, enforce consistent security posture, and minimize breaking changes by clearly stating the mandatory guarantees for every endpoint within the public connectors API.
## Requirements
### Requirement: Unified Error Envelope
The API SHALL return errors using a single JSON envelope with media type `application/problem+json`.

Shape (MVP):

- `code` (string, machine‑readable; e.g., `VALIDATION_FAILED`, `UNAUTHORIZED`, `NOT_FOUND`, `CONFLICT`, `RATE_LIMITED`, `PROVIDER_ERROR`, `INTERNAL_SERVER_ERROR`)
- `message` (string, human‑readable debug message; safe to show to operators)
- `details` (object, optional; key‑value metadata like field errors)
- `retry_after` (integer, optional; seconds until retry is advised)
- `trace_id` (string, optional; correlation ID for logs/traces)

#### Details Structure
- For validation errors, use a flat or nested object that maps field names to error messages (string), preserving dotted keys when helpful for deeply nested fields.
- For provider errors, include an object `{ "provider": "<name>", "status": <http_code>, "upstream_error": "<message>" }` to surface upstream context.
- All values in `details` MUST be JSON-serializable (strings, numbers, booleans, null, objects, arrays).
- Error messages inside `details` SHALL be human-readable but NOT localized; clients are responsible for mapping stable keys to localized UI text.

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
- WHEN fetching `/openapi.json`
- THEN a schema for `ApiError` exists
- AND endpoints declare `ApiError` for non‑2xx responses where applicable

### Requirement: Correlation ID Propagation
Error responses SHALL include a `trace_id` correlating with structured logs for the same request.

#### Scenario: Trace ID included
- WHEN an error occurs during a request
- THEN the error body includes `trace_id`
- AND a matching `trace_id` appears in server logs

### Requirement: Health Endpoint
The system SHALL expose a liveness endpoint at `GET /healthz` that responds quickly without calling external dependencies.

Response (MVP):
- HTTP 200 with `Content-Type: application/json`
- Body includes `{ "status": "ok", "service": "<service-id>", "version": "<semver>" }`; additional diagnostic fields (e.g., `timestamp`) MAY be present
- No authentication or tenant header required

#### Scenario: Health returns 200 without auth
- WHEN calling `GET /healthz` without any headers
- THEN the response is HTTP 200 with JSON body including `{ "status": "ok", "service": "...", "version": "..." }`

### Requirement: Readiness Endpoint
The system SHALL expose a readiness endpoint at `GET /readyz` that reflects dependency health.

Checks (MVP):
- Database reachable: ability to acquire a connection and run a trivial query (e.g., `SELECT 1`)
- No pending migrations: migrator reports zero pending migrations for the current schema

Responses:
- Ready: HTTP 200 with `Content-Type: application/json` and JSON body
  ```json
  {
    "status": "ready",
    "checks": {
      "database": "ok",
      "migrations": "ok"
    }
  }
  ```
- Each entry in the `checks` object SHALL be either `"ok"` or `"error"` and use the same key names as the defined readiness checks.
- Not ready: HTTP 503 with `Content-Type: application/problem+json` using the unified `ApiError` envelope. The response SHALL set `code` to `SERVICE_UNAVAILABLE`, surface the failing checks inside `details.checks`, and include a human-readable `message`. Example:
  ```json
  {
    "code": "SERVICE_UNAVAILABLE",
    "message": "Database not reachable",
    "details": {
      "checks": {
        "database": "error",
        "migrations": "ok"
      }
    }
  }
  ```
- No authentication or tenant header required

#### Scenario: Ready when DB reachable and no pending migrations
- GIVEN the database is reachable and the migrator reports zero pending migrations
- WHEN calling `GET /readyz`
- THEN the response is HTTP 200 with JSON
  ```json
  {
    "status": "ready",
    "checks": {
      "database": "ok",
      "migrations": "ok"
    }
  }
  ```

#### Scenario: Not ready when DB not reachable
- GIVEN the database is not reachable
- WHEN calling `GET /readyz`
- THEN the response is HTTP 503 with JSON
  ```json
  {
    "code": "SERVICE_UNAVAILABLE",
    "message": "Database not reachable",
    "details": {
      "checks": {
        "database": "error",
        "migrations": "ok"
      }
    }
  }
  ```

#### Scenario: Not ready when pending migrations
- GIVEN there are pending migrations
- WHEN calling `GET /readyz`
- THEN the response is HTTP 503 with JSON
  ```json
  {
    "code": "SERVICE_UNAVAILABLE",
    "message": "Migrations pending",
    "details": {
      "checks": {
        "database": "ok",
        "migrations": "error"
      }
    }
  }
  ```

### Requirement: OpenAPI Documentation for Health and Readiness
The OpenAPI document SHALL describe `GET /healthz` and `GET /readyz` endpoints and mark them as public (no auth required).

#### Scenario: OpenAPI has health and readiness paths
- WHEN fetching `/openapi.json`
- THEN the document contains entries for `/healthz` and `/readyz` with `GET` operations
- AND the operations do not list bearer auth requirements

