# QA Review: add-google-calendar-connector

## Summary
- Total Issues Found: 3
- Critical: 0 | High: 1 | Medium: 1 | Low: 1

## Detailed Reports

### Report 1
In openspec/changes/add-google-calendar-connector/proposal.md around line 11, the text says "Define normalized Signals for calendar events: `event_created`, `event_updated`, `event_deleted` mapped from sync results" while the delta spec maps only `event_deleted` explicitly and treats all other changes as `event_updated` (creation vs update disambiguation deferred); remove `event_created` from the MVP description and Acceptance Criteria, or define a precise disambiguation method (e.g., include `since` in a JSON cursor to compare against `item.created`), and apply the chosen approach consistently across the proposal and spec.

### Report 2
In openspec/changes/add-google-calendar-connector/specs/connectors/spec.md around line 31, the webhook handling requirement states headers must be forwarded into `payload.headers` but does not specify header key normalization; clarify that headers MUST be forwarded with lower‑case keys (e.g., `x-goog-resource-state`) to simplify parsing and mirror the Google Drive connector spec’s approach, and update the scenario wording to reference lower‑case keys.

### Report 3
In openspec/changes/add-google-calendar-connector/specs/connectors/spec.md around line 54, the scope "primary calendar only" is noted without stating how to reference it; add that the connector uses `calendarId=primary` for MVP to avoid ambiguity and ensure consistent API calls.

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

