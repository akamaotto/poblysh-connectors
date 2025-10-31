## ADDED Requirements

### Requirement: Service Bootstrap
The system SHALL start an HTTP server for local development and expose a minimal API surface.

#### Scenario: Server boots with default bind address
- WHEN the service starts without `POBLYSH_API_BIND_ADDR` set
- THEN it binds to `0.0.0.0:8080`
- AND `GET /` returns HTTP 200 with JSON body

#### Scenario: Server boots with custom bind address
- GIVEN `POBLYSH_API_BIND_ADDR` is set to `127.0.0.1:3000`
- WHEN the service starts
- THEN it binds to `127.0.0.1:3000`

### Requirement: Root Hello Route
The system SHALL provide a root endpoint that confirms service identity and version.

#### Scenario: Root returns service and version
- WHEN a client calls `GET /`
- THEN the response is HTTP 200 with `Content-Type: application/json`
- AND the JSON includes `{ "service": "poblysh-connectors", "version": "0.1.0" }` (values may be sourced from build metadata)

### Requirement: Swagger Documentation
The system SHALL expose OpenAPI documentation with an interactive UI.

#### Scenario: Swagger UI available
- WHEN a client calls `GET /docs`
- THEN the response is HTTP 200 with HTML containing "Swagger UI"

#### Scenario: OpenAPI JSON available
- WHEN a client calls `GET /openapi.json`
- THEN the response is HTTP 200 with `Content-Type: application/json`
- AND the body contains a valid OpenAPI 3 document

### Requirement: Local Development Only
The bootstrap SHALL avoid Docker or cloud dependencies in MVP.

#### Scenario: No Docker dependency
- WHEN following the local run instructions
- THEN the service starts using a locally installed toolchain
- AND no Docker or cloud services are required

