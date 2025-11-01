## 1. Implementation
- [ ] 1.1 Add deps: `sea-orm`, `sea-orm-migration`, `uuid` (v4), and `testcontainers` (dev-deps).
- [ ] 1.2 Create `src/db.rs` exposing `async fn init_pool(cfg: &AppConfig) -> anyhow::Result<DatabaseConnection>` with pool config (max connections, acquire timeout).
- [ ] 1.3 Add config fields: `database_url`, `db_max_connections` (default 10), `db_acquire_timeout_ms` (default 5000). Source from `POBLYSH_*`.
- [ ] 1.4 Wire `main` to call `init_pool` and store the `DatabaseConnection` in Axum `State`.
- [ ] 1.5 Scaffold SeaORM Migrator (module or `migration/` crate) with baseline migration `mYYYY_MM_DD_000001_create_tenants`.
- [ ] 1.6 On startup: if `profile in {local, test}`, run `Migrator::up(&db, None).await?` before serving.
- [ ] 1.7 Add a `cargo run -- migrate <up|down|status>` subcommand for non-local profiles.
- [ ] 1.8 Integration test: start Postgres via `testcontainers`, run migrations, assert `tenants` exists and can insert/select.

## 2. Validation
- [ ] 2.1 `openspec validate add-seaorm-setup-and-migrations --strict` passes.
- [ ] 2.2 `cargo test -q` passes including DB integration test (behind `#[cfg(test)]` or feature flag).

## 3. Notes / Non-goals
- Do not introduce provider/connection/signal/sync_job tables here; those land in subsequent changes.
- Prefer application-generated UUID v4 for `tenants.id` to avoid DB extensions.
- Keep migrations forward-only; use `down` only in local/test.

