# QA Review: add-local-run-scripts

## Summary
- Total Issues Found: 4
- Critical: 1 | High: 2 | Medium: 1 | Low: 0

## Detailed Reports

### Report 1
In openspec/changes/add-local-run-scripts/proposal.md around line 9, the environment variable name `DATABASE_URL` is inconsistent with the project’s `POBLYSH_*` schema; replace `DATABASE_URL` with `POBLYSH_DATABASE_URL` to match `src/config/mod.rs` expectations, and update any other occurrences in this change accordingly.

### Report 2
In openspec/changes/add-local-run-scripts/proposal.md around lines 8 and 29, the SQLite DSN is shown as `sqlite:dev.db` which is not the standard SQLx/SeaORM DSN format; replace with `sqlite://dev.db` everywhere, and apply the same correction in `openspec/changes/add-local-run-scripts/tasks.md:4` and `openspec/changes/add-local-run-scripts/design.md:34` for consistency and correctness.

### Report 3
In openspec/changes/add-local-run-scripts/proposal.md under the `env` task description (around line 7), it doesn’t explicitly include `POBLYSH_OPERATOR_TOKEN=...`, which is required by `AppConfig::validate()` for `local`/`test` profiles. Add that explicit variable so `cargo run` succeeds on a fresh setup without additional manual steps.

### Report 4
In openspec/changes/add-local-run-scripts/design.md around lines 31–36, the `db-sqlite` step mentions ensuring `dev.db` exists but does not clarify that the DSN should be `sqlite://dev.db`. Add that DSN example to avoid ambiguity and to align with the corrected DSN format across the proposal.

## Improvement Tasks

### Task 1: Fix env var naming to POBLYSH_
**Priority**: Critical
**Files**: openspec/changes/add-local-run-scripts/proposal.md
**Issue**: Inconsistent env var naming (`DATABASE_URL` vs `POBLYSH_DATABASE_URL`)
**Action Required**: Replace with `POBLYSH_DATABASE_URL` and re-check all occurrences

### Task 2: Standardize SQLite DSN format
**Priority**: High
**Files**: openspec/changes/add-local-run-scripts/proposal.md, openspec/changes/add-local-run-scripts/tasks.md, openspec/changes/add-local-run-scripts/design.md
**Issue**: Incorrect DSN `sqlite:dev.db`
**Action Required**: Replace all with `sqlite://dev.db`; note it in design

### Task 3: Include operator token in env scaffolding
**Priority**: High
**Files**: openspec/changes/add-local-run-scripts/proposal.md
**Issue**: Missing explicit `POBLYSH_OPERATOR_TOKEN`, causing validation failure
**Action Required**: Add the variable to the `env` task description

### Task 4: Clarify DSN in db-sqlite description
**Priority**: Medium
**Files**: openspec/changes/add-local-run-scripts/design.md
**Issue**: Ambiguous DSN in narrative
**Action Required**: Add explicit `sqlite://dev.db` wording in the `db-sqlite` step

## Review Notes
- Good alignment with current config loader and migration approach; keeping Postgres optional minimizes friction for first-run.
- Mirrored Makefile/Justfile targets provide a consistent DX; ensure `help` text is included per acceptance criteria when implementing.
