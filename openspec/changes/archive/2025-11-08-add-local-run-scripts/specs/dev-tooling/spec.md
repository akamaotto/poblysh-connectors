## ADDED Requirements
### Requirement: Local Developer Run Scripts
The system SHALL provide repository-level automation (Makefile and Justfile targets) enabling contributors to set up, run, and inspect the API locally without Docker while honoring configuration validation rules and OpenSpec standards.

#### Scenario: Env scaffolding includes crypto key
- **WHEN** a developer runs `make env` or its equivalent Justfile target
- **THEN** a `.env.local` file is created (or updated) with:
  - `POBLYSH_PROFILE=local`
  - `POBLYSH_DATABASE_URL=sqlite://dev.db`
  - `POBLYSH_OPERATOR_TOKEN=local-dev-token`
  - `POBLYSH_CRYPTO_KEY` set to a base64-encoded value that decodes to exactly 32 bytes of entropy
- **AND** the implementation MAY use `openssl rand -base64 32` or any equivalent mechanism, but MUST guarantee the 32-byte requirement regardless of host tooling.
- **AND** the command MUST be safe and idempotent to run multiple times without corrupting existing valid configuration.

#### Scenario: Minimal run sequence succeeds
- **GIVEN** a clean workspace without an existing database
- **WHEN** the developer runs `make env && make db-sqlite && make migrate && make run`
- **THEN** the API starts on `POBLYSH_API_BIND_ADDR` (default `0.0.0.0:8080` if unset)
- **AND** `GET /healthz` returns HTTP 200 within two minutes
- **AND** this sequence MUST NOT require Docker or Postgres to be installed locally.

#### Scenario: Watch target handles missing cargo-watch
- **WHEN** the developer runs `make watch` or the corresponding Justfile target on a machine without `cargo-watch` installed
- **THEN** the command:
  - Detects that `cargo-watch` is not available
  - Emits clear installation guidance (for example, how to install `cargo-watch`)
  - Falls back to running `cargo run` or exits successfully after printing guidance
- **AND** it MUST NOT exit with a non-zero status solely because `cargo-watch` is missing.

#### Scenario: OpenAPI export works locally
- **GIVEN** the API server is running locally and reachable on the configured bind address
- **WHEN** the developer runs `make openapi` or the corresponding Justfile target
- **THEN** an `openapi.json` file is produced in the repository root
- **AND** the produced file parses as valid JSON
- **AND** if `curl` and/or `jq` are not available, the target:
  - Emits a clear, actionable message indicating the missing dependency
  - Uses a graceful fallback if implemented (e.g., a simpler HTTP check without `jq`)
  - MUST either:
    - Exit with a clear non-zero status indicating the missing tools, OR
    - Succeed with a documented degraded behavior that still leaves the repository in a consistent state
- **AND** the behavior (including expected tools and fallbacks) MUST be documented in the local development documentation referenced by this change.

#### Scenario: Postgres connectivity check is optional and non-blocking
- **WHEN** the developer runs `make db-pg-check` or the corresponding Justfile target
- **THEN** if `POBLYSH_DATABASE_URL` is unset or does NOT start with a Postgres scheme (`postgres://` or `postgresql://`), the command:
  - Performs no external checks
  - Exits successfully (no-op)
- **AND** if `POBLYSH_DATABASE_URL` is a Postgres URL:
  - The command SHOULD attempt a lightweight connectivity check (e.g., via `psql`, `pg_isready`, or an equivalent mechanism)
  - If the connectivity check fails, the command SHOULD:
    - Exit with a non-zero status OR
    - Exit successfully but clearly report that Postgres is unreachable
  - If required tooling for the check (such as `psql`/`pg_isready`) is missing, the command:
    - Emits a clear, actionable message about the missing tool
    - MUST NOT block other local workflows (e.g., `make env`, `make db-sqlite`, `make migrate`, `make run`) from succeeding.

#### Scenario: Help output and target parity
- **WHEN** a developer runs `make help` or the default `just` invocation (or a dedicated help target)
- **THEN** the output MUST:
  - List all primary local-development targets with concise one-line descriptions, including at minimum:
    - `env`, `db-sqlite`, `db-pg-check`, `migrate`, `run`, `watch`, `test`, `lint`, `fmt`, `openapi`, and `help`
- **AND** the Makefile and Justfile:
  - SHOULD expose a consistent set of primary targets for local workflows
  - MAY differ in implementation details, but MUST NOT diverge in a way that confuses contributors about the supported local run sequences.

## Quality & Compliance Requirements

### Requirement: OpenSpec and Code Quality Compliance
The local run script capability defined in this change SHALL comply with OpenSpec standards and internal quality expectations.

#### Scenario: OpenSpec validation passes
- **WHEN** `openspec validate add-local-run-scripts --strict` is executed against this change
- **THEN** it completes successfully with no validation errors related to:
  - Missing or malformed requirements
  - Missing or malformed scenarios
  - Incorrect section headers or delta formats.

#### Scenario: Coderabbit-quality implementation readiness
- **WHEN** this specification is implemented
- **THEN** the resulting Makefile and Justfile:
  - Adhere to these requirements exactly
  - Use clear, deterministic, and idempotent commands suitable for automated review
  - Provide actionable, developer-friendly error messages and fallbacks for all optional tools and dependencies.