# tenant-mapping Specification

## Purpose
TBD - created by archiving change add-tenant-mapping-and-signals-ux. Update Purpose after archive.
## Requirements
### Requirement: Canonical Tenant Identifier for Connectors

The system MUST have a canonical tenant identifier used for all interactions with the Connectors Service.

#### Environment-Specific Strategy:

**Local Development:**
- Use Poblysh tenant UUID directly as `X-Tenant-Id`
- Simplifies debugging and testing
- No security concerns in local environment

**Test Environment:**
- Use Poblysh tenant UUID directly as `X-Tenant-Id`
- Enables straightforward integration testing
- Test tenants are isolated by design

**Staging Environment:**
- Use Poblysh tenant UUID directly as `X-Tenant-Id`
- Maintains consistency with production approach
- Allows realistic testing of tenant isolation

**Production Environment:**
- Use Poblysh tenant UUID directly as `X-Tenant-Id`
- Chosen for simplicity and security
- Poblysh tenant UUIDs are stable, unpredictable UUIDs safe for internal service communication
- No additional mapping complexity required

**Ownership and Stability:**
- The mapping strategy is owned by Poblysh Core
- Once established, the mapping between Poblysh tenant and `X-Tenant-Id` MUST be stable
- Any changes to mapping strategy require a controlled migration process affecting all environments

#### Scenarios:

1) #### Scenario: Using Poblysh Tenant ID directly (chosen approach)
   - Given Poblysh Core issues a stable UUID as the tenant identifier
   - And this identifier is safe to use across internal services (confirmed for production)
   - When Poblysh Core calls any Connectors tenant-scoped endpoint
   - Then it MUST:
     - Set `X-Tenant-Id` to that Poblysh tenant UUID
     - Treat this value as the canonical Connectors tenant identifier
   - And all connections, jobs, and signals for that tenant MUST be scoped to this same `X-Tenant-Id`.

2) #### Scenario: Using a distinct Connectors Tenant ID (fallback)
   - Given Poblysh Core cannot or SHOULD NOT expose its internal tenant ID directly
   - When establishing a relationship with the Connectors Service
   - Then Poblysh Core MUST:
     - Generate a stable UUID as `connectors_tenant_id`
     - Persist a 1:1 mapping: `poblysh_tenant_id` ↔ `connectors_tenant_id`
   - And when calling Connectors tenant-scoped endpoints
     - It MUST set `X-Tenant-Id = connectors_tenant_id`
   - And the mapping MUST be immutable once established, except under a controlled migration process.

---

### Requirement: Mandatory `X-Tenant-Id` for Tenant-Scoped Endpoints

All tenant-scoped operations in the Connectors Service MUST be explicitly scoped by `X-Tenant-Id`.

#### Scenarios:

1) #### Scenario: Tenant-scoped read operations
   - When Poblysh Core calls a tenant-scoped read endpoint, such as:
     - `GET /connections`
     - `GET /signals` (and similar)
   - Then the request MUST include:
     - `X-Tenant-Id` set to the canonical Connectors tenant identifier
   - And the response data MUST:
     - Include only resources belonging to that `X-Tenant-Id`.

2) #### Scenario: Tenant-scoped write or mutation operations
   - When Poblysh Core initiates a tenant-scoped mutation, such as:
     - Starting an OAuth flow: `POST /connect/{provider}`
     - Accepting or associating webhooks for a tenant
   - Then the request MUST include:
     - `X-Tenant-Id` set to the canonical Connectors tenant identifier
   - And the Connectors Service MUST:
     - Persist any created connections or related entities under that `X-Tenant-Id`.

3) #### Scenario: Missing or invalid `X-Tenant-Id`
   - When a request targets a tenant-scoped endpoint without a valid `X-Tenant-Id`
   - Then the Connectors Service MUST:
     - Reject the request with a 400-class error using the unified `ApiError` envelope
     - Use the existing validation error pattern with code `VALIDATION_ERROR` and appropriate details
     - Include specific details about the X-Tenant-Id validation failure in the error details field
     - Never infer or default the tenant ID from other fields
   - **Missing header example:** Request without `X-Tenant-Id` returns `400 VALIDATION_ERROR`
   - **Invalid format example:** Request with `X-Tenant-Id=12345` (non-UUID) returns `400 VALIDATION_ERROR` with details explaining the malformed UUID
   - **Validation logic:** MUST validate that `X-Tenant-Id` is present AND is a valid UUID format when UUID strategy is used

#### Concrete Error Examples:

**Missing `X-Tenant-Id` Header:**
```http
GET /connections HTTP/1.1
Authorization: Bearer <operator-token>

Response:
HTTP/1.1 400 Bad Request
Content-Type: application/problem+json

{
  "code": "VALIDATION_ERROR",
  "message": "Missing required header: X-Tenant-Id",
  "details": {
    "field": "X-Tenant-Id",
    "error": "Header is required for tenant-scoped operations"
  },
  "status": 400,
  "trace_id": "trace_12345"
}
```

**Invalid `X-Tenant-Id` Format:**
```http
GET /connections HTTP/1.1
Authorization: Bearer <operator-token>
X-Tenant-Id: 12345

Response:
HTTP/1.1 400 Bad Request
Content-Type: application/problem+json

{
  "code": "VALIDATION_ERROR",
  "message": "Invalid X-Tenant-Id format",
  "details": {
    "field": "X-Tenant-Id",
    "error": "X-Tenant-Id must be a valid UUID, received: 12345",
    "provided_value": "12345"
  },
  "status": 400,
  "trace_id": "trace_67890"
}
```

---

### Requirement: Single Source of Truth for Tenant Mapping

Poblysh Core MUST own and manage the mapping between Poblysh tenants and Connectors tenants.

#### Scenarios:

1) #### Scenario: Mapping ownership
   - Given multiple services interact with the Connectors Service
   - When they require a tenant identifier for Connectors
   - Then they MUST obtain it from Poblysh Core (or a shared identity service under Core’s control)
   - And they MUST NOT:
     - Locally invent or guess `X-Tenant-Id` values
     - Derive `X-Tenant-Id` purely from user context without Core’s mapping.

2) #### Scenario: Consistency across environments
   - When operating in `local`, `test`, `staging`, or `prod` environments
   - Then the mapping strategy (direct reuse vs. mapping table) MUST be:
     - Documented as specified in the Environment-Specific Strategy section
     - Consistent within each environment
   - And any environment-specific differences MUST NOT:
     - Change semantics of `X-Tenant-Id` for a given Connectors instance
   - **Error consistency:** All environments MUST return the same error format and codes for `X-Tenant-Id` validation failures

---

### Requirement: Isolation of Tenant Data

The Connectors Service MUST enforce strict isolation of tenant data based on `X-Tenant-Id`.

#### Scenarios:

1) #### Scenario: Cross-tenant isolation
   - Given two different tenants with different `X-Tenant-Id` values
   - When querying connections or signals with `X-Tenant-Id = A`
   - Then no data associated with tenant `B` MUST be returned.

2) #### Scenario: Webhook-scoped isolation
   - When a webhook is ingested for a specific tenant via:
     - Authenticated route with `X-Tenant-Id`
     - Or public route that includes a tenant identifier in the path or validated signature context
   - Then all resulting jobs, connections (if applicable), and signals MUST:
     - Be associated with the correct `X-Tenant-Id`
     - Be visible only via that same `X-Tenant-Id`.

---

### Requirement: Clear Separation of User Auth and Tenant Mapping

End-user authentication MUST be handled by Poblysh Core, and MUST NOT leak into the Connectors tenancy model.

#### Scenarios:

1) #### Scenario: Backend-mediated calls
   - When the Poblysh frontend initiates an action (connect provider, list signals)
   - Then:
     - Poblysh Core authenticates the user
     - Resolves the tenant and corresponding Connectors tenant identifier
     - Calls Connectors with:
       - `Authorization: Bearer <operator_token>` (per security spec)
       - `X-Tenant-Id: <canonical connectors tenant id>`
   - And:
     - The frontend MUST NOT send user tokens directly to the Connectors Service.
     - The Connectors Service MUST treat `Authorization` as operator-level, not end-user-level.

2) #### Scenario: Avoiding mixed concerns
   - When designing new Connectors endpoints
   - Then:
     - Tenant scoping MUST rely solely on `X-Tenant-Id` (and validated webhook signatures where applicable)
     - User identity MUST remain a Poblysh Core concern.

---

