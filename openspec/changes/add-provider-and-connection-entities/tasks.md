## 1. Implementation
- [ ] 1.1 Add migrations: `mYYYY_MM_DD_000002_create_providers`, `mYYYY_MM_DD_000003_create_connections` with schema and indices as specified.
- [ ] 1.2 Create SeaORM entities under `src/entity/` for `providers` and `connections`.
- [ ] 1.3 Implement repositories under `src/repos/`:
      - `providers.rs` with `get_by_slug`, `list_all`, `upsert`.
      - `connections.rs` with `create`, `get_by_id`, `find_by_unique`, `list_by_tenant_provider`, `update_tokens_status`.
- [ ] 1.4 Wire provider seeding in startup for `local`/`test` profiles (idempotent upserts).
- [ ] 1.5 Add unit tests for repository methods (use in-memory patterns/mocks where possible).
- [ ] 1.6 Add integration tests with Postgres (via `testcontainers`) covering constraints and tenant isolation.

## 2. Validation
- [ ] 2.1 `openspec validate add-provider-and-connection-entities --strict` passes.
- [ ] 2.2 `cargo test -q` passes including DB integration tests.

## 3. Notes / Non-goals
- Token encryption is handled in a later change (`add-local-token-encryption`); store opaque ciphertext columns here.
- Do not add API endpoints in this change; CRUD exposure will come later.
- Keep repo layer minimal; avoid premature abstraction until more providers land.

