## Why
Operators need an easy way to view active connections for a tenant for debugging OAuth, verifying token freshness, and understanding current integration coverage. A tenant-scoped listing endpoint supports operational workflows and aligns with the API plan.

## What Changes
- Add HTTP endpoint: `GET /connections` (tenant-scoped) to list active connections.
- Auth: Requires operator bearer token and `X-Tenant-Id` header.
- Query: Optional `provider` filter (snake_case id, e.g., `github`) validated against provider registry.
- Response: `{ connections: [ { id: uuid, provider: string, expires_at?: RFC3339 string, metadata: object } ] }`.
- Sorting: Stable ordering by `id` ascending (MVP; pagination deferred).
- OpenAPI: Document endpoint and response schema in Swagger.

## Impact
- Affected specs: `api-connections` (new capability), references existing `auth` error behavior.
- Affected code: new handler (e.g., `src/handlers/connections.rs`), DTOs, router wiring.
- Dependencies: repository functions from `add-provider-and-connection-entities` to fetch tenant-scoped connections.

