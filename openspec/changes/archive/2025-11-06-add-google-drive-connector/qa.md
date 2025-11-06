# QA Review: add-google-drive-connector

## Summary
- Total Issues Found: 4
- Critical: 0 | High: 1 | Medium: 2 | Low: 1

## Detailed Reports

### Report 1
In openspec/changes/add-google-drive-connector/specs/connectors/spec.md around line 33, the handling of webhook headers is described as "payload MAY include a `headers` object" without specifying a required convention; explicitly require that the platform forwards Google headers into `payload.headers` so the `Connector::handle_webhook(payload)` signature can access them, and mirror this wording in the acceptance criteria if used for testing.

### Report 2
In openspec/changes/add-google-drive-connector/specs/connectors/spec.md around line 60, rate‑limit behavior mentions a standardized error but did not name it prior to revision; ensure consistency with other connectors by explicitly using `SyncError::RateLimited { retry_after_secs }` (aligned with the GitHub connector spec). This was updated; verify no lingering references remain.

### Report 3
In openspec/changes/add-google-drive-connector/proposal.md around line 18, acceptance criteria require persisting connections on token exchange, but the repository does not yet expose OAuth endpoints or persistence wiring for this provider; clarify this relies on existing OAuth endpoint changes and that the connector implementation can stub behavior until those changes are merged.

### Report 4
In openspec/changes/add-google-drive-connector/specs/connectors/spec.md around line 46, the `file_moved` mapping is declared but later described as treated as `file_updated` when ambiguous; clarify the MVP stance by either removing the `file_moved` kind from MVP or explicitly stating that moves are mapped to `file_updated` until parent comparisons are implemented.

## Improvement Tasks

### Task 1: Define webhook header forwarding convention
**Priority**: High
**Files**: openspec/changes/add-google-drive-connector/specs/connectors/spec.md
**Issue**: `handle_webhook` only receives payload JSON, while Google sends critical data in headers.
**Action Required**: Add explicit requirement that the platform forwards Google headers into `payload.headers.*` with canonicalized header names (e.g., `x-goog-channel-id`, `x-goog-resource-id`, `x-goog-resource-state`, `x-goog-message-number`).

### Task 2: Align `file_moved` semantics for MVP
**Priority**: Medium
**Files**: openspec/changes/add-google-drive-connector/specs/connectors/spec.md
**Issue**: `file_moved` is named but not practically detectable without parent comparison.
**Action Required**: Declare that moves map to `file_updated` in MVP and reserve `file_moved` for a later change when parent lookup exists.

### Task 3: Cross‑change dependency note for OAuth persistence
**Priority**: Medium
**Files**: openspec/changes/add-google-drive-connector/proposal.md
**Issue**: Proposal assumes OAuth endpoints/persistence wired for this provider.
**Action Required**: Add a note referencing the OAuth start/callback change and state that Drive will reuse those endpoints; implementation may stub until merged.

### Task 4: Naming consistency guidance
**Priority**: Low
**Files**: openspec/changes/add-google-drive-connector/proposal.md
**Issue**: Provider naming varies across docs (e.g., `google-workspace` vs `google-drive`).
**Action Required**: Clarify that this change introduces the connector `google-drive` and that a future consolidation may align provider names across the Providers API and registry.

## Review Notes
- The spec now reuses the standardized `SyncError::RateLimited` shape for rate limits to align with other connectors.
- Clear, monotonic cursoring guidance via Drive `pageToken` is in place; initial baseline behavior is acceptable for MVP.
