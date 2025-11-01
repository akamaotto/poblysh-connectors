## ADDED Requirements

### Requirement: Signal Entity Schema
The system SHALL define a `signals` table storing normalized events emitted by connectors, tenant‑scoped and queryable by provider, kind, and time.

Columns (MVP):
- `id UUID PRIMARY KEY NOT NULL`
- `tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE`
- `provider_slug TEXT NOT NULL REFERENCES providers(slug)`
- `connection_id UUID NOT NULL REFERENCES connections(id) ON DELETE CASCADE`
- `kind TEXT NOT NULL` (normalized event kind, e.g., `issue_created`, `pr_merged`, `message_posted`)
- `occurred_at TIMESTAMPTZ NOT NULL` (provider event timestamp)
- `received_at TIMESTAMPTZ NOT NULL DEFAULT now()` (time processed by system)
- `payload JSONB NOT NULL` (normalized event payload)
- `dedupe_key TEXT NULL` (optional, for future idempotency logic)
- `created_at TIMESTAMPTZ NOT NULL DEFAULT now()`
- `updated_at TIMESTAMPTZ NOT NULL DEFAULT now()`

Indices:
- `(tenant_id, provider_slug, occurred_at DESC)` for provider time‑range queries
- `(tenant_id, kind, occurred_at DESC)` for kind‑filtered queries
- `(connection_id, occurred_at DESC)` for per‑connection exploration
- `(tenant_id, provider_slug, dedupe_key)` (non‑unique, to support future de‑dupe checks)

#### Scenario: Insert signal succeeds
- GIVEN an existing tenant, provider, and connection
- WHEN inserting a new signal row
- THEN the row is created and timestamps are set appropriately

#### Scenario: Query by tenant and kind is efficient
- GIVEN many signals across kinds and providers
- WHEN querying by `(tenant_id, kind)` ordered by `occurred_at DESC`
- THEN the database uses the composite index and returns results quickly

### Requirement: SyncJob Entity Schema
The system SHALL define a `sync_jobs` table representing scheduled or webhook‑triggered units of work for connectors, tenant‑scoped with status, cursors, and timing metadata.

Columns (MVP):
- `id UUID PRIMARY KEY NOT NULL`
- `tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE`
- `provider_slug TEXT NOT NULL REFERENCES providers(slug)`
- `connection_id UUID NOT NULL REFERENCES connections(id) ON DELETE CASCADE`
- `job_type TEXT NOT NULL` (e.g., `full`, `incremental`, `webhook`)
- `status TEXT NOT NULL DEFAULT 'queued'` (e.g., `queued`, `running`, `succeeded`, `failed`)
- `priority SMALLINT NOT NULL DEFAULT 0`
- `attempts INT NOT NULL DEFAULT 0`
- `scheduled_at TIMESTAMPTZ NOT NULL DEFAULT now()`
- `retry_after TIMESTAMPTZ NULL` (next eligible time after backoff)
- `started_at TIMESTAMPTZ NULL`
- `finished_at TIMESTAMPTZ NULL`
- `cursor JSONB NULL` (opaque provider cursor)
- `error JSONB NULL` (structured failure details)
- `created_at TIMESTAMPTZ NOT NULL DEFAULT now()`
- `updated_at TIMESTAMPTZ NOT NULL DEFAULT now()`

Indices:
- `(status, scheduled_at, priority DESC)` for picking the next ready job
- `(tenant_id, provider_slug, status, scheduled_at)` for tenant/provider queue views
- `(connection_id, status, scheduled_at)` for per‑connection queue operations

#### Scenario: Queue job and pick order by priority and time
- GIVEN multiple `queued` jobs with varying `scheduled_at` and `priority`
- WHEN selecting next job ordered by highest `priority` and earliest `scheduled_at`
- THEN the index supports efficient retrieval

#### Scenario: Retry scheduling
- GIVEN a `failed` job with backoff
- WHEN setting `retry_after` in the future
- THEN job pickers exclude it until `retry_after <= now()`

