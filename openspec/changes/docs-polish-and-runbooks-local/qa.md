# QA Review: docs-polish-and-runbooks-local

## Summary
- Total Issues Found: 2
- Critical: 0 | High: 0 | Medium: 1 | Low: 1

## Detailed Reports

### Report 1
In openspec/changes/docs-polish-and-runbooks-local/proposal.md around line 10, “Affected specs: none directly (doc/runbook only)” may cause confusion about OpenAPI/API docs scope; add an explicit note that no OpenAPI schema changes are included and any API doc edits (utoipa annotations) will be proposed separately as needed.

### Report 2
In openspec/changes/docs-polish-and-runbooks-local/design.md around line 35, the references list crate names and versions but omits direct links; append crates.io reference links for quick reviewer validation.

## Improvement Tasks

### Task 1: Clarify OpenAPI scope
**Priority**: Medium
**Files**: openspec/changes/docs-polish-and-runbooks-local/proposal.md
**Issue**: Ambiguity whether API schema changes are part of this change
**Action Required**: Add a sentence under Impact clarifying that OpenAPI/utoipa annotations are unchanged; any schema edits will be proposed separately

### Task 2: Add crate reference links
**Priority**: Low
**Files**: openspec/changes/docs-polish-and-runbooks-local/design.md
**Issue**: Missing direct links to crates for version verification
**Action Required**: Append crates.io URLs for `aes-gcm`, `zeroize`, `base64`, `hkdf`

## Review Notes
- Env var naming (`POBLYSH_CRYPTO_KEY`) is consistent across files and matches the related change (`add-local-token-encryption`).
- Runbook clearly separates local/dev reset path from a future preserve‑tokens path; suitable for this change’s scope.

