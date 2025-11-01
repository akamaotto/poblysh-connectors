## Context
Introduce SeaORM and SeaORM Migrator to establish a Postgres backbone for connectors. This enables tenant isolation and prepares for entities and sync workflows. Postgres runs locally; future deployment will manage migrations explicitly.

## Goals / Non-Goals
- Goals: reliable DB connection pooling; automatic migrations for local/test; minimal baseline schema (`tenants`);
  integration test harness using a real Postgres.
- Non-Goals: adding domain entities (providers, connections, signals, sync jobs); API endpoints; readiness/health endpoints.

## Decisions
- Use SeaORM `Database::connect` with a tuned pool (max 10 conns, 5s acquire timeout default) sourced from `POBLYSH_*` config.
- Run `Migrator::up` automatically for `local` and `test` profiles; require explicit run for others to avoid accidental prod schema drift.
- Generate UUID v4 in application code for `tenants.id` to avoid DB extensions; keep migration portable.
- Keep migrations in a `migration` module initially (single-crate). If complexity grows, promote to a `migration` crate.

## Alternatives considered
- Diesel ORM: rejected due to SeaORM chosen in project stack and async support.
- DB-generated UUID via `uuid-ossp`: rejected for portability and to avoid extension management in MVP.

## Risks / Trade-offs
- Risk: accidental migration in non-local env. Mitigation: profile gate + explicit CLI.
- Trade-off: app-generated UUIDs rely on correct usage in code; acceptable for MVP simplicity.

## Migration Plan
1) Add dependencies and `src/db.rs` connection helper.
2) Scaffold migrator and baseline migration for `tenants`.
3) Gate auto-migrate by profile; add CLI for manual operations.
4) Add integration test with `testcontainers` to validate end-to-end.
5) Follow-on changes add domain tables building on this baseline.

## Open Questions
- Should we separate a `migration` crate now to match SeaORM templates? Proposed: start in-module; promote later if needed.
- Do we want per-tenant schemas or a single schema with `tenant_id` FK on all tables? Proposed: single schema with tenant scoping (YAGNI for multi-schema).

