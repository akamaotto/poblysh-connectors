## Why
We need standard health and readiness endpoints for orchestration and local development. Health provides a fast liveness probe, while readiness reflects dependency status (e.g., database). These routes must be public and avoid auth to work in diverse environments and with Kubernetes.

## What Changes
- Add `GET /healthz` liveness endpoint returning HTTP 200 with a minimal JSON body.
- Add `GET /readyz` readiness endpoint that returns HTTP 200 when dependencies are ready, otherwise HTTP 503 with a diagnostic JSON body.
- Bypass operator auth and tenant header for both endpoints.
- Document endpoints in OpenAPI.

## Impact
- Affected specs: `api-core`, `auth` (already notes bypass; no change needed)
- Affected code: `src/server.rs` (routing), `src/handlers/health.rs` (new), `src/db.rs` (readiness check helper)
- Dependencies: none new

