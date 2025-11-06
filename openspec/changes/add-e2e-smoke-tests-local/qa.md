# QA Review: add-e2e-smoke-tests-local

## Summary
- Total Issues Found: 7 (original 3 + 4 additional)
- Critical: 1 | High: 2 | Medium: 2 | Low: 2

## Detailed Reports

### Report 1 (Original)
Proposal assumed portpicker reduces all bind races but didn't specify retry. **RESOLVED**: Added explicit one‑retry strategy on bind failure with new port selection and child respawning.

### Report 2 (Original) 
Readiness gate is `/readyz` which depends on DB reachability, but acceptance criteria didn't specify standard timeout and error messaging. **RESOLVED**: Added 60s default timeout with 200-500ms backoff and detailed error reporting including DB URL and last `/readyz` status.

### Report 3 (Original)
Protected endpoint verification requires operator token in env; the proposal didn't define the fallback or failure mode. **RESOLVED**: Added requirement for `POBLYSH_OPERATOR_TOKEN` with fail-fast guidance and generated tenant UUID for complete auth flow testing.

### Report 4 (Additional)
Dependency specification listed reqwest as both existing and additional, creating confusion. **RESOLVED**: Clarified that reqwest is an existing dependency being leveraged, not an additional one.

### Report 5 (Additional)
Proposal didn't address security implications of port binding. **RESOLVED**: Added explicit 127.0.0.1 binding requirement to prevent network exposure during local testing.

### Report 6 (Additional)
Database validation strategy was unclear between Postgres and SQLite. **RESOLVED**: Defined Postgres as primary validation target with SQLite as secondary development convenience.

### Report 7 (Additional)
Missing environment pre-flight validation requirements. **RESOLVED**: Added comprehensive pre-flight checks for required environment variables with clear error guidance.

## Improvement Tasks

### Task 1: Add bind retry on conflict
**Priority**: High
**Files**: tests/e2e_smoke_tests.rs, openspec/changes/add-e2e-smoke-tests-local/proposal.md
**Status**: COMPLETED
**Action Required**: Attempt bind with portpicker; if child exits due to bind error, pick a new port and retry once before failing.

### Task 2: Standardize readiness timeout and messages
**Priority**: Medium
**Files**: tests/e2e_smoke_tests.rs, README
**Status**: COMPLETED
**Action Required**: Use a 60s readiness deadline with 200–500ms backoff between polls; on timeout, print DB URL, port, and last `/readyz` status/body.

### Task 3: Enforce operator token presence
**Priority**: Low
**Files**: tests/e2e_smoke_tests.rs, README
**Status**: COMPLETED
**Action Required**: Require `POBLYSH_OPERATOR_TOKEN` for the smoke test; if missing, exit with guidance to set a token.

### Task 4: Clarify dependency specification
**Priority**: Medium
**Files**: openspec/changes/add-e2e-smoke-tests-local/proposal.md
**Status**: COMPLETED
**Action Required**: Remove reqwest from "selected additional dev crates" section and clarify existing dependency usage.

### Task 5: Add security constraints for binding
**Priority**: High
**Files**: openspec/changes/add-e2e-smoke-tests-local/proposal.md, openspec/changes/add-e2e-smoke-tests-local/design.md
**Status**: COMPLETED
**Action Required**: Explicitly specify 127.0.0.1 binding only (not 0.0.0.0) to prevent network exposure during local testing.

### Task 6: Define database validation requirements
**Priority**: Medium
**Files**: openspec/changes/add-e2e-smoke-tests-local/proposal.md, openspec/changes/add-e2e-smoke-tests-local/tasks.md
**Status**: COMPLETED
**Action Required**: Specify Postgres as primary validation target with SQLite as secondary fallback, or require testing against both types.

### Task 7: Add explicit migration validation
**Priority**: Medium
**Files**: openspec/changes/add-e2e-smoke-tests-local/tasks.md
**Status**: COMPLETED
**Action Required**: Add task to verify schema consistency between Postgres and SQLite migrations during smoke testing.

## Review Notes

The proposal is now well-structured with comprehensive coverage of technical requirements and implementation details. All identified issues have been addressed:

1. **Port Management**: Clear specification of port selection, retry logic, and security constraints
2. **Dependency Clarity**: Removed ambiguity about dev-dependencies
3. **Security Considerations**: Explicit localhost-only binding for safe local testing
4. **Database Strategy**: Clear primary/secondary database validation approach
5. **Configuration Validation**: Comprehensive pre-flight environment checks
6. **Error Handling**: Detailed timeout, backoff, and error reporting requirements
7. **Authentication**: Complete auth flow testing with generated tenant IDs

The change represents a valuable addition to the development workflow with proper attention to security, reliability, and developer experience. All critical and high-priority issues have been resolved, with comprehensive implementation guidance provided in the updated tasks.