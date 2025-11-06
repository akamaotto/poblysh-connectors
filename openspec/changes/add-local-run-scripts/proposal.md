## Why
Local developer workflows currently require manual commands and ad-hoc environment setup. Adding first-class local run scripts enables fast iteration without Docker: one-command startup, migrations, tests, linting, and OpenAPI export. This improves onboarding, reliability, and parity across contributors’ machines.

## What Changes
- Add Makefile and Justfile at repo root for consistent local workflow (no Docker required):
  - `setup` — verify prerequisites and print install hints
  - `env` — scaffold `.env.local` with safe defaults, including `POBLYSH_OPERATOR_TOKEN=local-dev-token` and a generated `POBLYSH_CRYPTO_KEY` (32-byte base64 via `openssl rand -base64 32`)
  - `db-sqlite` — ensure `sqlite://dev.db` path configured for local profile
  - `db-pg-check` — optional: verify Postgres connectivity if `POBLYSH_DATABASE_URL` is postgres
  - `migrate` — run SeaORM migrations (idempotent)
  - `run` — start API on `POBLYSH_API_BIND_ADDR` (default `0.0.0.0:8080`)
  - `watch` — start `cargo watch` dev loop (optional if installed)
  - `test` — unit/integration tests
  - `lint` — clippy with `-D warnings`
  - `fmt` — rustfmt all crates
  - `openapi` — curl `http://localhost:8080/openapi.json` → `openapi.json`
- Default to SQLite for local profile to avoid external dependencies; support Postgres when developer opts-in by setting `POBLYSH_DATABASE_URL`.
- Add lightweight docs in `README.md` linking to these tasks.

## Impact
- Affected specs: none (tooling-only; no behavior change to shipped API)
- Affected code: no runtime code changes; add Makefile and Justfile
- Affected docs: update `README.md` (Local development section)

## Acceptance Criteria
- `make help` and `just` print available tasks with one-line descriptions.
- New contributors can: `make env && make db-sqlite && make migrate && make run` and reach `GET /healthz` locally within 2 minutes on a clean machine with Rust installed, with `.env.local` providing both operator token(s) and `POBLYSH_CRYPTO_KEY`.
- `make watch` works when `cargo-watch` is installed; otherwise prints guidance without failing the run.
- SQLite path (`sqlite://dev.db`) works by default; Postgres flow works when `POBLYSH_DATABASE_URL` points to a reachable server.
- Proposal validates: `openspec validate add-local-run-scripts --strict` (no deltas expected; tooling-only change).

## Core Technologies and Versions
Existing crates (Cargo.toml):
- axum 0.8.6, tokio 1.48.0, tower 0.5.1
- utoipa 5.3.1, utoipa-swagger-ui 9.0.2
- serde 1.0.217, serde_json 1.0.138
- sea-orm 1.1.17, sea-orm-migration 1.1.1
- clap 4.5.26, anyhow 1.0.95, thiserror 2.0.11
- tracing 0.1.41, tracing-subscriber 0.3.19

Selected dev tools (no runtime crates added):
- just 1.43.0 — task runner (https://just.systems/man/en/)
- cargo-watch v8.5.3 — dev loop (https://github.com/watchexec/cargo-watch)
- watchexec v2.3.2 — optional alternative watcher (https://github.com/watchexec/watchexec)
- curl/jq — for OpenAPI export and quick checks

Rationale:
- Keep runtime binary free of new dependencies; concentrate workflow in Makefile/Justfile.
- Default to SQLite for predictable, zero-setup local runs; Postgres remains compatible.
- Use ubiquitous CLI tools to keep installation simple and cross-platform.

## Lightweight Deep Research Algorithm (for this change)
Goal: derive best-practice local run tasks for Rust + Axum + SeaORM projects and align with our codebase.

1) Parallel discovery (run concurrently)
- Docs sweep: Cargo commands, SeaORM migration usage, `just` patterns, Axum dev loops
- Community scan: GitHub gists/PRs, blog posts on Makefile/Justfile for Rust services
- Codebase scan: ripgrep in this repo for env keys, migration entrypoints, and OpenAPI path

2) Sequential reinforcement (focused narrowing)
- Normalize task names and env usage to our `POBLYSH_*` schema and `src/config` loader
- Confirm migration invocation path (`migration` crate + `Migrator`) and idempotency expectations
- Validate OpenAPI route (`/openapi.json`) and add export task that curls it
- Choose default DB for local (`sqlite://dev.db`) and provide optional Postgres check
- Derive cross-platform-friendly commands (avoid GNU-only flags where possible)

3) Synthesis & validation
- Draft Makefile/Justfile with mirrored targets and help output
- Dry-run each command sequence against current tree; ensure failures are friendly and actionable
- Document prerequisites and graceful degradation (watch tasks optional)

## Docs and References
- just manual: https://just.systems/man/en/
- Cargo book: https://doc.rust-lang.org/cargo/
- SeaORM migrations: https://www.sea-ql.org/SeaORM/docs/migration/sea-orm-cli-and-entity/
- Axum docs: https://docs.rs/axum/0.8.6/axum/
- Utoipa docs: https://docs.rs/utoipa/5.3.1/utoipa/

## Risks / Trade-offs
- Postgres availability varies across machines; defaulting to SQLite reduces friction but may drift from prod. Mitigation: optional `db-pg-check` and clear override via `POBLYSH_DATABASE_URL`.
- `cargo-watch` not installed everywhere; make watch non-fatal and suggest install commands.

## Migration Plan
- Land Makefile/Justfile and README updates behind single change; no runtime impact
- Announce dev flow in PR summary and changelog

## Open Questions
- Should we include `nextest` as an optional test runner target?
- Do we want a task to seed baseline provider rows after migrations?
