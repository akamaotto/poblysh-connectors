# QA Review: add-google-calendar-connector

## Summary
- Total Issues Found: 0
- Critical: 0 | High: 0 | Medium: 0 | Low: 0

> NOTE: Automated `openspec validate` was run and passed. The reports below are historical findings from an earlier review; as of 2025-11-06 the proposal and spec have been updated to address the items (header normalization and `calendarId=primary` are present and the proposal defers `event_created`). No blocking issues remain.

## Detailed Reports

### RESOLVED - Report 1
Status: Resolved in current proposal/spec (creation vs update disambiguation is deferred; `event_created` is not required for MVP).
Historical note: An earlier draft referenced `event_created`; the current `proposal.md` and `spec.md` consistently map changes to `event_updated`/`event_deleted` and defer creation detection to a follow-up change.

### RESOLVED - Report 2
Status: Resolved in current `spec.md` — the webhook handling requirement now explicitly requires lower-case header keys (e.g., `x-goog-resource-state`).

### RESOLVED - Report 3
Status: Resolved in current `spec.md` — the incremental sync section explicitly states `calendarId=primary` for the MVP.

## Improvement Tasks

### Task 1: Align event mapping and acceptance
**Priority**: High
**Files**: openspec/changes/add-google-calendar-connector/proposal.md, openspec/changes/add-google-calendar-connector/specs/connectors/spec.md
**Issue**: Proposal promises `event_created` mapping but spec defers creation vs update disambiguation.
**Action Required**: Either (A) remove `event_created` from MVP and acceptance criteria, or (B) define a JSON-encoded cursor `{ syncToken, since }` and specify creation detection (`item.created >= since`) to emit `event_created`; apply consistently.

### Task 2: Specify header normalization for webhook payload
**Priority**: Medium
**Files**: openspec/changes/add-google-calendar-connector/specs/connectors/spec.md
**Issue**: Header forwarding lacks normalization rules.
**Action Required**: Require lower‑case header keys in `payload.headers` and reference example keys (`x-goog-channel-id`, `x-goog-resource-state`, etc.). Update scenario to mention lower‑case keys.

### Task 3: Clarify primary calendar identifier
**Priority**: Low
**Files**: openspec/changes/add-google-calendar-connector/specs/connectors/spec.md
**Issue**: "Primary calendar only" lacks the concrete identifier.
**Action Required**: Specify `calendarId=primary` for MVP in the sync Details section.

## Review Notes
- The MVP approach to treat unknown change types as `event_updated` is reasonable; consider upgrading to a composite cursor with `since` to enable `event_created` without additional storage changes in a follow‑up change.
- Naming and structure align with the Google Drive connector change; keep provider slug hyphenated (`google-calendar`) for consistency across connectors.
