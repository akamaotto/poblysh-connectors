# integration-guide Specification

## Purpose
TBD - created by archiving change add-connectors-integration-guide. Update Purpose after archive.
## Requirements
### Requirement: Integration guide describes the Connectors Service role and boundaries
The integration guide MUST clearly define the Connectors Service’s responsibilities and boundaries so engineers understand what it does, what it does not do, and how it fits into the Poblysh architecture.

#### Scenario: Reader understands what the Connectors Service does and does not do

- GIVEN a Poblysh engineer opens the Connectors integration guide
- THEN the guide MUST:
  - Clearly describe the Connectors Service as a separate microservice responsible for:
    - Managing third-party connections (e.g., GitHub, Slack, Jira, Google, Zoho)
    - Handling OAuth flows and token storage
    - Receiving and normalizing external events into Signals
  - Clearly state that it:
    - Is not the primary source of truth for Poblysh tenants
    - Is not a general-purpose backend for all Poblysh product data
    - Is expected to be called by Poblysh Core (and trusted services), not directly by untrusted clients

---

### Requirement: Integration guide defines the tenancy and scoping model
The integration guide MUST specify the tenancy model and how `X-Tenant-Id` is chosen, passed, and enforced for all tenant‑scoped operations.

#### Scenario: Poblysh Core knows how to choose and send `X-Tenant-Id`

- GIVEN Poblysh Core is integrating with the Connectors Service
- WHEN calling any tenant-scoped Connectors endpoint
- THEN the guide MUST specify that:
  - All tenant-scoped calls require an `X-Tenant-Id` header.
  - `X-Tenant-Id` is the canonical tenant identifier within the Connectors Service.
  - Poblysh Core is responsible for resolving and providing the correct `X-Tenant-Id` value.
- AND the guide MUST present:
  - A recommended strategy to reuse the Poblysh Tenant ID as `X-Tenant-Id` when it is:
    - A stable, non-sensitive UUID suitable for cross-service usage.
  - An alternative strategy using a 1:1 mapping table:
    - `poblysh_tenant_id` ↔ `connectors_tenant_id` (UUID)
    - Owned and managed by Poblysh Core.
- AND the guide MUST state that:
  - Frontend clients MUST NOT invent or guess `X-Tenant-Id`.
  - Any mapping logic lives in Poblysh Core or another trusted backend layer.

---

### Requirement: Integration guide clarifies the security model
The integration guide MUST define how operator authentication works, which credentials are used, and how frontend vs. backend responsibilities are separated.

#### Scenario: Engineers understand how to authenticate calls to Connectors

- GIVEN Poblysh engineers implement calls from Poblysh Core to the Connectors Service
- THEN the guide MUST:
  - Explain that the Connectors Service uses operator-level bearer tokens:
    - Configured via environment variables `POBLYSH_OPERATOR_TOKENS` (array of tokens) with a compatibility fallback to `POBLYSH_OPERATOR_TOKEN` for single-token setups.
    - Document that when both variables are present, `POBLYSH_OPERATOR_TOKENS` takes precedence and `POBLYSH_OPERATOR_TOKEN` can still be honored for historical deploys.
  - State that:
    - Poblysh frontend never holds or sends the operator token.
    - Poblysh Core (or another trusted backend) injects:
      - `Authorization: Bearer <operator_token>`
      - `X-Tenant-Id: <resolved tenant id>`
    - Public provider-to-Connectors flows (OAuth callbacks, webhooks) are handled via separate mechanisms (e.g. callback URLs, signatures) and do NOT rely on frontend-controlled credentials.
- AND the guide MUST clearly distinguish:
  - End-user authentication and sessions (Poblysh Core concern).
  - Connectors operator authentication (infrastructure/backend concern).

---

### Requirement: Integration guide documents the core integration journey
The integration guide MUST describe end‑to‑end integration flows (providers, OAuth, connections, signals) with required headers and roles for each step.

#### Scenario: Implementing “Connect GitHub” follows a documented sequence

- GIVEN a team wants to implement “Connect GitHub” (or similar provider) in Poblysh
- WHEN they follow the integration guide
- THEN the guide MUST describe the high-level sequence:

  1. List providers:
     - Poblysh Core calls `GET /providers` on Connectors.
     - Frontend uses a Poblysh Core endpoint (e.g. `/api/connectors/providers`) to render available connectors.
  2. Start OAuth:
     - Frontend calls Poblysh Core (e.g. `POST /api/connectors/providers/{provider}/authorize`).
     - Poblysh Core:
       - Resolves `X-Tenant-Id` for the current tenant.
       - Calls `POST /connect/{provider}` on Connectors with:
         - `Authorization: Bearer <operator_token>`
         - `X-Tenant-Id` header.
       - Receives `authorize_url` and returns it to frontend.
     - Frontend redirects the user to `authorize_url`.
  3. Handle OAuth callback:
     - Provider redirects to a Poblysh-controlled callback URL.
     - Poblysh Core:
       - Validates request.
       - Calls `GET /connect/{provider}/callback` on Connectors with query parameters `code` and `state`.
       - Receives created `connection` details.
       - Persists or associates connection metadata as needed.
       - Redirects user back to the Poblysh UI.
  4. Show connections:
     - Frontend calls Poblysh Core (e.g. `GET /api/connectors/connections`).
     - Poblysh Core calls `GET /connections` on Connectors with:
       - `Authorization` and `X-Tenant-Id`.
     - Response is mapped into the UI’s “Connected accounts” list.
  5. Retrieve signals:
     - Frontend calls Poblysh Core (e.g. `GET /api/connectors/signals?provider=github&limit=25`).
     - Poblysh Core calls `GET /signals` on Connectors with:
       - `Authorization` and `X-Tenant-Id`.
       - Appropriate filters (e.g., `?provider=github&connection_id=uuid&kind=issue_created&limit=50`).
     - Response includes paginated signals with optional payloads based on `include_payload` parameter.
     - Response is used to render activity, timelines, or other views.

- AND the guide MUST:
  - Highlight which steps are frontend-visible vs backend-only.
  - Emphasize that all direct Connectors calls (except provider callbacks/webhooks) originate from trusted services.

---

### Requirement: Integration guide provides a curated endpoint reference for Poblysh integration
The integration guide MUST include a concise, use‑case‑oriented list of endpoints with purpose, caller, required headers, and how each maps to Poblysh UX flows.

#### Scenario: Engineer can map UI/flows to specific endpoints quickly

- GIVEN an engineer is implementing Poblysh features using the Connectors Service
- WHEN they consult the integration guide
- THEN the guide MUST include a concise endpoint overview oriented around Poblysh use cases, including at minimum:

  - `GET /providers`
    - Purpose: populate “available integrations” UI.
    - Auth: public.
  - `POST /connect/{provider}`
    - Purpose: start OAuth.
    - Auth: operator token, `X-Tenant-Id` required.
  - `GET /connect/{provider}/callback`
    - Purpose: complete OAuth.
    - Usage: Poblysh Core backend callback handler.
  - `GET /connections`
    - Purpose: list tenant connections.
    - Auth: operator token, `X-Tenant-Id` required.
  - `GET /signals`
    - Purpose: list signals for tenant/connection.
    - Intended usage: filters and pagination via query parameters, always scoped by `X-Tenant-Id` through Poblysh Core.
    - Query parameters: `provider` (slug), `connection_id` (UUID), `kind` (signal kind), `occurred_after`/`occurred_before` (RFC3339 timestamps), `limit` (default: 50, max: 100), `cursor` (for pagination), `include_payload` (boolean).
    - Example usage: `GET /signals?provider=github&limit=25&include_payload=true` or `GET /signals?connection_id=123e4567-e89b-12d3-a456-426614174000&kind=issue_created`
    - Note: if the live OpenAPI or implementation models `/signals` differently (e.g., as path parameters), treat that as an inconsistency to be corrected by a dedicated follow-up change so it matches this intended contract.
    - Auth: operator token, `X-Tenant-Id` required.
  - Selected webhook endpoints (`/webhooks/{provider}`, `/webhooks/{provider}/{tenant_id}`) described at a conceptual level:
    - Purpose: provider → Connectors ingestion.
    - Who configures: backend/infra, not frontend.

- AND for each endpoint in this curated list, the guide MUST:
  - Indicate who calls it (Poblysh Core vs provider vs infra).
  - Indicate required headers and auth.
  - Link its role back to the integration journey.

---

### Requirement: Integration guide ties into Swagger UI and OpenAPI as the detailed reference
The integration guide MUST point to Swagger UI (`/docs`) and `GET /openapi.json` as the authoritative detailed reference and explain how to keep examples in sync with the OpenAPI contract.

#### Scenario: Guide consumers know where to find exact schemas

- GIVEN the integration guide references Connectors endpoints
- THEN the guide MUST:
  - Point to:
    - Swagger UI at `/docs` for interactive exploration.
    - `GET /openapi.json` as the machine-readable API spec.
  - Explicitly state that:
    - The integration guide explains “how Poblysh should use the API.”
    - The OpenAPI document is the authoritative source for detailed request/response schema and error formats.
  - Instruct contributors to:
    - Keep examples aligned with the current OpenAPI.
    - Update the guide when relevant API changes are made.

---

