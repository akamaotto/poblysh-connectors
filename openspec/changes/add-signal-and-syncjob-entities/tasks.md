## 1. Implementation
- [ ] 1.1 Add migrations: `mYYYY_MM_DD_000004_create_signals`, `mYYYY_MM_DD_000005_create_sync_jobs` with columns and indices as specified.
- [ ] 1.2 Create SeaORM entities under `src/entity/` for `signals` and `sync_jobs`.
- [ ] 1.3 Add integration tests verifying FKs, index‑backed query patterns, and basic insert/select.

## 2. Validation
- [ ] 2.1 `openspec validate add-signal-and-syncjob-entities --strict` passes.
- [ ] 2.2 `cargo test -q` passes including DB integration tests.

## 3. Notes / Non-goals
- No scheduler/executor logic in this change; covered by `add-sync-executor-and-cursoring` later.
- No API endpoints here; read models only.
- Dedupe strategy for signals will be finalized in a later change; `dedupe_key` is provided for future use.

