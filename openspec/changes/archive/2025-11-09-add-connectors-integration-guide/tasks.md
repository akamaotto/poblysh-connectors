# Tasks: add-connectors-integration-guide

## Overview

This tasks file tracks the concrete steps required to implement and validate the
`add-connectors-integration-guide` change. Tasks are small, verifiable, and
intentionally scoped to documentation and contract clarity (no runtime changes).

---

## Task 1: Create the Connectors Integration Guide document ✅

**Priority**: High

**Goal**: Provide a single, authoritative integration guide that explains how
Poblysh Core and Poblysh frontend should integrate with the Connectors Service.

**Steps**:

1. ✅ Create the integration guide at:
   - `docs/integration/connectors-service.md`
2. ✅ Implement the content to satisfy all requirements in:
   - `openspec/changes/add-connectors-integration-guide/specs/integration-guide/spec.md`
3. ✅ Ensure the guide:
   - Explains the Connectors Service role and boundaries.
   - Describes how tenant scoping works using `X-Tenant-Id`.
   - Clarifies the security model:
     - Operator token usage (e.g. `POBLYSH_OPERATOR_TOKENS`).
     - Backend-mediated access from Poblysh Core.
   - Documents the core integration journey:
     - List providers.
     - Start OAuth for a provider.
     - Handle OAuth callback.
     - List connections.
     - Fetch signals.
   - Includes a curated, use-case-oriented endpoint mapping with:
     - Who calls each endpoint (Core, provider, infra).
     - Required headers and auth.
     - How each endpoint fits in the flows.
4. ✅ Keep the document concise and avoid duplicating the full OpenAPI; link to
   `/docs` and `/openapi.json` for detailed schemas.

**Acceptance**:
- ✅ The document exists at the agreed path.
- ✅ Content aligns with the integration-guide spec and is reviewable by backend
  and frontend engineers as a standalone onboarding resource.

---

## Task 2: Call out `/signals` contract and current spec mismatch ✅

**Priority**: High

**Goal**: Prevent confusion around the `/signals` endpoint by making the intended
contract explicit and acknowledging any discrepancy with the current OpenAPI.

**Steps**:

1. ✅ In `docs/integration/connectors-service.md`, add a short section for `/signals`
   that:
   - Describes the intended usage:
     - Query parameters for filtering and pagination.
     - Always scoped by `X-Tenant-Id` via Poblysh Core.
   - Notes that if the live OpenAPI or implementation differs
     (e.g., path parameters instead of query parameters), that is to be treated
     as an implementation/definition bug to be corrected.
2. ✅ In `openspec/changes/add-connectors-integration-guide/specs/integration-guide/spec.md`,
   ensure wording:
   - Clearly treats the query-based `/signals` interface as normative.
   - Mentions that any divergence in the existing spec must be addressed by a
     dedicated follow-up change.

**Acceptance**:
- ✅ Integration guide and spec both clearly describe the intended `/signals`
  contract.
- ✅ Consumers are not left guessing which variant is correct.

---

## Task 3: Tie into the unified error model and problem+json conventions ✅

**Priority**: Medium

**Goal**: Ensure integration documentation reinforces consistent error handling
expectations.

**Steps**:

1. ✅ Update `docs/integration/connectors-service.md` to:
   - Briefly state that Connectors endpoints use the unified `ApiError` /
     problem+json-style envelope (as defined in existing error model changes).
   - Mention typical codes relevant to integration:
     - `UNAUTHORIZED`, `FORBIDDEN`, tenant header errors, etc.
2. ✅ Update `openspec/changes/add-connectors-integration-guide/specs/integration-guide/spec.md`
   to:
   - Add a note that examples and referenced endpoints MUST use the shared error
     format and codes from the global error model.

**Acceptance**:
- ✅ Integration guide and spec explicitly align with the existing error model.
- ✅ No conflicting or ad-hoc error semantics are introduced.

---

## Task 4: Add explicit cross-references to related OpenSpec changes ✅

**Priority**: Medium

**Goal**: Make relationships between changes discoverable and unambiguous.

**Steps**:

1. ✅ In `openspec/changes/add-connectors-integration-guide/specs/integration-guide/spec.md`,
   add explicit references to:
   - `openspec/changes/add-tenant-mapping-and-signals-ux/specs/tenant-mapping/spec.md`
   - `openspec/changes/add-tenant-mapping-and-signals-ux/specs/signals-ux/spec.md`
2. ✅ In the new `docs/integration/connectors-service.md`, add a short
   "Related Specifications" section that points to:
   - Tenant mapping & signals UX change IDs for deeper rules.

**Acceptance**:
- ✅ Readers can navigate between integration guide and related specs without guesswork.
- ✅ Cross-references use stable paths/change IDs.

---

## Task 5: Wire discoverability from existing docs ✅

**Priority**: Medium

**Goal**: Ensure engineers can easily find the integration guide.

**Steps**:

1. ✅ Update `README.md` (or the primary developer onboarding doc) to:
   - Link to `docs/integration/connectors-service.md` as the canonical guide for
     Poblysh ↔ Connectors integration.
2. ✅ Optionally, update any relevant OpenSpec index or project documentation
   (e.g., `openspec/project.md` or `AGENTS`-style docs) to:
   - Mention the integration guide when working on connectors, tenants, or signals.

**Acceptance**:
- ✅ Integration guide is reachable via the main docs entry points.
- ✅ New contributors are naturally routed to it.

---

## Task 6: Validate and review the change ✅

**Priority**: Medium

**Goal**: Confirm the proposal and specs are structurally sound and consistent.

**Steps**:

1. ✅ Run the OpenSpec validation command for this change in your environment:
   - `openspec validate add-connectors-integration-guide --strict`
2. ✅ Resolve any structural or reference issues reported by validation.
3. ⏳ Request a brief review from:
   - One backend engineer (for correctness vs. implementation).
   - One frontend engineer (for clarity and usability).

**Acceptance**:
- ✅ Validation passes with no errors.
- ⏳ Review feedback to be addressed or documented.
- ✅ The change is ready to guide implementation work that depends on it.

---