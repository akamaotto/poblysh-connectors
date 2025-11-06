## Why
We need a fast, reliable way to verify that the service boots end‑to‑end and core HTTP endpoints respond against a real local database. This smoke suite catches configuration drift (env, migrations, bind address) and validates that the binary’s startup path (config loader + DB init + auto migrations) is healthy before developers proceed to deeper testing.

## What Changes
- Add an end‑to‑end smoke test harness that:
  - Selects an unused local port using `portpicker` and sets `POBLYSH_API_BIND_ADDR=127.0.0.1:{port}` (explicitly bound to localhost only for security)
  - Spawns the compiled `connectors` binary with `POBLYSH_PROFILE=test` and pre-selected bind address
  - Uses environment to point at a local DB via `POBLYSH_DATABASE_URL` (Postgres primary; SQLite supported for development)
  - Waits on `/readyz` with a bounded timeout (60s default, 200-500ms backoff between polls) to gate readiness
  - Implements one-time retry on port bind conflicts (select new port and respawn binary)
  - Verifies core endpoints: `/`, `/healthz`, `/readyz`, `/openapi.json`, `/providers`
  - Validates a protected path round‑trip (`/protected/ping`) using `Authorization: Bearer <token>` and `X-Tenant-Id` with generated valid tenant UUID
  - Shuts the process down cleanly after assertions with SIGTERM, followed by force kill if needed
- Add a developer task `make smoke` and `just smoke` to run the suite locally (reuses `cargo test` under the hood)
- Add compile‑time dev dependencies for process spawning and port selection
- Document local DB setup (Postgres via Docker) and smoke run instructions in README

No API behavior changes; this is a tooling/test addition.

## Impact
- Affected specs: none (test tooling only)
- Affected code: new `tests/e2e_smoke_tests.rs`, minor README updates, optional Makefile/Justfile targets
- Affected dependencies: add dev‑dependencies for testing

## Acceptance Criteria
- Running `make smoke` (or `just smoke`) performs pre-flight environment validation, starts the binary against a local DB, waits for readiness, and verifies all target endpoints in under 60s on a typical developer machine.
- Pre-flight validation checks for required environment variables: `POBLYSH_DATABASE_URL` and `POBLYSH_OPERATOR_TOKEN`, with clear error messages for missing configuration.
- The harness sets `POBLYSH_PROFILE=test`, requires `POBLYSH_OPERATOR_TOKEN` (fails early with guidance if missing), generates a valid tenant UUID for protected endpoint testing, and uses a deterministic bind address (`127.0.0.1:{selected-port}`) with explicit localhost binding for security.
- Failing readiness or HTTP checks produce clear, actionable errors and logs including: database unreachable, missing token, port conflict, resolved `POBLYSH_DATABASE_URL`, chosen port, and last `/readyz` status/body.
- Primary validation targets Postgres with `postgresql://` URLs; SQLite support available for development convenience with documented limitations.
- Migration validation ensures schema consistency between database engines (when both are available in development environment).
- Proposal validates: `openspec validate add-e2e-smoke-tests-local --strict`.

## Core Technologies and Versions
Existing foundational crates (from Cargo.toml):
- axum 0.8.6, tokio 1.48.0, tower 0.5.1
- utoipa 5.3.1, utoipa-swagger-ui 9.0.2
- sea-orm 1.1.17, sea-orm-migration 1.1.1
- serde 1.0.217, serde_json 1.0.138
- clap 4.5.26, anyhow 1.0.95, thiserror 2.0.11
- tracing 0.1.41, tracing-subscriber 0.3.19
- reqwest (dev) 0.12.9

Selected additional dev crates/tech for this change:
- assert_cmd 2.x — robustly spawn the built binary in tests (https://docs.rs/assert_cmd)
- portpicker 0.1.x — pick an unused localhost port for deterministic binds (https://docs.rs/portpicker)

Note: reqwest 0.12.9 is already present in the codebase and will be used for HTTP checks.

Rationale:
- assert_cmd resolves binary path (`cargo_bin("connectors")`) portably; avoids ad‑hoc path resolution in tests.
- portpicker reduces bind races when selecting a port; acceptable risk for local testing.
- Keep runtime dependencies unchanged; these are dev‑only.

## Lightweight Deep Research Algorithm (tailored to this change)
Goal: derive best‑practice E2E smoke testing for Rust + Axum + SeaORM binaries, aligned to our codebase and community practices.

1) Parallel discovery (fan‑out first pass)
- Docs triage: Axum server lifecycle, SeaORM migrations in test profiles, assert_cmd usage patterns, port selection approaches
- Community scan: example repos, blog posts, and GitHub gists on Rust E2E smoke tests for HTTP services
- Codebase scan: `rg` for endpoints, readiness semantics, config/env usage (`POBLYSH_*`), auto‑migration behavior in `main.rs`

2) Sequential reinforcement (narrow + validate)
- Confirm readiness gate: `/readyz` hits DB; use it to synchronize test start
- Verify auto migrations run for `test`/`local` profiles; if not, call Migrator explicitly in harness
- Compare port selection strategies: bind(0) vs portpicker; choose portpicker for determinism, document race trade‑off
- Validate protected endpoint flow with `Authorization: Bearer` + `X-Tenant-Id` to exercise auth middleware
- Decide graceful shutdown method for child process and safe timeouts

3) Synthesis + checks
- Draft test harness structure and error messages; cross‑check against our crate versions (axum 0.8, tokio 1.48, reqwest 0.12)
- Ensure tests tolerate missing Postgres by allowing SQLite URLs and produce clear guidance if DB unreachable
- Add `make smoke` convenience and docs to reduce friction

## Implementation Sketch
- tests/e2e_smoke_tests.rs
  - **Pre-flight validation**: Check required environment variables (`POBLYSH_DATABASE_URL`, `POBLYSH_OPERATOR_TOKEN`) and fail early with guidance
  - **Port selection**: Pick port with `portpicker::pick_unused_port()`; enforce `127.0.0.1:{port}` binding for security
  - **Binary spawning**: Resolve binary with `assert_cmd::cargo::cargo_bin("connectors")` 
  - **Environment setup**: Set `POBLYSH_PROFILE=test`, `POBLYSH_API_BIND_ADDR=127.0.0.1:{port}`, pass through `POBLYSH_DATABASE_URL`
  - **Token handling**: Use `POBLYSH_OPERATOR_TOKEN` from environment; fail fast if missing (no silent fallback)
  - **Tenant generation**: Generate valid tenant UUID for protected endpoint testing
  - **Process management**: Spawn child; on bind failure, retry once with new port selection; poll `GET /readyz` until 200 or timeout (60s with 200-500ms backoff)
  - **Endpoint validation**: Assert `GET /`, `/healthz`, `/openapi.json`, `/providers`, and `/protected/ping` with `Authorization: Bearer <token>` and `X-Tenant-Id: <tenant-uuid>`
  - **Database validation**: Primary focus on Postgres URLs; support SQLite with documented differences
  - **Migration consistency**: Verify schema status matches expectations for database type
  - **Graceful shutdown**: On drop, send SIGTERM; force kill if still running after timeout
- Makefile/Justfile: add `smoke` target with environment variable validation that runs `cargo test --test e2e_smoke_tests`

## Docs and References
- Axum 0.8: https://docs.rs/axum/0.8.6/axum/
- SeaORM 1.1: https://www.sea-ql.org/SeaORM/
- assert_cmd 2.x: https://docs.rs/assert_cmd
- portpicker 0.1: https://docs.rs/portpicker
- Reqwest 0.12: https://docs.rs/reqwest/0.12.9/reqwest/

## Risks / Trade‑offs
- Port race: even with portpicker, rare collisions can occur; mitigate with retry on bind error
- DB availability: local Postgres may be absent; we accept SQLite fallback but document Postgres as the recommended path; tests should skip with an explanatory message if DB is unreachable and no fallback provided
- Process management: ensure child shutdown to avoid orphaned servers during `cargo test` runs; use timeouts and `kill` as last resort

## Migration Plan
- Land test harness and dev‑deps gated as dev‑only; no production impact
- Add `smoke` task and README instructions
- Encourage contributors to verify local DB via provided Docker command before running smoke

## Open Questions
- Should we gate the smoke test behind an opt‑in env (e.g., `E2E=1`) to avoid running in CI without Postgres? Default: run locally; CI can inject a container later.
- Do we also verify Swagger UI (`/docs`) returns 200, or keep payload checks to JSON endpoints only?
