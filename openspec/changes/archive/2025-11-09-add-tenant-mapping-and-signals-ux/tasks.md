# Tasks: add-tenant-mapping-and-signals-ux

## Overview

This tasks file breaks down the work required to implement the
`add-tenant-mapping-and-signals-ux` change proposal and its specs:

- `specs/tenant-mapping/spec.md`
- `specs/signals-ux/spec.md`

Each task should be small, verifiable, and aligned with existing
OpenSpec conventions and the current implementation.

---

## Task 1: Confirm and Document Tenant Mapping Strategy per Environment ✅

**Priority**: High

**Description**: Decide whether to use the Poblysh tenant UUID directly
as `X-Tenant-Id` or introduce a separate `connectors_tenant_id`, and
document this for each environment (`local`, `test`, `staging`, `prod`).

**Steps**:
1. For each environment, choose one of:
   - Direct reuse of Poblysh tenant UUID as `X-Tenant-Id`; or
   - 1:1 mapping table: `poblysh_tenant_id` ↔ `connectors_tenant_id`.
2. Update `specs/tenant-mapping/spec.md` to:
   - Explicitly describe the chosen strategy per environment.
   - Clarify that the mapping is owned by Poblysh Core (or a shared
     identity service).
3. Ensure the spec states that once a mapping is created, it MUST be
   stable and only change via a controlled migration process.

**Validation**:
- Mapping rules are unambiguous and environment-specific notes are
  included.
- Reviewed by owners of Poblysh Core and Connectors.


## Task 2: Define Deterministic Error Semantics for `X-Tenant-Id` ✅

**Priority**: High

**Description**: Make the behavior for missing/invalid `X-Tenant-Id`
explicit and aligned with the unified error model.

**Steps**:
1. Update `specs/tenant-mapping/spec.md` to:
   - Specify that tenant-scoped endpoints MUST return a deterministic
     status and error code for:
       - Missing `X-Tenant-Id`.
       - Malformed `X-Tenant-Id` (e.g., not a valid UUID when required).
   - Use the existing validation error pattern:
       - 400 status with `VALIDATION_ERROR` code and specific details
         about the X-Tenant-Id validation failure.
       - 401 only for missing/invalid operator auth, not tenant header.
2. Ensure language matches the existing `ApiError`/problem+json
   conventions defined in earlier changes.
3. Add at least one concrete example error body to the spec to make the
   contract obvious, showing the actual `VALIDATION_ERROR` pattern with details.

**Validation**:
- Error handling accurately reflects the current implementation's use of `validation_error` with 400 status codes.
- Examples match the global error model and existing OpenAPI patterns.


## Task 3: Reinforce Unified Error Model and Conventions ✅

**Priority**: Medium

**Description**: Tie tenant mapping and signals UX specs explicitly to
the standard error response and naming conventions.

**Steps**:
1. In `specs/tenant-mapping/spec.md` and `specs/signals-ux/spec.md`:
   - Ensure references to the unified `ApiError` model from the existing error-model change
     are accurate and reflect the actual implementation patterns.
   - State that all described behaviors MUST use that envelope and
     screaming-snake-case `code` values (e.g., `VALIDATION_ERROR`, `UNAUTHORIZED`).
2. Ensure scenarios referencing failures (auth, tenant, rate limits,
   etc.) use realistic codes and shapes consistent with the current
   OpenAPI and prior specs.

**Validation**:
- All spec examples use consistent error shapes/codes.
- No ad hoc or conflicting error formats are introduced.


## Task 4: Specify Core-Mediated Frontend Contract for Signals ✅

**Priority**: Medium

**Description**: Make the Core-facing signals contract for frontend
clients explicit and stable.

**Steps**:
1. In `specs/signals-ux/spec.md`:
   - Clearly describe the recommended Poblysh Core endpoint shape for
     frontend consumption (e.g. `GET /api/connectors/signals`).
   - Enumerate supported query parameters:
     - `provider`, `connection_id`, `kind`,
       `occurred_after`, `occurred_before`,
       `cursor`, `limit`.
2. Document that:
   - Frontend treats `cursor` as opaque.
   - Core is responsible for translating these into Connectors `/signals`
     calls with `X-Tenant-Id` and operator auth.
3. Ensure the spec emphasizes that any breaking changes in Connectors
   MUST be absorbed by Core to keep the frontend contract stable.

**Validation**:
- Frontend engineers can implement against Core’s API without reading
  Connectors internals.
- No contradictions with the integration-guide change.


## Task 5: Align `/signals` Behavior with Intended UX (Non-Breaking) ✅

**Priority**: Medium

**Description**: Reconcile the intended `/signals` usage described in the
spec with the current implementation/OpenAPI, without introducing
breaking changes inside this change.

**Steps**:
1. In `specs/signals-ux/spec.md`:
   - Clarify the intended `/signals` contract:
     - Filters and pagination via query parameters.
   - Add a note that any discrepancies in current OpenAPI or handlers
     SHOULD be corrected in a focused follow-up change (e.g.
     `update-signals-endpoint-shape`) and that this spec reflects the
     target behavior.
2. Ensure wording makes it clear this change is normative for UX and
   integration, while actual endpoint shape adjustments will be handled
   separately.

**Validation**:
- No confusion between current quirks and intended design.
- Clear pointer for future endpoint-shape clean-up work.


## Task 6: Harden Cross-References Between Related Specs ✅

**Priority**: Medium

**Description**: Replace vague cross-references with explicit paths and
change IDs for maintainability.

**Steps**:
1. In:
   - `specs/tenant-mapping/spec.md`
   - `specs/signals-ux/spec.md`
   - (and in the related `add-connectors-integration-guide` specs)
   update references like “see tenant-mapping spec” to:
   - “see `openspec/changes/add-tenant-mapping-and-signals-ux/specs/tenant-mapping/spec.md`”
   - “see `openspec/changes/add-tenant-mapping-and-signals-ux/specs/signals-ux/spec.md`”
   - “see `openspec/changes/add-connectors-integration-guide/specs/integration-guide/spec.md`”
2. Ensure cross-links form a coherent graph:
   - Integration guide → tenant mapping + signals UX.
   - Tenant mapping → integration guide (for context) + signals UX (for usage).
   - Signals UX → tenant mapping (for scoping rules).

**Validation**:
- All references resolve to exact files.
- No ambiguous “see above” or “see related change” language remains.


## Task 7: Add Concrete Request/Response Examples ✅

**Priority**: Medium

**Description**: Provide a few precise examples to make contracts
immediately actionable.

**Steps**:
1. In `specs/tenant-mapping/spec.md`:
   - Add example requests showing:
     - Valid `X-Tenant-Id` usage for `/connections` or `/signals`.
     - Error example for missing/invalid `X-Tenant-Id`.
2. In `specs/signals-ux/spec.md`:
   - Add an example:
     - Frontend → Core → Connectors sequence for listing signals with
       provider/connection filters and cursor.
3. Ensure examples use:
   - Realistic URLs.
   - Correct headers.
   - `ApiError`-compliant error bodies.

**Validation**:
- Examples are self-consistent and match the described requirements.
- Engineers can implement against them without guesswork.


## Task 8: Run OpenSpec Validation and Peer Review ✅

**Priority**: Medium

**Description**: Ensure structural and semantic consistency with
OpenSpec standards.

**Steps**:
1. Run strict validation for this change:
   - `openspec validate add-tenant-mapping-and-signals-ux --strict`
2. Resolve any reported issues:
   - Missing sections.
   - Broken references.
   - Invalid requirement formatting.
3. Request a brief peer review from:
   - A Connectors maintainer.
   - A Poblysh Core/Frontend representative to confirm that:
     - Flows are understandable.
     - Contracts are practical for implementation.

**Validation**:
- Validation passes with no errors.
- Review feedback is addressed or documented.


---