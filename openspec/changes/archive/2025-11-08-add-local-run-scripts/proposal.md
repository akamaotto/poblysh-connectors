## Why
Local developer workflows currently require manual commands and ad-hoc environment setup. Adding first-class local run scripts enables fast iteration without Docker: one-command startup, migrations, tests, linting, and OpenAPI export. This improves onboarding, reliability, and parity across contributors’ machines.

## What Changes
- Add a `Makefile` and `Justfile` at the repository root for a consistent local workflow (no Docker required), using a shared set of primary targets:
  - `help` — list all primary tasks with one-line descriptions (MUST exist in both `Makefile` and `Justfile` and stay reasonably in sync).
  - `setup` — verify core prerequisites (`rustup`, `cargo`) and print install hints for optional tooling (e.g., `just`, `cargo-watch`, `curl`, `jq`).
  - `env` — scaffold `.env.local` with safe defaults aligned with our configuration loader:
    - `POBLYSH_PROFILE=local`
    - `POBLYSH_DATABASE_URL=sqlite://dev.db`
    - `POBLYSH_OPERATOR_TOKEN=local-dev-token`
    - `POBLYSH_CRYPTO_KEY=<base64 string>` that decodes to 32 bytes of entropy
    - The implementation MAY use `openssl rand -base64 32` or any equivalent mechanism; the normative requirement is the 32-byte entropy property, not the specific tool.
  - `db-sqlite` — ensure `dev.db` exists and is compatible with `POBLYSH_DATABASE_URL=sqlite://dev.db` for the local profile. Idempotent and safe to re-run.
  - `db-pg-check` — optional Postgres connectivity check:
    - Only attempts a check when `POBLYSH_DATABASE_URL` has a postgres-like scheme.
    - If Postgres client tools (e.g., `psql`, `pg_isready`) are unavailable, it emits clear guidance and exits successfully.
    - MUST NOT block the basic local workflow; it is informational only.
  - `migrate` — run SeaORM migrations in an idempotent, repeatable way (e.g., `cargo run -- migrate up`) using the existing `migration` crate wiring.
  - `run` — start the API using `cargo run`, honoring `POBLYSH_API_BIND_ADDR` (default `0.0.0.0:8080`).
  - `watch` — start a development loop:
    - If `cargo-watch` is installed, run a standard watch command (e.g., `cargo watch -x run`).
    - If not installed, print clear installation guidance and fall back to `cargo run` without exiting non-zero.
  - `test` — run unit and integration tests (e.g., `cargo test --all`), aligning with existing project conventions.
  - `lint` — run `cargo clippy` with appropriate flags (e.g., `--all-targets --all-features -- -D warnings`) to enforce a clean, warning-free codebase.
  - `fmt` — run `cargo fmt --all` to enforce consistent formatting.
  - `openapi` — export the OpenAPI specification to `openapi.json` in a robust, predictable way:
    - Primary behavior:
      - If the API is already running locally (e.g., via `make run`), use `curl http://localhost:8080/openapi.json > openapi.json`.
    - Required behavior when preconditions are not met:
      - If the server is not reachable, the target MUST:
        - Either start a temporary server instance to generate `openapi.json` and then shut it down,
        - OR clearly instruct the user to start the server (e.g., “Run `make run` in another terminal, then re-run `make openapi`”) and exit with a meaningful, documented status.
      - If `curl` is unavailable, the target MUST print a clear message explaining the requirement and how to install it.
      - If `jq` is unavailable, the target MUST still write `openapi.json` if possible and SHOULD print a note that JSON validation is skipped, without failing the core export step solely due to missing `jq`.

- Default to SQLite for the `local` profile to avoid external dependencies:
  - Local runs SHOULD “just work” on a clean machine with Rust installed.
  - Postgres remains fully supported as an opt-in by setting `POBLYSH_DATABASE_URL` appropriately.
- Add concise documentation in `README.md` (Local development section) describing:
  - The primary targets.
  - The recommended `make`/`just` flows.
  - The behavior when optional tools (e.g., `cargo-watch`, `curl`, `jq`) are not available.

## Impact
- Affected specs: introduce a tooling-focused capability under dev tooling; no changes to shipped API behavior.
- Affected code:
  - No new runtime crates.
  - No changes to request/response schemas, handlers, or database models.
  - Only adds repository-level automation files and associated documentation.
- Affected docs:
  - Update `README.md` to describe the unified Makefile/Justfile workflow.
  - Reference the `add-local-run-scripts` change where appropriate.

## Acceptance Criteria
- Help and parity:
  - `make help` and the default `just` listing MUST show all primary tasks with concise, accurate one-line descriptions.
  - `Makefile` and `Justfile` SHOULD expose a consistent set of primary targets (`help`, `setup`, `env`, `db-sqlite`, `db-pg-check`, `migrate`, `run`, `watch`, `test`, `lint`, `fmt`, `openapi`) to avoid contributor confusion.
- Minimal run sequence:
  - On a clean machine with Rust installed, a new contributor can run:
    - `make env && make db-sqlite && make migrate && make run`
    - And receive HTTP 200 from `GET /healthz` within two minutes.
  - `.env.local` generated by `make env` MUST satisfy all validation performed by the existing configuration system for the `local` profile (including a valid operator token and 32-byte crypto key).
- Watch target resilience:
  - `make watch`:
    - MUST use `cargo-watch` when available.
    - MUST NOT fail the workflow when `cargo-watch` is missing; instead MUST print guidance and fall back to a reasonable `cargo run` behavior.
- OpenAPI export robustness:
  - `make openapi` MUST produce a valid `openapi.json` file in the repository root under documented conditions.
  - The behavior MUST be clearly defined for:
    - Missing running server.
    - Missing `curl` or `jq`.
  - At minimum, contributors following the documented steps (including starting the server when instructed) can reliably generate `openapi.json`.
- Optional Postgres support:
  - When `POBLYSH_DATABASE_URL` uses a Postgres scheme and Postgres is reachable, `db-pg-check` SHOULD confirm connectivity and report clear failures.
  - When Postgres is not configured or client tools are missing, `db-pg-check` MUST NOT block the standard SQLite-based local workflow.
- Validation:
  - The change and its spec deltas MUST pass `openspec validate add-local-run-scripts --strict`.
  - The automation scripts and documentation MUST adhere to established OpenSpec conventions and be suitable for review under strict quality standards (clear, deterministic behavior; no hidden requirements; no unnecessary runtime dependencies).

## Core Technologies (Informative, Non-Normative)
This change is explicitly tooling-only and MUST NOT introduce new runtime crate dependencies. It is designed to work with the existing stack as defined in `Cargo.toml`, including (non-exhaustive, versions illustrative):

- Axum, Tokio, Tower — HTTP server and async runtime.
- Utoipa and Utoipa Swagger UI — OpenAPI generation and documentation.
- SeaORM and sea-orm-migration — ORM and migrations for Postgres/SQLite.
- Clap, Anyhow, Thiserror — CLI and error handling.
- Tracing stack — logging and observability.

The local run scripts:
- MUST rely only on these existing crates for runtime behavior.
- MAY rely on ubiquitous CLI tools (`make`, `just`, `curl`, `jq`, `openssl`, `cargo-watch`) as optional or recommended enhancements, with graceful degradation when they are absent.

## Lightweight Deep Research & Maintenance Approach (Informative)
To keep this change aligned with best practices and the evolving codebase:

1. Parallel discovery:
   - Monitor official documentation for Cargo, Axum, SeaORM, and Utoipa.
   - Sample community examples of Makefile/Justfile patterns for Rust APIs.
   - Periodically scan this repository for:
     - `POBLYSH_*` environment variable expectations.
     - Migration entrypoints and OpenAPI routes.

2. Sequential reinforcement:
   - Revalidate that:
     - `POBLYSH_PROFILE`, `POBLYSH_DATABASE_URL`, `POBLYSH_OPERATOR_TOKEN`, and `POBLYSH_CRYPTO_KEY` defaults used by scripts match `src/config/mod.rs`.
     - `migrate` targets call the correct CLI entrypoints.
     - `openapi` targets use the actual OpenAPI route or generation mechanism implemented in this codebase.
   - Adjust scripts if core behavior changes, without introducing breaking runtime dependencies.

3. Synthesis:
   - Keep Makefile/Justfile targets:
     - Idempotent.
     - Self-documenting via `help`.
     - Friendly in failure modes (especially around optional tools).
   - Ensure all behavior remains fully compatible with OpenSpec conventions and passes strict validation for this change.
