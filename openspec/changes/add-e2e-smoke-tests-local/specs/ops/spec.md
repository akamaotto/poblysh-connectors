# OPS Capability – Local Smoke Tests

## ADDED Requirements

### Requirement: Local smoke test harness

The system SHALL provide a developer-facing smoke test harness that boots the compiled `connectors` binary against a local database and verifies core HTTP health.

#### Scenario: Successful smoke run against local database

- **WHEN** the harness spawns the binary with `POBLYSH_PROFILE=test`, a selected `127.0.0.1:{port}` bind address, and a reachable `POBLYSH_DATABASE_URL`
- **AND** waits for `/readyz` to return `200 OK` within 60 seconds using a 200–500ms polling backoff
- **THEN** it validates the public endpoints `/`, `/healthz`, `/readyz`, `/openapi.json`, `/providers`
- **AND** exercises `/protected/ping` with `Authorization: Bearer <token>` and `X-Tenant-Id` headers generated for the run
- **AND** shuts the child process down gracefully (SIGTERM then forced kill if still running after the timeout)

#### Scenario: Missing required environment variables

- **WHEN** the harness detects that `POBLYSH_DATABASE_URL` or `POBLYSH_OPERATOR_TOKEN` is unset before spawning the binary
- **THEN** it aborts the run without starting the process and emits actionable guidance describing the missing variable(s)

#### Scenario: Readiness timeout surfaces diagnostics

- **WHEN** `/readyz` fails to return `200 OK` within the 60 second deadline
- **THEN** the harness terminates the child process and reports the chosen bind address, resolved database URL, elapsed wait duration, and the last `/readyz` status/body payload

#### Scenario: Port conflict triggers retry

- **WHEN** the first attempt to bind the selected port fails due to an `AddrInUse` error
- **THEN** the harness selects a new unused port via `portpicker`, respawns the binary once, and continues the readiness flow
- **AND** if the second attempt also fails, it aborts with a clear error message describing the bind conflict
