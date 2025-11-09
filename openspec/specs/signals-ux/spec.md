# signals-ux Specification

## Purpose
TBD - created by archiving change add-tenant-mapping-and-signals-ux. Update Purpose after archive.
## Requirements
### Requirement: Signals are always tenant-scoped

All signals exposed to Poblysh UI MUST be scoped by a single tenant identifier derived by Poblysh Core.

- Poblysh Core:
  - Resolves the current tenant (session, token, etc.).
  - Resolves or derives the corresponding `X-Tenant-Id` according to the tenant-mapping spec.
  - Calls the Connectors Service `/signals` endpoint with:
    - `X-Tenant-Id: <resolved-id>`
    - `Authorization: Bearer <operator-token>` (or equivalent operator credential).
- Poblysh Frontend:
  - NEVER constructs `X-Tenant-Id` or operator tokens.
  - Only calls Poblysh Core endpoints.

#### Scenario: Fetch tenant signals for dashboard

**Note**: The Core API paths shown below (e.g., `/api/connectors/signals`) are recommendations. Actual Core API implementations should follow existing Poblysh API patterns and conventions.

1. User opens the Poblysh dashboard for Tenant T.
2. Frontend calls `GET /api/connectors/signals` (Poblysh Core).
3. Poblysh Core:
   - Authenticates user and resolves Tenant T.
   - Resolves `connectors_tenant_id` for Tenant T.
   - Calls Connectors `GET /signals` with `X-Tenant-Id=<connectors_tenant_id>`.
4. Connectors returns only signals for that tenant.
5. Poblysh Core returns signals to the frontend.
6. Frontend displays signals; no cross-tenant leakage is possible.

---

### Requirement: Signals API is consumed via a stable Core-facing contract and unified error model

The Connectors `/signals` endpoint MAY evolve, but Poblysh Core MUST provide a stable façade to frontend clients and MUST follow the unified `ApiError` / problem+json error model defined in the global error spec.

- **Core-mediated Frontend Contract:**
  - Exposes a stable, documented route following Poblysh Core conventions:
    - Recommended pattern: `GET /api/connectors/signals` (actual Core API paths should follow existing Poblysh API patterns)
    - Core MUST provide backward-compatible guarantees for frontend clients
  - **Supported query parameters:**
    - `provider` (string, optional): Filter by provider slug (e.g., `github`, `slack`)
    - `connection_id` (UUID, optional): Filter by specific connection ID
    - `kind` (string, optional): Filter by signal kind
    - `occurred_after` (ISO 8601 timestamp, optional): Filter signals after this time
    - `occurred_before` (ISO 8601 timestamp, optional): Filter signals before this time
    - `cursor` (string, optional): Opaque pagination cursor from previous response
    - `limit` (integer, optional, default: 50, max: 100): Maximum number of signals to return
  - **Core responsibilities:**
    - Translate frontend query parameters to Connectors `/signals` query format and headers
    - Inject `X-Tenant-Id` based on resolved tenant mapping
    - Handle authentication to Connectors Service
    - Preserve pagination semantics and cursor opaqueness
  - **Contract stability:**
    - Any breaking changes in Connectors API MUST be absorbed by Core
    - Frontend contract stability is guaranteed by Core
    - Core MAY evolve internally but MUST maintain frontend compatibility
  - **API contract alignment:**
    - The normative contract for `/signals` uses query parameters for filters and cursor-based pagination
    - Any discrepancy in current OpenAPI or implementation MUST be treated as a bug to be addressed in a dedicated follow-up change

#### Scenario: Frontend requests GitHub-related signals

1. Frontend calls (using recommended Core API pattern):
   - `GET /api/connectors/signals?provider=github&limit=50`
   - Note: Actual Core API path should follow existing Poblysh API conventions
2. Poblysh Core:
   - Resolves tenant and `X-Tenant-Id`.
   - Calls Connectors:
     - `GET /signals?provider=github&limit=50`
     - With appropriate headers.
3. Connectors returns normalized signals.
4. Core forwards them (possibly lightly reshaped) to the UI.

---

### Requirement: Cursor-based pagination for signals UX

Signals listing MUST use cursor-based pagination to support stable, scalable browsing.

- Connectors `/signals` returns:
  - `signals: [...]`
  - `next_cursor: string | null`
- Poblysh Core:
  - Passes through or wraps `next_cursor` to frontend.
- Frontend:
  - Treats `cursor` as an opaque token.
  - Requests subsequent pages via:
    - `GET /api/connectors/signals?cursor=<token>&limit=<n>`

#### Scenario: User scrolls through older signals

1. Initial page load:
   - Frontend calls without `cursor`.
   - Receives `signals` and `next_cursor="abc123"`.
2. On "Load more":
   - Frontend calls:
     - `GET /api/connectors/signals?cursor=abc123&limit=50`
3. Poblysh Core:
   - Forwards request to Connectors `/signals` with same cursor.
4. Connectors returns next page and possibly another `next_cursor`.
5. Process repeats until `next_cursor` is `null`.

---

### Requirement: Filtering signals by connection and provider

The UX MUST support filtering signals by provider and individual connection.

At minimum, Poblysh Core MUST support:

- Filter by provider slug (e.g. `github`, `slack`).
- Filter by connection ID (UUID) returned from the connections API.
- Combined filters (e.g. GitHub + specific connection).

#### Scenario: View signals for a specific GitHub connection

1. Tenant has multiple connections (e.g. GitHub Org A, GitHub Org B).
2. User selects "Org A" in the UI.
3. Frontend calls:
   - `GET /api/connectors/signals?provider=github&connection_id=<org-a-connection-id>`
4. Core:
   - Calls Connectors `/signals` with those filters and correct `X-Tenant-Id`.
5. Only signals from that connection are returned and displayed.

---

### Requirement: Signals include enough metadata for basic UX

The signals payload MUST include fields sufficient to:

- Display:
  - Provider (e.g. GitHub).
  - Connection or source context (e.g. repo/org or workspace).
  - Kind (semantic label of the event).
  - Occurred timestamp.
- Deep link:
  - A way (directly or via metadata) to link out to the source item when applicable.

Exact fields (e.g. `kind`, `provider_slug`, `connection_id`, `occurred_at`) are defined in the Signals schema; this spec requires they remain available to build the described UX.

#### Scenario: Render a signal row

Given a signal from Connectors:

- `provider_slug: "github"`
- `connection_id: "<uuid>"`
- `kind: "pull_request_opened"`
- `occurred_at: "2024-01-15T10:30:00Z"`

Frontend (via Core response) MUST be able to render:

- Provider icon (GitHub)
- Short label (e.g. "Pull request opened")
- Time (e.g. "2 minutes ago")
- Optional link to underlying resource (if provided in metadata).

---

### Requirement: Error handling is UX-safe and aligned with the unified error model
Error handling for signals flows MUST conform to the shared `ApiError` / problem+json contract and MUST be presented to users in a safe, non‑leaky way via Poblysh Core.

**Strict compliance with unified error model:**
- All error responses from Connectors Service MUST use the standardized `ApiError` envelope with screaming-snake-case `code` values
- Poblysh Core MUST preserve this exact structure when translating errors for frontend consumption
- Frontend MUST handle errors based on the `code` field only, not on message content
- No service-specific error formats may be introduced at any layer

When Connectors `/signals` returns errors:

- Poblysh Core MUST:
  - Normalize error responses for the frontend.
  - Avoid leaking implementation-specific error details (tokens, internals).
  - Preserve the standard `ApiError` / problem+json structure from the Connectors Service, including `code` in screaming-snake-case (e.g., `UNAUTHORIZED`, `TENANT_HEADER_REQUIRED`, `RATE_LIMITED`).
- Frontend UX:
  - Shows user-friendly messages mapped from these standardized errors:
    - Example: "We’re unable to load signals right now. Please try again."

#### Scenario: Database unavailable on Connectors

1. Poblysh Core calls `/signals`.
2. Connectors returns:
   - `503` with a standardized error body.
3. Core:
   - Maps to a generic 503 or a safe error structure for frontend.
4. Frontend:
   - Displays a non-technical error message and a retry option.

---

