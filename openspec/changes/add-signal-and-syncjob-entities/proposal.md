## Why
We need persistent storage for normalized `Signal` events and the `SyncJob` work queue. These entities underpin ingestion, normalization, and scheduling: Signals are the canonical event records consumed downstream, and SyncJobs track units of work (full/incremental/webhook) with state and cursors. Proper indices are required for fast queries and job picking.

## What Changes
- Define SeaORM entities for `signals` and `sync_jobs` with required columns and constraints.
- Add migrations to create both tables with high‑value indices for query patterns.
- Keep repository logic minimal in this change (entities + schema only); scheduling/execution comes later.
- Add integration tests validating constraints and performant queries via the indices.

## Impact
- Affected specs: `database`
- Affected code: `migration/` (new migrations), `src/entity/` (SeaORM models)
- Dependencies: reuse `sea-orm`, `sea-orm-migration`, `uuid`, `serde_json`

