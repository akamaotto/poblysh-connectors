## ADDED Requirements

### Requirement: Health Endpoint
The system SHALL expose a liveness endpoint at `GET /healthz` that responds quickly without calling external dependencies.

Response (MVP):
- HTTP 200 with `Content-Type: application/json`
- Body includes `{ "status": "ok" }` and MAY include `service` and `version` fields
- No authentication or tenant header required

#### Scenario: Health returns 200 without auth
- WHEN calling `GET /healthz` without any headers
- THEN the response is HTTP 200 with JSON body including `{ "status": "ok" }`

### Requirement: Readiness Endpoint
The system SHALL expose a readiness endpoint at `GET /readyz` that reflects dependency health.

Checks (MVP):
- Database reachable: ability to acquire a connection and run a trivial query (e.g., `SELECT 1`)
- No pending migrations: migrator reports zero pending migrations for the current schema

Responses:
- Ready: HTTP 200 with JSON `{ "status": "ready", "checks": { "database": "ok", "migrations": "ok" } }`
- Not ready: HTTP 503 with JSON `{ "status": "degraded", "checks": { ... }, "message": "..." }`
- No authentication or tenant header required

#### Scenario: Ready when DB reachable and no pending migrations
- GIVEN the database is reachable and the migrator reports zero pending migrations
- WHEN calling `GET /readyz`
- THEN the response is HTTP 200 with JSON indicating all checks are `ok`

#### Scenario: Not ready when DB not reachable
- GIVEN the database is not reachable
- WHEN calling `GET /readyz`
- THEN the response is HTTP 503 with JSON indicating the database check failed

#### Scenario: Not ready when pending migrations
- GIVEN there are pending migrations
- WHEN calling `GET /readyz`
- THEN the response is HTTP 503 with JSON indicating migrations are pending

### Requirement: OpenAPI Documentation for Health and Readiness
The OpenAPI document SHALL describe `GET /healthz` and `GET /readyz` endpoints and mark them as public (no auth required).

#### Scenario: OpenAPI has health and readiness paths
- WHEN fetching `/openapi.json`
- THEN the document contains entries for `/healthz` and `/readyz` with `GET` operations
- AND the operations do not list bearer auth requirements

