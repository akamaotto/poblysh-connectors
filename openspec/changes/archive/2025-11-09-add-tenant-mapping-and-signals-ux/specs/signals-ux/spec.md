# Spec: Signals UX for Poblysh Connectors Integration

## Overview

This spec defines how Poblysh surfaces and interacts with normalized signals produced by the Poblysh Connectors Service.

It focuses on:
- How frontend and backend consumers should query, filter, and paginate signals.
- How signals relate to tenants and connections.
- The expected user experience patterns (UX) around viewing and exploring signals.

This document is scoped to the interaction and UX contract. It does not redefine the low-level Signals schema or provider-specific normalization rules, which are (or will be) covered in dedicated specs.

**Related specifications:**
- See `openspec/changes/add-tenant-mapping-and-signals-ux/specs/tenant-mapping/spec.md` for tenant scoping and `X-Tenant-Id` usage requirements
- See `openspec/changes/add-connectors-integration-guide/specs/integration-guide/spec.md` for integration patterns and implementation guidance

---

## Objectives

### Primary Objectives

1. Ensure a consistent, tenant-safe way to retrieve signals from the Connectors Service.
2. Define integration patterns so Poblysh frontend can:
   - Display recent activity/signals for a tenant.
   - Filter signals by provider, connection, and kind.
   - Navigate results via cursor-based pagination.
3. Make it clear how Poblysh Core should mediate all access:
   - Injecting `X-Tenant-Id`.
   - Handling authentication to the Connectors Service.
   - Exposing a stable, frontend-friendly API.

### Non-Objectives

- Redesigning the underlying `/signals` API shape.
- Defining all possible `Signal.kind` values or provider mappings.
- Implementing visual design (colors, typography); this spec focuses on flows and contracts.
- Describing ingestion mechanics (`/webhooks`, jobs) except where needed to explain UX expectations.

---

## Terminology

- Tenant:
  - A Poblysh customer account in the primary product.
  - Mapped to an `X-Tenant-Id` used by the Connectors Service (see tenant-mapping spec).

- Connection:
  - A tenant-scoped authorization to a provider (e.g. GitHub, Slack).
  - Identified by a `connection.id` (UUID) returned by the Connectors Service.

- Signal:
  - A normalized representation of an external event (e.g. GitHub PR opened).
  - Exposed via the Connectors Service `/signals` endpoint.

- Poblysh Core:
  - The main backend/API for Poblysh.com.
  - Acts as the only trusted caller of the Connectors Service on behalf of tenants.

---

## ADDED Requirements

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

## MODIFIED Requirements

**Important Implementation Note:**
This spec establishes the intended contract and UX patterns for signals consumption. Any discrepancies between the current Connectors `/signals` implementation/OpenAPI and the behavior described herein should be addressed in a focused follow-up change (e.g., `update-signals-endpoint-shape`). This spec serves as the normative target for UX and integration patterns.

None yet. This spec is an additive layer on top of existing `/signals` behavior. Any future changes to the Signals schema or URL shape must update this section to describe how UX and Core contracts are affected.

---

## REMOVED Requirements

None.

---

## Open Questions

These MUST be resolved (or explicitly answered) in follow-up iterations or related specs:

1. Which `Signal.kind` values are guaranteed/stable across providers?
2. How much provider-specific metadata should be exposed directly vs behind Core transforms?
3. Do we need first-class support for:
   - "Unread" or "dismissed" state per signal?
   - Pinning or grouping signals?
4. How are rate limits surfaced to UX (e.g. show banner vs silent retry)?

These do not block the initial Signals UX contract but will influence future enhancements.

---

## Concrete Request/Response Examples

### Example: Frontend → Core → Connectors Signal Flow

**Step 1: Frontend requests signals from Poblysh Core**
```http
GET /api/connectors/signals?provider=github&connection_id=conn-12345&limit=20 HTTP/1.1
Authorization: Bearer <user-session-token>

Response:
HTTP/1.1 200 OK
Content-Type: application/json

{
  "signals": [
    {
      "id": "signal-abc123",
      "provider_slug": "github",
      "connection_id": "conn-12345",
      "kind": "pull_request_opened",
      "occurred_at": "2024-01-15T14:30:00Z",
      "metadata": {
        "repo": "poblysh/core",
        "title": "Add tenant mapping feature",
        "url": "https://github.com/poblysh/core/pull/123"
      }
    }
  ],
  "next_cursor": "next_page_token_xyz",
  "has_more": true
}
```

**Step 2: Poblysh Core calls Connectors Service**
```http
GET /signals?provider=github&connection_id=conn-12345&limit=20 HTTP/1.1
Authorization: Bearer <operator-token>
X-Tenant-Id: 550e8400-e29b-41d4-a716-446655440000

Response:
HTTP/1.1 200 OK
Content-Type: application/json

{
  "signals": [...],
  "next_cursor": "next_page_token_xyz",
  "has_more": true
}
```

**Step 3: Frontend loads next page**
```http
GET /api/connectors/signals?cursor=next_page_token_xyz&limit=20 HTTP/1.1
Authorization: Bearer <user-session-token>

Response:
HTTP/1.1 200 OK
Content-Type: application/json

{
  "signals": [...],
  "next_cursor": null,
  "has_more": false
}
```

### Example: Error Flow

**Step 1: Frontend request with invalid filter**
```http
GET /api/connectors/signals?connection_id=invalid-uuid HTTP/1.1
Authorization: Bearer <user-session-token>

Response:
HTTP/1.1 400 Bad Request
Content-Type: application/problem+json

{
  "code": "VALIDATION_ERROR",
  "message": "Invalid connection_id format",
  "details": {
    "field": "connection_id",
    "error": "Must be a valid UUID"
  }
}
```

**Step 2: Core handles Connectors error**
```http
# Core → Connectors request
GET /signals?connection_id=invalid-uuid HTTP/1.1
Authorization: Bearer <operator-token>
X-Tenant-Id: 550e8400-e29b-41d4-a716-446655440000

# Connectors → Core response
HTTP/1.1 400 Bad Request
Content-Type: application/problem+json

{
  "code": "VALIDATION_ERROR",
  "message": "Invalid connection_id format",
  "details": {
    "field": "connection_id",
    "error": "Must be a valid UUID"
  },
  "status": 400,
  "trace_id": "trace_789"
}

# Core → Frontend response (preserves structure)
HTTP/1.1 400 Bad Request
Content-Type: application/problem+json

{
  "code": "VALIDATION_ERROR",
  "message": "Invalid connection_id format",
  "details": {
    "field": "connection_id",
    "error": "Must be a valid UUID"
  }
}
```

---
