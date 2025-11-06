## Context
We will containerize the Connectors API (Rust, Axum) and provision a Postgres service for local/dev parity and deployment preparation. The codebase already provides:
- HTTP server on configurable addr (`POBLYSH_API_BIND_ADDR`, default `0.0.0.0:8080`). See src/config/mod.rs:27 and src/server.rs:63.
- Health endpoints: `/healthz` and `/readyz` that hit the DB (src/handlers/mod.rs:55, src/handlers/mod.rs:87).
- DB layer with SeaORM and migrator crate, CLI for migrations: `connectors migrate up` (src/main.rs:20 and src/main.rs:92).

Cross‑cutting concerns justify this design doc: new packaging strategy, external dependencies, health, and startup sequencing.

## Goals / Non‑Goals
- Goals: reproducible images, compose orchestration with DB, migrations, health checks, non‑root runtime, minimal base.
- Non‑Goals: full production infra, TLS termination, autoscaling, secrets management beyond envs, registry publish.

## Tech Selection
- Container tooling: Docker, Compose v2 (no `version:` key required).
- Images:
  - Builder: `rust:1.85-slim` (parametrized; match stable toolchain that supports edition 2024).
  - Runtime: `debian:bookworm-slim` non‑root user. Optional MUSL + distroless follow‑up.
- Database: `postgres:16-alpine` with healthcheck via `pg_isready`.
- Health: container leverages existing `/readyz` (requires DB connectivity).

## References & Docs (for review)
- Docker: Multi‑stage builds (BuildKit), Compose service dependencies and health conditions (Compose v2 spec).
- Postgres: `pg_isready` health probe usage.
- Rust: Containerizing Rust apps best practices (multi‑stage, non‑root, slim runtime), MUSL static linking trade‑offs.
- SeaORM: Deployment notes and TLS backends with `runtime-tokio-rustls`.

## Current Crates (Cargo.toml)
- axum 0.8.6, tokio 1.48.0, utoipa 5.3.1, utoipa-swagger-ui 9.0.2
- sea-orm 1.1.17 (+ postgres/sqlite, tokio-rustls, macros, chrono, uuid, json)
- sea-orm-migration 1.1.1 (tokio-rustls)
- dotenvy 0.15.7, serde 1.0.217, tracing 0.1.41, tracing-subscriber 0.3.19, clap 4.5.26

No additional crates are strictly required for containerization. The runtime already uses rustls‑backed TLS for DB HTTP clients, avoiding OpenSSL issues in slim images.

## Decisions
- Multi‑stage build with separate builder and runtime images to minimize size and drop build toolchain.
- Run as non‑root (uid:gid 10001) in runtime image; expose only 8080.
- Use compose service health checks: `db` with `pg_isready`, `app` via `/readyz`.
- Start order: `db` → `migrate` (one‑off) → `app` using `depends_on` with health conditions.
- Configure via env: `POBLYSH_*` only; do not copy `.env*` into images.

## Dockerfile (outline)
1) Builder (`rust:1.85-slim`): install `libclang` if needed, create user, cache dependencies, build release.
2) Runtime (`debian:bookworm-slim`): add non‑root user, copy binary, set `PORT 8080` and CMD.

## Compose (outline)
- `db`: postgres:16-alpine, volume, healthcheck via `pg_isready`; other services use `depends_on` with `condition: service_healthy`.
- `migrate`: runs `connectors migrate up`; `depends_on: db` with `condition: service_healthy`.
- `app`: uses built image; `depends_on` both `db: condition: service_healthy` and `migrate: condition: service_completed_successfully`; publishes 8080 and health‑checks `/readyz`.
- Env mapping:
  - `POBLYSH_PROFILE=dev`
  - `POBLYSH_API_BIND_ADDR=0.0.0.0:8080`
  - `POBLYSH_DATABASE_URL=postgresql://pbl:secret@db:5432/connectors`
  - `POBLYSH_OPERATOR_TOKEN=dev-operator`

## Risks / Trade‑offs
- Image size vs complexity: MUSL + distroless reduces size but can complicate linking; defer to follow‑up.
- Startup race conditions: mitigated with `depends_on` health and `/readyz` readiness.
- Secrets exposure: ensure no `.env` copied to images; use runtime envs only.

## Migration Plan
1) Add Dockerfile and compose.
2) Validate locally.
3) Document runbook.
4) Optionally add CI build job (future).

## Research Algorithm (Lightweight Deep Research)
Purpose: gather best practices for containerizing Rust/Axum + SeaORM + Postgres with Compose.

Inputs
- Codebase context: crates/versions from Cargo.toml; health endpoints; config envs in src/config/mod.rs:11.
- Topics: Rust multi‑stage Docker, Axum runtime, SeaORM in containers, Postgres compose health, non‑root images, MUSL vs glibc.

Phase A — Parallel Searches (fan‑out)
- A1 Codebase scan (ripgrep): search for `Dockerfile`, `docker`, `compose`, `healthz`, `readyz`, `POBLYSH_` to map current shape and gaps.
- A2 Official docs: fetch Docker multi‑stage patterns; Compose `depends_on` with health; Postgres healthcheck guidance.
- A3 Rust ecosystem: sample Dockerfiles for Axum (multi‑stage), MUSL static linking tips, caching (`cargo chef`).
- A4 SeaORM notes: runtime dependencies, TLS backends, Alpine vs Debian considerations.
- A5 Community feedback: blog posts and GH templates about Rust container hardening and non‑root setups.

Phase B — Sequential Reinforcement (narrowing)
- B1 Filter findings to match our crates/versions (axum 0.8, sea‑orm 1.1, tokio 1.48, utoipa 5.x).
- B2 Prototype decision checks: ensure rustls avoids OpenSSL at runtime; verify `/readyz` covers DB.
- B3 Compose correctness: validate `depends_on` health, create minimal migration job pattern; confirm env var names map to POBLYSH_*.
- B4 Security baselines: enforce non‑root, drop caps, use slim images; confirm no secrets in layers.

Outputs
- Concrete Dockerfile and compose templates aligned to our codebase.
- Curated references list with doc URLs and doc versions (captured during execution of research script if automated).
- Decision notes and trade‑offs captured here.

## Open Questions
- Do we want a MUSL/distroless image now or as a size‑reduction follow‑up?
- Should migrations be sidecar/entrypoint in app or remain a dedicated one‑off job in compose?
