## ADDED Requirements

### Requirement: Provider Entity Schema
The system SHALL define a `providers` table representing supported integration providers (global catalog), with unique slug identifiers and minimal metadata used for display and policy.

Columns (MVP):
- `slug TEXT PRIMARY KEY` (e.g., `slack`, `github`, `jira`, `google_drive`, `google_calendar`, `gmail`, `zoho_cliq`, `zoho_mail`)
- `display_name TEXT NOT NULL`
- `auth_type TEXT NOT NULL` (enum-like string; e.g., `oauth2`, `webhook-only`)
- `created_at TIMESTAMPTZ NOT NULL DEFAULT now()`
- `updated_at TIMESTAMPTZ NOT NULL DEFAULT now()`

Constraints/Indices:
- Primary key on `slug`

#### Scenario: Insert provider succeeds
- GIVEN a new `slug` not present in the table
- WHEN inserting into `providers (slug, display_name, auth_type)`
- THEN the row is created and timestamps are set

#### Scenario: Duplicate slug rejected
- GIVEN an existing `providers.slug = 'github'`
- WHEN inserting another row with `slug = 'github'`
- THEN the operation fails due to primary key violation

### Requirement: Connection Entity Schema
The system SHALL define a `connections` table representing tenant-scoped authorizations for a given provider and external account/workspace. Connections MUST be uniquely identified per `(tenant_id, provider_slug, external_id)`.

Columns (MVP):
- `id UUID PRIMARY KEY NOT NULL`
- `tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE`
- `provider_slug TEXT NOT NULL REFERENCES providers(slug)`
- `external_id TEXT NOT NULL` (provider-specific account/workspace/user/install id)
- `status TEXT NOT NULL DEFAULT 'active'` (enum-like: `active`, `revoked`, `error`)
- `scopes TEXT[] NULL` (granted scopes if applicable)
- `access_token_ciphertext BYTEA NULL` (opaque; to be encrypted by crypto module in later change)
- `refresh_token_ciphertext BYTEA NULL`
- `expires_at TIMESTAMPTZ NULL`
- `metadata JSONB NULL` (opaque provider-specific details)
- `created_at TIMESTAMPTZ NOT NULL DEFAULT now()`
- `updated_at TIMESTAMPTZ NOT NULL DEFAULT now()`

Constraints/Indices:
- Unique index on `(tenant_id, provider_slug, external_id)`
- Index on `tenant_id`

#### Scenario: Insert connection succeeds
- GIVEN an existing tenant and provider
- WHEN inserting a new `(tenant_id, provider_slug, external_id)`
- THEN the row is created and timestamps are set

#### Scenario: Duplicate tuple rejected
- GIVEN a connection exists for `(tenant_id=T, provider_slug='github', external_id='org:42')`
- WHEN inserting another row with the same tuple
- THEN the operation fails due to unique constraint

#### Scenario: Tenant isolation enforced
- GIVEN two tenants `T1`, `T2`
- WHEN each inserts a connection with the same `(provider_slug, external_id)`
- THEN both rows exist because uniqueness is scoped per tenant

### Requirement: Repository Layer (Providers, Connections)
The system SHALL provide a thin repository layer encapsulating SeaORM operations for providers and connections with clear, tenant-aware methods.

Providers repository (MVP):
- `get_by_slug(slug)` → provider or not found
- `list_all()` → iterable list
- `upsert(slug, display_name, auth_type)` → insert or update for seeding

Connections repository (MVP):
- `create(conn)` → creates a connection; enforces unique tuple
- `get_by_id(id)` → returns connection by id
- `find_by_unique(tenant_id, provider_slug, external_id)` → unique lookup
- `list_by_tenant_provider(tenant_id, provider_slug, limit, cursor?)` → paginated listing
- `update_tokens_status(id, tokens?, status?, expires_at?)` → partial update

#### Scenario: Tenant-scoped listing
- GIVEN multiple connections across tenants
- WHEN listing with `tenant_id = T1` and `provider_slug='github'`
- THEN only connections for `(T1, 'github')` are returned

#### Scenario: Upsert providers for seeding
- WHEN calling `upsert` for `slack`, `github`, `jira`, `google_drive`, `google_calendar`, `gmail`, `zoho_cliq`, `zoho_mail`
- THEN rows exist for each slug with latest display name/auth type values

