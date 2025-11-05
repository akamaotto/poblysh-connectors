## ADDED Requirements

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
