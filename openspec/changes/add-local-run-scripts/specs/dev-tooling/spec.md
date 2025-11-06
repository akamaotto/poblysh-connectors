## ADDED Requirements
### Requirement: Local Developer Run Scripts
The system SHALL provide repository-level automation (Makefile and Justfile targets) enabling contributors to set up, run, and inspect the API locally without Docker while honoring configuration validation rules.

#### Scenario: Env scaffolding includes crypto key
- **WHEN** a developer runs `make env`
- **THEN** a `.env.local` file is created (or updated) with `POBLYSH_PROFILE=local`, `POBLYSH_DATABASE_URL=sqlite://dev.db`, `POBLYSH_OPERATOR_TOKEN=local-dev-token`, and a generated `POBLYSH_CRYPTO_KEY` that decodes to 32 bytes of entropy.

#### Scenario: Minimal run sequence succeeds
- **GIVEN** a clean workspace without an existing database
- **WHEN** the developer runs `make env && make db-sqlite && make migrate && make run`
- **THEN** the API starts on `POBLYSH_API_BIND_ADDR` (default `0.0.0.0:8080`) and `GET /healthz` returns HTTP 200 within two minutes.

#### Scenario: Watch target handles missing cargo-watch
- **WHEN** the developer runs `make watch` on a machine without `cargo-watch`
- **THEN** the command emits installation guidance and falls back to `cargo run` without exiting with a non-zero status.

#### Scenario: OpenAPI export works locally
- **WHEN** the developer runs `make openapi`
- **THEN** an `openapi.json` file is produced in the repository root and validates as JSON (e.g., via `jq .info.title`).
