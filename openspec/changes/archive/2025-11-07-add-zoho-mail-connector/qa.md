# QA Review: add-zoho-mail-connector

## Summary
- Total Issues Found: 3
- Critical: 0 | High: 1 | Medium: 2 | Low: 0

## Detailed Reports

### Report 1
In openspec/changes/add-zoho-mail-connector/specs/connectors/spec.md around line 21, dedupe key references `last_modified` while the provider field elsewhere is `lastModifiedTime`; use one term consistently. Replace `last_modified` with `lastModifiedTime` in the dedupe key description, and update any other occurrences to match the provider field name.

### Report 2
In openspec/changes/add-zoho-mail-connector/proposal.md around line 101, the dedupe key uses `hash(message_id || last_modified)`; align to `hash(message_id || lastModifiedTime)` to match the provider field and the spec scenarios.

### Report 3
In openspec/changes/add-zoho-mail-connector/proposal.md around line 94, the env var `POBLYSH_ZOHO_DC` lists US/EU/IN but omits other known DCs (e.g., AU, JP). Clarify that values beyond `us|eu|in` are supported via a resolver mapping (e.g., `au`, `jp`) and ensure the design explicitly mentions extendable mapping.

## Improvement Tasks

### Task 1: Normalize dedupe field naming
**Priority**: High
**Files**: openspec/changes/add-zoho-mail-connector/specs/connectors/spec.md, openspec/changes/add-zoho-mail-connector/proposal.md
**Issue**: Mixed use of `last_modified` and `lastModifiedTime` creates ambiguity in dedupe key definition.
**Action Required**: Replace all `last_modified` references with `lastModifiedTime`; ensure normalized payload fields can still expose a `last_modified` if desired, but dedupe key and cursor use the provider field name consistently.

### Task 2: Expand DC mapping note
**Priority**: Medium
**Files**: openspec/changes/add-zoho-mail-connector/proposal.md, openspec/changes/add-zoho-mail-connector/design.md
**Issue**: Region/DC configuration examples omit some DCs.
**Action Required**: Clarify that `POBLYSH_ZOHO_DC` supports additional DCs (e.g., `au`, `jp`) via a resolver mapping and link to Zoho docs for the full list.

### Task 3: Explicit rate limit behavior in examples
**Priority**: Medium
**Files**: openspec/changes/add-zoho-mail-connector/specs/connectors/spec.md
**Issue**: The scenario mentions a typed rate limit error but lacks a concrete example of `Retry-After` handling.
**Action Required**: Add an example note in the spec scenario or proposal indicating that `Retry-After` header (seconds or HTTP date) is parsed and converted to `retry_after_secs`.

## Review Notes
- Overall structure aligns with existing connector changes (Gmail, Google Drive) and follows OpenSpec conventions.
- The research plan is targeted for this provider and should validate exact list/search params for time-based filters.
- Ensure identifier stability for dedupe (prefer provider message id + lastModifiedTime) and document fallback if thread id is not present.

