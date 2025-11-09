# Connectors Service Integration Guide

This guide explains how Poblysh Core and frontends should integrate with the Poblysh Connectors Service, a standalone microservice that manages third-party integrations and their data flows.

## Connectors Service Role and Boundaries

The Connectors Service is a separate microservice responsible for:

- **Managing third-party connections** (e.g., GitHub, Slack, Jira, Google, Zoho)
- **Handling OAuth flows and token storage** in an encrypted, secure manner
- **Receiving and normalizing external events** into standardized Signals
- **Providing a catalog of available providers** for integration

The Connectors Service **does not**:

- Serve as the primary source of truth for Poblysh tenants
- Function as a general-purpose backend for all Poblysh product data
- Handle end-user authentication and sessions (that's Poblysh Core's responsibility)
- Expect to be called directly by untrusted clients

The service is designed to be called by Poblysh Core (and other trusted backend services), not directly by frontend clients.

## Tenancy and Scoping Model

All tenant-scoped operations require the `X-Tenant-Id` header. This header is the canonical tenant identifier within the Connectors Service.

### Recommended Strategy
Use the Poblysh Tenant ID directly as `X-Tenant-Id` when it is:
- A stable, non-sensitive UUID suitable for cross-service usage

### Alternative Strategy
Maintain a 1:1 mapping table owned by Poblysh Core:
- `poblysh_tenant_id` ↔ `connectors_tenant_id` (UUID)
- Managed exclusively by Poblysh Core

**Important**: Frontend clients MUST NOT invent or guess `X-Tenant-Id`. Any mapping logic lives in Poblysh Core or another trusted backend layer.

## Security Model

### Operator Authentication
The Connectors Service uses operator-level bearer tokens:

- **Primary configuration**: `POBLYSH_OPERATOR_TOKENS` (array of tokens)
- **Compatibility fallback**: `POBLYSH_OPERATOR_TOKEN` (single token)
- **Precedence**: When both variables are present, `POBLYSH_OPERATOR_TOKENS` takes precedence

### Call Flow
- **Poblysh Core** injects both headers:
  - `Authorization: Bearer <operator_token>`
  - `X-Tenant-Id: <resolved tenant id>`
- **Frontend** never holds or sends the operator token
- **Public provider flows** (OAuth callbacks, webhooks) use separate mechanisms (callback URLs, signatures)

## Core Integration Journey

### Step 1: List Available Providers

**Sequence:**
1. Poblysh Core calls `GET /providers` on Connectors
2. Frontend calls Poblysh Core endpoint (e.g., `/api/connectors/providers`)
3. Frontend renders available integrations UI

**Key points:**
- `/providers` is public (no auth required)
- Returns provider catalog with capabilities and metadata

### Step 2: Start OAuth Flow

**Sequence:**
1. Frontend calls Poblysh Core (e.g., `POST /api/connectors/providers/{provider}/authorize`)
2. Poblysh Core:
   - Resolves `X-Tenant-Id` for the current tenant
   - Calls `POST /connect/{provider}` on Connectors with:
     - `Authorization: Bearer <operator_token>`
     - `X-Tenant-Id` header
   - Receives `authorize_url` and returns it to frontend
3. Frontend redirects user to `authorize_url`

**Key points:**
- This is a backend-mediated flow
- Frontend never sees operator tokens
- Tenant scoping is enforced by Poblysh Core

### Step 3: Handle OAuth Callback

**Sequence:**
1. Provider redirects to a Poblysh-controlled callback URL
2. Poblysh Core:
   - Validates request authenticity
   - Calls `GET /connect/{provider}/callback` on Connectors with query parameters (`code`, `state`)
   - Receives created `connection` details
   - Persists or associates connection metadata as needed
   - Redirects user back to Poblysh UI

**Key points:**
- Callback handling is backend-only
- Connectors Service validates OAuth state and tokens
- Connection details are stored securely

### Step 4: List Connections

**Sequence:**
1. Frontend calls Poblysh Core (e.g., `GET /api/connectors/connections`)
2. Poblysh Core calls `GET /connections` on Connectors with:
   - `Authorization: Bearer <operator_token>`
   - `X-Tenant-Id` header
3. Response is mapped into UI's "Connected accounts" list

**Key points:**
- Always scoped to specific tenant
- Shows active connections with provider metadata
- Supports connection management (delete, refresh)

### Step 5: Retrieve Signals

**Sequence:**
1. Frontend calls Poblysh Core (e.g., `GET /api/connectors/signals?provider=github&limit=25`)
2. Poblysh Core calls `GET /signals` on Connectors with:
   - `Authorization: Bearer <operator_token>`
   - `X-Tenant-Id` header
   - Appropriate filters as query parameters
3. Response is used to render activity, timelines, or other views

**Key points:**
- Always scoped by `X-Tenant-Id` through Poblysh Core
- Supports rich filtering and pagination
- Optional payload inclusion for detailed data

## Curated Endpoint Reference

### Core Integration Endpoints

#### `GET /providers`
- **Purpose**: Populate "available integrations" UI
- **Auth**: Public (no authentication required)
- **Called by**: Poblysh Core (on behalf of frontend)
- **Returns**: Provider catalog with capabilities and OAuth scopes

#### `POST /connect/{provider}`
- **Purpose**: Start OAuth flow for a specific provider
- **Auth**: Operator token required, `X-Tenant-Id` required
- **Called by**: Poblysh Core backend
- **Returns**: Authorization URL for user redirect

#### `GET /connect/{provider}/callback`
- **Purpose**: Complete OAuth callback and create connection
- **Usage**: Poblysh Core backend callback handler
- **Auth**: Operator token required
- **Parameters**: OAuth `code` and `state` query parameters
- **Returns**: Created connection details

#### `GET /connections`
- **Purpose**: List tenant's connections
- **Auth**: Operator token required, `X-Tenant-Id` required
- **Called by**: Poblysh Core (on behalf of frontend)
- **Returns**: Paginated list of tenant connections

#### `GET /signals`
- **Purpose**: List signals for tenant/connection with filtering
- **Intended usage**: Query parameters for filtering and pagination, always scoped by `X-Tenant-Id` through Poblysh Core
- **Query parameters**:
  - `provider` (slug): Filter by provider (e.g., `github`, `slack`)
  - `connection_id` (UUID): Filter by specific connection
  - `kind` (signal kind): Filter by signal type (e.g., `issue_created`, `pull_request_opened`)
  - `occurred_after`/`occurred_before` (RFC3339 timestamps): Time range filtering
  - `limit` (default: 50, max: 100): Pagination limit
  - `cursor` (string): Pagination cursor for next page
  - `include_payload` (boolean): Include full signal payload
- **Example usage**: 
  - `GET /signals?provider=github&limit=25&include_payload=true`
  - `GET /signals?connection_id=123e4567-e89b-12d3-a456-426614174000&kind=issue_created`
- **Note**: If the live OpenAPI or implementation models `/signals` differently (e.g., as path parameters), treat that as an inconsistency to be corrected by a dedicated follow-up change so it matches this intended contract.
- **Auth**: Operator token required, `X-Tenant-Id` required

### Webhook Endpoints (Backend/Infra Only)

#### `/webhooks/{provider}` and `/webhooks/{provider}/{tenant_id}`
- **Purpose**: Provider → Connectors event ingestion
- **Who configures**: Backend/infrastructure, not frontend
- **Authentication**: Provider-specific signatures or tokens
- **Usage**: Automatic webhook registration during connection setup

## Error Handling

All Connectors endpoints use the unified `ApiError` / problem+json-style envelope:

- **Format**: Consistent error response structure across all endpoints
- **Common codes**:
  - `UNAUTHORIZED`: Missing or invalid operator token
  - `FORBIDDEN`: Valid token but insufficient permissions
  - `TENANT_HEADER_REQUIRED`: Missing `X-Tenant-Id` header
  - `TENANT_NOT_FOUND`: Invalid tenant identifier
  - `CONNECTION_NOT_FOUND`: Connection does not exist for tenant
  - `PROVIDER_NOT_SUPPORTED`: Requested provider is not available

For detailed error schemas and codes, refer to the OpenAPI specification.

## OpenAPI Reference

- **Swagger UI**: Available at `/docs` for interactive exploration
- **Machine-readable spec**: `GET /openapi.json` 
- **Relationship**: 
  - This integration guide explains "how Poblysh should use the API"
  - The OpenAPI document is the authoritative source for detailed request/response schemas and exact error formats

**Keeping in sync**: When contributing API changes, update examples in this guide to stay aligned with the current OpenAPI contract.

## Related Specifications

For deeper rules on tenant mapping and signals UX, refer to:
- `openspec/changes/add-tenant-mapping-and-signals-ux/specs/tenant-mapping/spec.md`
- `openspec/changes/add-tenant-mapping-and-signals-ux/specs/signals-ux/spec.md`

## Integration Checklist

When implementing Connectors Service integration:

- [ ] Ensure all tenant-scoped calls include `X-Tenant-Id` header
- [ ] Never expose operator tokens to frontend code
- [ ] Use Poblysh Core as the mediation layer for all frontend requests
- [ ] Implement proper error handling using the unified error model
- [ ] Test OAuth flows end-to-end with actual providers
- [ ] Handle webhook payload validation and processing
- [ ] Monitor rate limits and implement backoff strategies
- [ ] Securely store any provider-specific configuration