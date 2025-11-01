## Why
We need a reliable Postgres foundation to persist connectors’ state and normalized Signals. The service currently has no database layer. Establishing SeaORM with a migration workflow and a minimal baseline schema unblocks subsequent changes (entities, endpoints, sync engine) and enables local/test automation.

## What Changes
- Introduce SeaORM database connection pooling using `POBLYSH_DATABASE_URL`.
- Add SeaORM Migrator and wire automatic `up` on startup for `local`/`test` profiles; manual for others.
- Create baseline schema with an initial `tenants` table (UUID PK, timestamps).
- Expose `DatabaseConnection` via application state for future repos/services.
- Add integration test utility to boot Postgres (testcontainers) and run migrations.

## Impact
- Affected specs: `database`
- Affected code: `Cargo.toml`, `src/db.rs` (new), `src/main.rs` (startup), optional `migration/` module or crate
- Dependencies: `sea-orm`, `sea-orm-migration`, `uuid` (for app-side UUID v4), `testcontainers` (tests)

