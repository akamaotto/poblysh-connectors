## Context
This change adds local developer run scripts (Makefile/Justfile) for a Rust service using Axum, SeaORM, and Utoipa. The goal is to enable fast iteration without Docker, with SQLite as the default local database and Postgres supported optionally.

## Goals / Non-Goals
- Goals: zero-docker local dev; one-command startup; consistent targets; minimal prerequisites
- Non-Goals: change runtime code or specifications; introduce new runtime crates; manage OS-level DB services

## Existing Stack (from Cargo.toml)
- axum `0.8.6`, tokio `1.48.0`, tower `0.5.1`
- utoipa `5.3.1`, utoipa-swagger-ui `9.0.2`
- serde `1.0.217`, serde_json `1.0.138`
- sea-orm `1.1.17`, sea-orm-migration `1.1.1`
- clap `4.5.26`, anyhow `1.0.95`, thiserror `2.0.11`
- tracing `0.1.41`, tracing-subscriber `0.3.19`

## Selected Tools & Versions
- just `1.43.0` (task runner) — https://just.systems/man/en/
- cargo-watch `v8.5.3` (dev loop) — https://github.com/watchexec/cargo-watch
- watchexec `v2.3.2` (optional) — https://github.com/watchexec/watchexec
- curl/jq — standard CLI utilities for HTTP and JSON checks

Rationale:
- No new runtime dependencies; scripts orchestrate `cargo` and existing project CLIs
- `just` provides a friendly DX; Makefile kept for parity and ubiquity
- `cargo-watch` is optional; scripts degrade gracefully if not installed

## Task Map (Draft)
Target names are mirrored in Makefile/Justfile for consistency.

- `help` — list tasks
- `setup` — check for `rustup`, `cargo`; print optional installs for `just`, `cargo-watch`
- `env` — create `.env.local` if missing with:
  - `POBLYSH_PROFILE=local`
  - `POBLYSH_DATABASE_URL=sqlite://dev.db`
  - `POBLYSH_OPERATOR_TOKEN=local-dev-token` (satisfy validation)
  - `POBLYSH_CRYPTO_KEY=<32-byte base64>` (generate via `openssl rand -base64 32` to satisfy `AppConfig::validate()`)
- `db-sqlite` — ensure `dev.db` exists (creating parent dir if needed), DSN `sqlite://dev.db`
- `db-pg-check` — if `POBLYSH_DATABASE_URL` starts with `postgres`, try `psql` connect or `pg_isready`; otherwise no-op
- `migrate` — `cargo run -- migrate up` (uses `Migrator` via `src/main.rs`)
- `run` — `cargo run` (server binds at `POBLYSH_API_BIND_ADDR`, default `0.0.0.0:8080`)
- `watch` — `cargo watch -x 'run'` if installed; else echo a hint then `cargo run`
- `test` — `cargo test --all`
- `lint` — `cargo clippy --all-targets --all-features -- -D warnings`
- `fmt` — `cargo fmt --all`
- `openapi` — `curl http://localhost:8080/openapi.json > openapi.json && jq .info.version openapi.json`

## Lightweight Deep Research Algorithm (Specialized)
1) Parallel searches
   - Docs: just manual; Cargo book; SeaORM migration guide; Axum and Utoipa docs for route and OpenAPI path
   - Community: examples of Rust service Makefiles/Justfiles; `cargo watch` usage patterns
   - Codebase scans: ripgrep for `POBLYSH_` envs, migration entrypoints, `/openapi.json`

2) Sequential reinforcement
   - Verify our env loader contract (src/config/mod.rs) and align task defaults (`PROFILE`, `DATABASE_URL`, `OPERATOR_TOKEN`)
   - Confirm migration CLI (`connectors migrate up`) backed by `Migrator::up`
   - Validate OpenAPI exposure path in `src/server.rs` and ensure `openapi` target curls the correct route
   - Choose SQLite-by-default for local productivity; keep Postgres opt-in without blocking local runs
   - Normalize naming: short, conventional target names with one concern each and clear help text

3) Synthesis
   - Author Makefile/Justfile with mirrored commands, optional dependencies, and helpful failure messages
   - Ensure idempotency: `env`, `db-sqlite`, and `migrate` should be safe to re-run

## Risks / Trade-offs
- Divergence between SQLite local runs and Postgres production specifics; mitigate by offering `db-pg-check` and advocating occasional local PG runs
- Optional tools not present; mitigate with graceful fallback and install guidance

## Migration Plan
- Add files and README usage docs in a single PR
- No schema or code changes required; revert is trivial

## Open Questions
- Should we add a `providers-seed` target (post-migration) to ensure minimal registry rows?
- Should `nextest` be included as an optional faster test target?
