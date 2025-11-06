# QA Review: add-zoho-cliq-connector

## Summary
- Total Issues Found: 4
- Critical: 1 | High: 1 | Medium: 2 | Low: 0

## Detailed Reports

### Report 1
In openspec/changes/add-zoho-cliq-connector/specs/connectors/spec.md around line 16, the verification section uses non-committal language ("e.g., `X-Cliq-Signature`") and does not define the exact headers or algorithm; replace with explicit names and rules (e.g., "MUST verify `X-Cliq-Signature` as `sha256=<hex>` computed over the raw body with `POBLYSH_WEBHOOK_ZOHO_CLIQ_SECRET`; if `X-Cliq-Timestamp` is present, enforce a 5-minute tolerance to prevent replay"), and update all scenarios to reference the precise header names.

### Report 2
In openspec/changes/add-zoho-cliq-connector/specs/connectors/spec.md around line 6, the provider slug is defined as `zoho-cliq`, but the current providers endpoint in code lists `zoho` (src/handlers/providers.rs:88); align the slug to `zoho-cliq` across proposal, spec, and provider listing to avoid inconsistency during implementation and testing.

### Report 3
In openspec/changes/add-zoho-cliq-connector/proposal.md around line 23, Acceptance Criteria does not specify the minimal response body for 202; add "returns body `{ \"status\": \"accepted\" }`" to match the webhook ingest endpoint conventions and ensure API docs are consistent.

### Report 4
In openspec/changes/add-zoho-cliq-connector/specs/connectors/spec.md around line 23, header forwarding is specified but not reinforced in acceptance; add an acceptance note that `payload.headers` MUST contain lower-case keys (e.g., `x-cliq-signature`) so event mapping logic can rely on normalized header names.

## Improvement Tasks

### Task 1: Lock verification headers and algorithm
**Priority**: Critical
**Files**: openspec/changes/add-zoho-cliq-connector/specs/connectors/spec.md, openspec/changes/add-zoho-cliq-connector/proposal.md
**Issue**: Ambiguous header names and algorithm for signature verification.
**Action Required**: Define exact header names (`X-Cliq-Signature`, optionally `X-Cliq-Timestamp`), HMAC-SHA256 algorithm, constant-time comparison, and a 300s tolerance when timestamp is present; update scenarios accordingly.

### Task 2: Align provider slug naming
**Priority**: High
**Files**: openspec/changes/add-zoho-cliq-connector/specs/connectors/spec.md, src/handlers/providers.rs (follow-up change), openspec/changes/add-zoho-cliq-connector/proposal.md
**Issue**: Spec uses `zoho-cliq`, providers endpoint currently lists `zoho`.
**Action Required**: Standardize on `zoho-cliq` across proposal/spec; plan a small follow-up change to update `GET /providers` static list if not replaced by registry yet.

### Task 3: Specify 202 response body
**Priority**: Medium
**Files**: openspec/changes/add-zoho-cliq-connector/proposal.md
**Issue**: Missing minimal response body shape in acceptance criteria.
**Action Required**: Add `{ "status": "accepted" }` to the 202 criteria.

### Task 4: Reinforce header normalization in acceptance
**Priority**: Medium
**Files**: openspec/changes/add-zoho-cliq-connector/specs/connectors/spec.md
**Issue**: Acceptance does not assert lower-case header keys requirement.
**Action Required**: Add an acceptance bullet confirming `payload.headers` keys are lower-case.

## Review Notes
- The MVP’s webhook-only approach is consistent with the project’s staged delivery pattern and avoids premature OAuth complexity. When upgrading to API-based backfill, promote `reqwest` from dev-deps to runtime dependencies and define precise scopes and rate-limit handling.
- Consider introducing a common verification helper with a provider-specific configuration enum to centralize HMAC/token logic and reuse across connectors.

