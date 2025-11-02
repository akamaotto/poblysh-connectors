## Why
Providers deliver near‑real‑time updates via webhooks. We need a base ingest endpoint to accept these callbacks and translate them into internal work. For MVP, the handler will be simple and secure behind operator auth and tenant scoping; signature verification and public exposure will follow in a separate change.

## What Changes
- Add `POST /webhooks/{provider}` endpoint to accept webhook calls.
- Require operator bearer auth and `X-Tenant-Id` for MVP (public + signature verification comes later).
- Accept JSON bodies and optional `X-Connection-Id` (UUID) to target a specific connection.
- If a valid connection is provided, enqueue a `sync_jobs` row with `job_type = "webhook"` and optional context in `cursor`.
- Always respond quickly with `202 Accepted` on success (after basic validation). Return `4xx/5xx` on validation or server errors.
- Document the endpoint in OpenAPI.

## Impact
- Affected specs: `api-webhooks` (new capability)
- Affected code: `src/server.rs` (routing), `src/handlers/webhooks.rs` (new), `src/repositories/{provider,connection,sync_job}.rs` (lookups, enqueue)
- Dependencies: none new

