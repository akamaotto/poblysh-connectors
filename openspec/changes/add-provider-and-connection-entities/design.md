## Context
Introduce concrete SeaORM entities for `providers` and `connections` and a thin repository layer. Providers are a global catalog; connections are tenant-scoped and represent authorizations to external systems. This prepares the system for OAuth/token flows and connector sync operations.

## Goals / Non-Goals
- Goals: normalized schemas, clear uniqueness and FKs, tenant isolation, minimal repo API.
- Non-Goals: encryption implementation (separate change), API endpoints, background jobs.

## Decisions
- Providers use a stable `slug` (TEXT PK) for easy joins and readability; avoids UUID indirection.
- Connections use UUID v4 PK for efficiency and uniqueness, with a unique tuple `(tenant_id, provider_slug, external_id)` to allow multiple connections per provider across tenants and external accounts.
- Token fields are `*_ciphertext` opaque columns (BYTEA) to be filled by the crypto module later.
- Use `JSONB` `metadata` for provider-specific extras (e.g., Slack team ID, GitHub installation ID) to reduce premature schema coupling.
- Repositories enforce tenant scoping at the method level to prevent cross-tenant leakage.

## Alternatives Considered
- FK to providers by `id UUID` instead of `slug`: unnecessary indirection for a small, well-known catalog.
- Separate tables per provider: rejected; overfits and complicates queries and repos.

## Risks / Trade-offs
- JSONB metadata may encourage dumping dynamic fields; mitigation: document expected keys per provider in connector specs later.
- Slug drift vs code constants; mitigation: central enum/const list and seeding validates slugs on startup.

## Repository API Sketch
```rust
// providers
get_by_slug(slug: &str) -> Result<Option<Provider>>
list_all() -> Result<Vec<Provider>>
upsert(p: ProviderUpsert) -> Result<Provider>

// connections
create(input: NewConnection) -> Result<Connection>
get_by_id(id: Uuid) -> Result<Option<Connection>>
find_by_unique(tenant: Uuid, provider: &str, external_id: &str) -> Result<Option<Connection>>
list_by_tenant_provider(tenant: Uuid, provider: &str, page: Page) -> Result<PageResult<Connection>>
update_tokens_status(id: Uuid, patch: TokenStatusPatch) -> Result<Connection>
```

## Migration Plan
1) Add `providers` table and seed initial rows (local/test boot).
2) Add `connections` table with constraints and indices.
3) Implement entities and repos; write unit tests.
4) Write integration tests asserting constraints and tenant scoping.

## Open Questions
- Should we support soft deletes on connections? Proposed: not for MVP; use `status` to represent revocation.
- Do we need multi-tenant provider overrides? Proposed: no; providers are global.

