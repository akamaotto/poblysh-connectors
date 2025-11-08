## Context

This change adds local developer run scripts (Makefile and Justfile) for the Poblysh Connectors API, a Rust service using Axum, SeaORM, and Utoipa.

The goals:

- Zero-docker local development.
- One-command, predictable startup flow.
- Clear, discoverable tasks for common operations.
- Strict alignment with existing configuration and validation rules.
- No new runtime dependencies; all orchestration lives in Makefile/Justfile.

This design refines the initial proposal to:

- Define `db-pg-check` behavior precisely.
- Clarify `POBLYSH_CRYPTO_KEY` generation flexibility.
- Specify `watch` and `openapi` behaviors, including failure modes.
- Document expectations for Makefile/Justfile parity and help output.


## Goals / Non-Goals

- Goals:
  - Enable a new contributor to reach a healthy local `/healthz` within minutes using documented commands.
  - Provide consistent, mirrored targets in Makefile and Justfile.
  - Respect `POBLYSH_*` configuration and `AppConfig` validation.
  - Make behavior deterministic and friendly when optional tools are missing.

- Non-Goals:
  - No changes to runtime API behavior or database schema.
  - No new application-level crates or runtime-linked tools.
  - No responsibility for provisioning Postgres or other external services.


## Existing Stack (Reference Only)

These are descriptive, not normative; the source of truth is `Cargo.toml`.

- Web/API: Axum, Tower, Tokio
- OpenAPI: Utoipa, utoipa-swagger-ui
- Data: SeaORM, sea-orm-migration
- Config & errors: serde, anyhow, thiserror
- Observability: tracing, tracing-subscriber
- CLI: clap
- HTTP: reqwest (for integrations), etc.

Design constraint: local run scripts MUST NOT add new runtime dependencies beyond what is already in `Cargo.toml`.


## Selected Tooling (Non-Runtime)

Recommended (not required) developer tools:

- `just` — task runner.
- `cargo-watch` — auto-reload for dev loop.
- `watchexec` — optional alternative watcher.
- `curl`, `jq` — for HTTP checks and JSON validation.

Principles:

- Scripts MUST degrade gracefully when these tools are missing.
- The absence of any of these tools MUST NOT block the minimal local run sequence.


## Target Map and Behaviors

All primary targets MUST exist in both Makefile and Justfile with equivalent behavior:

- `help`
- `setup`
- `env`
- `db-sqlite`
- `db-pg-check`
- `migrate`
- `run`
- `watch`
- `test`
- `lint`
- `fmt`
- `openapi`

If implementation needs to diverge, semantics MUST remain consistent.


### help

- Purpose: Discoverability.
- Behavior:
  - Print all primary targets with one-line descriptions.
  - MUST include at least: `env`, `db-sqlite`, `migrate`, `run`, `watch`, `test`, `lint`, `fmt`, `openapi`.
- Makefile/Justfile behavior:
  - Both MUST have a `help` (or default) target that lists the same set of primary commands.


### setup

- Purpose: Quick preflight check.
- Behavior:
  - Check for `rustup` and `cargo`.
  - Optionally detect `just`, `cargo-watch`, `curl`, `jq`, `psql`, `pg_isready` and print install hints.
  - MUST NOT fail the overall local dev flow if optional tools are missing.
- Exit:
  - SHOULD succeed (exit 0) unless core Rust tooling is missing.
  - On missing core tooling, MAY exit non-zero with a clear message.


### env

- Purpose: Create/update `.env.local` with safe, valid defaults.
- Behavior:
  - Ensure `.env.local` exists.
  - Set or update, at minimum:
    - `POBLYSH_PROFILE=local`
    - `POBLYSH_DATABASE_URL=sqlite://dev.db`
    - `POBLYSH_OPERATOR_TOKEN=local-dev-token`
    - `POBLYSH_CRYPTO_KEY=<base64 value decoding to 32 bytes>`
  - MUST be idempotent: re-running should not break existing valid values.
- Crypto key generation:
  - Normative requirement:
    - Key MUST be base64-encoded and MUST decode to exactly 32 bytes of entropy.
  - Implementation guidance:
    - `openssl rand -base64 32` is RECOMMENDED when available.
    - Other mechanisms are allowed (e.g., Rust helper, `head -c`, platform utilities) as long as the 32-byte invariant is upheld.
  - If generation fails (e.g., lacks required tools), the script MUST:
    - Emit a clear message describing how to create a valid key manually.
    - Prefer leaving an obviously invalid placeholder only if accompanied by clear instructions.
    - Avoid silently writing malformed keys.


### db-sqlite

- Purpose: Ensure the default local SQLite database is ready.
- Behavior:
  - Ensure that the file backing `sqlite://dev.db` exists (creating an empty file if needed).
  - Normalize path usage so that `POBLYSH_DATABASE_URL=sqlite://dev.db` works as configured.
  - MUST be safe to run multiple times.
- Exit:
  - MUST succeed (exit 0) under normal conditions.
  - On unexpected filesystem errors, SHOULD report a clear message.


### db-pg-check

- Purpose: Optional diagnostic for Postgres connectivity without blocking local dev.

Behavior rules:

1. If `POBLYSH_DATABASE_URL` is unset or does not start with `postgres://` or `postgresql://`:
   - `db-pg-check` MUST:
     - Print a brief message indicating Postgres mode is not active.
     - Exit 0 (no-op success).

2. If `POBLYSH_DATABASE_URL` is postgres-like:
   - If `psql` or `pg_isready` is available:
     - MAY attempt a lightweight connectivity check (e.g., `pg_isready` or simple `psql` connection).
     - On failure:
       - SHOULD print actionable diagnostics.
       - MAY exit non-zero, since this is an explicit opt-in path.
   - If neither `psql` nor `pg_isready` is available:
     - MUST:
       - Emit a clear message explaining that Postgres tools are missing and how to install them.
       - Exit 0 (since Postgres is optional for minimal local dev).

3. General:
   - `db-pg-check` MUST NEVER be a prerequisite for the minimal SQLite-based flow.
   - Documentation SHOULD position `db-pg-check` as an optional health tool for contributors who choose Postgres.


### migrate

- Purpose: Apply database migrations in an idempotent way.
- Behavior:
  - Use the existing migration entrypoint (e.g., `cargo run -- migrate up`) consistent with `src/main.rs` and the `migration` crate.
  - MUST be safe to re-run: no destructive side effects on up-to-date databases.
- Exit:
  - MUST exit non-zero on genuine migration failures and print clear error messages.


### run

- Purpose: Start the API using current configuration.
- Behavior:
  - Execute `cargo run` for the main binary.
  - Respect `POBLYSH_PROFILE`, `POBLYSH_DATABASE_URL`, and other envs from `.env.local`.
  - Default bind: `POBLYSH_API_BIND_ADDR` (e.g., `0.0.0.0:8080`) if configured that way by the application.
- Expectations:
  - Used after `make env`, `make db-sqlite`, and `make migrate` for the minimal happy path.


### watch

- Purpose: Provide a live-reload dev loop without breaking on missing tools.

Behavior rules:

1. If `cargo-watch` is installed:
   - Run a standard watch command (e.g., `cargo watch -x 'run'` or equivalent).

2. If `cargo-watch` is NOT installed:
   - MUST:
     - Print a clear message with installation instructions.
     - Fall back to `cargo run` (or equivalent single-run).
     - Exit 0 (do not treat missing `cargo-watch` as an error).

3. Parity:
   - Makefile and Justfile MUST implement equivalent behavior.

This ensures contributors always get a usable `watch` experience without mandatory extra tooling.


### test

- Purpose: Run the test suite conveniently.
- Behavior:
  - Run `cargo test` (or `cargo test --all` if appropriate for this repo).
- Exit:
  - MUST propagate the underlying test result exit code.


### lint

- Purpose: Enforce linting before commits/CI.
- Behavior:
  - Run `cargo clippy` with a strict configuration such as:
    - `cargo clippy --all-targets --all-features -- -D warnings`
- Exit:
  - MUST be non-zero on lint violations.


### fmt

- Purpose: Format Rust code consistently.
- Behavior:
  - Run `cargo fmt --all`.
- Exit:
  - MUST be non-zero when formatting fails (e.g., syntax errors).


### openapi

- Purpose: Export the OpenAPI spec to `openapi.json` in repo root.

Behavior rules:

1. Preconditions:
   - Primary mode (simple):
     - Assumes the API server is already running locally (e.g., via `make run`).
     - MUST be documented: “Start the server (make run) before running make openapi.”
   - An implementation MAY choose to start a temporary server; if so, behavior MUST still be deterministic and documented.

2. Operation:
   - Use `curl` to fetch `http://localhost:8080/openapi.json` (or the actual configured path).
   - Write output to `openapi.json` at the repo root.

3. Validation:
   - If `jq` is available:
     - SHOULD validate that `openapi.json` parses as JSON (e.g., `jq .info.title openapi.json`).
   - If `jq` is not available:
     - MUST NOT fail solely due to its absence.
     - SHOULD print a hint: “Install jq to validate OpenAPI JSON.”

4. Failure Modes:
   - If `curl` is missing:
     - MUST print a clear message with install hints.
     - SHOULD exit non-zero, since the requested export cannot be performed.
   - If the server is unreachable or returns non-200:
     - MUST print diagnostics and exit non-zero.
   - These behaviors make failures explicit while avoiding quiet corruption.

5. Parity:
   - Makefile and Justfile MUST implement equivalent semantics.


## Makefile / Justfile Parity Requirements

To meet OpenSpec and quality standards:

- Primary Targets:
  - Both Makefile and Justfile MUST expose the primary targets listed above with consistent names and semantics.

- Help:
  - Both MUST provide:
    - A `help` or default output that:
      - Enumerates primary targets.
      - Includes one-line descriptions.
  - This is part of the acceptance criteria and ensures discoverability.

- Behavior Consistency:
  - Differences in implementation syntax (Make vs just) are allowed, but:
    - Side effects, exit codes, and expectations MUST match.
    - Optional tool handling MUST follow the same rules.


## Idempotency and UX Considerations

- `env`, `db-sqlite`, and `migrate` MUST be safe to run multiple times.
- All targets MUST emit human-readable, actionable log messages.
- Optional dependencies (just, cargo-watch, curl, jq, psql, pg_isready) MUST:
  - Not break the default SQLite-based local flow.
  - Provide guidance when missing.

This aligns with professional DX expectations and Coderabbit-style quality standards.


## Alignment with OpenSpec Standards

- This change is tooling-only; it is captured under a dedicated capability delta (`dev-tooling`) without modifying runtime behavior specs.
- Requirements are expressed with clear SHALL/SHOULD/MUST/MAY language.
- Scenarios in the associated spec file:
  - Cover env scaffolding, minimal run success, watch behavior, and OpenAPI export.
  - Can be extended to reference the clarified behaviors defined in this design.
- The design ensures:
  - Deterministic behavior.
  - Clear error handling.
  - No hidden dependencies.
  - Consistency across all related documentation and scripts.