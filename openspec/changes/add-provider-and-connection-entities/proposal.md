## Why
We need concrete database entities for `Provider` and `Connection` to begin persisting tenant-scoped connections and their metadata. This enables listing providers, creating connections, and preparing for OAuth/token flows and sync jobs. A thin repository layer will encapsulate SeaORM and enforce tenant scoping.

## What Changes
- Define SeaORM entities for `providers` (global catalog) and `connections` (tenant-scoped authorizations).
- Add migrations creating `providers` and `connections` tables with indices and FKs.
- Implement repository layer with scoped CRUD/query operations (tenant-aware for connections).
- Seed or upsert provider catalog entries (Slack, GitHub, Jira, Google, Zoho) during local/test bootstrap.
- Unit/integration tests covering schema constraints and repository behavior.

## Impact
- Affected specs: `database`
- Affected code: `migration/` (new migrations), `src/entity/` (SeaORM models), `src/repos/` (repositories), `src/main.rs` (optional provider seeding on local/test)
- Dependencies: reuse `sea-orm`, `sea-orm-migration`, `uuid`, `serde_json`

