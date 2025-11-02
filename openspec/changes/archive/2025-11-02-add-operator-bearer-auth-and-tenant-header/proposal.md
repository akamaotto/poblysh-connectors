## Why
Operators need a consistent way to authenticate to the API and scope every request to a tenant. A simple bearer token guard plus a required `X-Tenant-Id` header (UUID) provides immediate security and isolation for local/test and early MVP operations, aligning with the project's tenant‑scoping principle.

## What Changes
- Add operator authentication using `Authorization: Bearer <token>` against configured tokens.
- Enforce required `X-Tenant-Id` header (UUID) on protected endpoints and propagate into request context.
- Bypass auth/tenant guard for public routes (`/healthz`, `/readyz`, `/docs`, `/openapi.json`).
- Document bearer auth security scheme and `X-Tenant-Id` header in OpenAPI.
- Tests for middleware/extractors covering 401/400 paths and happy path.

## Impact
- Affected specs: `auth`, `api-core`
- Affected code: `src/auth.rs` (new middleware/extractors), `src/server.rs` (layering + OpenAPI security), handlers updated to accept tenant in context where needed
- Dependencies: none new required; optionally `subtle` for constant‑time compare

