## Context
Provide a simple operator authentication layer and tenant scoping guard for the API. MVP uses static bearer tokens from env and a required `X-Tenant-Id` UUID header to enforce tenant isolation consistently across endpoints.

## Goals / Non-Goals
- Goals: bearer auth guard, tenant header validation and propagation, OpenAPI docs, tests.
- Non-Goals: JWT/OAuth, RBAC/permissions, tenant existence checks in DB (can be added later).

## Decisions
- Use Axum middleware (`tower::Layer` via `axum::middleware::from_fn`) to validate Authorization and `X-Tenant-Id` on protected routes.
- Compare tokens in constant time (e.g., `subtle` crate) to avoid timing attacks.
- Store parsed `TenantId(Uuid)` and an `OperatorAuth` marker in request extensions for handler/repo access.
- Bypass public endpoints to avoid breaking docs/health checks.
- OpenAPI: add HTTP bearer security scheme; model `X-Tenant-Id` as a required header parameter for protected operations.

## Alternatives Considered
- JWT with signature verification: overkill for MVP; static tokens are sufficient for local/test.
- Global middleware for entire app: increases special‑case logic for public routes; prefer per‑router layering.

## Open Questions
- Should we validate that `tenant_id` exists in DB on every request? Proposed: not in MVP due to cost; add an optional check in a later change.
- Multi‑token identities (operator names/claims)? Proposed: optional; keep tokens opaque in MVP.

