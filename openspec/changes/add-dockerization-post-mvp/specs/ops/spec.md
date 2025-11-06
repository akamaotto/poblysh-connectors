## ADDED Requirements

### Requirement: Container Image Build
The system SHALL provide a multi‑stage Dockerfile that builds and runs the `connectors` binary with a non‑root runtime image.

#### Scenario: Multi‑stage build produces runnable image
- WHEN running `docker build -t connectors:local .`
- THEN the build completes successfully
- AND the resulting image starts the API on port 8080 using `POBLYSH_API_BIND_ADDR`

#### Scenario: Non‑root runtime user (UID/GID pinned)
- WHEN the container is running
- THEN the primary process runs as a non‑root user with UID 10001 and GID 10001

### Requirement: Compose Orchestration With Postgres
The system SHALL define a Docker Compose configuration with `db`, `migrate`, and `app` services using health‑based startup ordering.

#### Scenario: Database service is healthy before dependent services
- WHEN running `docker compose up`
- THEN the `db` service reports healthy via `pg_isready`
- AND `migrate` starts only after `db` is healthy

#### Scenario: Migrations run before app becomes ready
- WHEN compose starts `migrate` with `connectors migrate up`
- THEN pending migrations are applied successfully
- AND `app` starts only after `migrate` completes

#### Scenario: App readiness depends on database connectivity
- GIVEN the app service has started
- WHEN the database is reachable and schema is present
- THEN `GET /readyz` returns 200 within 60 seconds

### Requirement: Runtime Configuration Via Environment
The system MUST configure the containerized app exclusively via `POBLYSH_*` environment variables.

#### Scenario: Bind address and database URL set via env
- GIVEN compose sets `POBLYSH_API_BIND_ADDR=0.0.0.0:8080`
- AND `POBLYSH_DATABASE_URL=postgresql://pbl:secret@db:5432/connectors`
- WHEN the app starts
- THEN it binds to `0.0.0.0:8080` and connects to the `db` service

#### Scenario: Operator token required
- GIVEN compose sets `POBLYSH_OPERATOR_TOKEN=dev-operator`
- WHEN the app starts
- THEN configuration validation succeeds and the app starts serving

### Requirement: Health Checks And Logs
The containerized app SHALL expose `/healthz` and `/readyz`, and write structured logs to stdout.

#### Scenario: Health endpoints reachable from host
- WHEN `docker compose up` publishes port 8080
- THEN `curl http://localhost:8080/healthz` returns 200 with a JSON body containing `status`

#### Scenario: JSON logs to stdout
- WHEN the app handles requests
- THEN logs are emitted to stdout in structured form suitable for container log collectors
