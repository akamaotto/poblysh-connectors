---
description: OpenSpec: Proposal QA Review
auto_execution_mode: 1
---

<!-- OPENSPEC:START -->
**Guardrails**
- Focus on finding genuine issues that would cause implementation problems, ambiguity, or inconsistency—avoid nitpicking style preferences.
- Be specific and actionable: every report must include exact file path, line number, the issue, and the precise correction needed.
- Prioritize consistency across documents: naming, response formats, error structures, and terminology must align throughout the proposal.
- Group related issues into coherent tasks rather than creating separate tasks for each minor occurrence of the same problem.
- Refer to `openspec/AGENTS.md` for OpenSpec conventions and standards if clarification is needed.

**Steps**
Track these steps as TODOs and complete them one by one.

1. **Context Gathering**:
   - Read `changes/<id>/proposal.md` to understand the overall goal and scope
   - Review `design.md` (if present) for architectural decisions and rationale
   - Identify all specification files in `changes/<id>/specs/` that need review

2. **Specification Validation**:
   - Check for naming consistency: environment variables, function names, endpoint paths, parameter names
   - Validate data structures: request/response schemas must be complete and internally consistent
   - Verify HTTP status codes follow REST conventions and are used consistently across endpoints
   - Ensure error response formats are uniform (consistent envelope structure across all error cases)
   - Confirm examples in specifications match the defined contracts exactly

3. **Cross-Document Consistency**:
   - Compare terminology usage across `proposal.md`, `design.md`, and all `spec.md` files
   - Check that configuration values (env vars, constants, defaults) match throughout all documents
   - Verify examples in different files don't contradict each other
   - Ensure referenced identifiers (IDs, names, paths) are consistent everywhere they appear

4. **Completeness Check**:
   - Identify missing edge cases or error scenarios in specifications
   - Flag undefined behavior or ambiguous requirements
   - Note missing validation rules, constraints, or business logic specifications
   - Check that all acceptance criteria in `proposal.md` are covered in specifications

5. **Generate QA Reports**:
   - For each issue found, create a report following this exact format:
     ```
     In [file-path] around line [number], [describe issue clearly]; [provide exact correction with concrete examples], and [note related locations needing same fix if applicable].
     ```
   - Example: "In openspec/changes/add-webhook-signature-verification/specs/api-webhooks/spec.md around line 37, the environment variable name `POBLYSH_WEBHOOK_GITHUB_SECRET` appears to be a typo; replace it with the correct configured name (e.g., `PUBLISH_WEBHOOK_GITHUB_SECRET` or the actual environment variable used by the codebase) so the spec matches the real configuration, and update any other occurrences in the spec to the same correct name."

6. **Create Improvement Tasks**:
   - Group related issues into logical tasks (e.g., "Fix environment variable naming consistency" rather than separate tasks per occurrence)
   - Prioritize each task: **Critical** (blocks implementation), **High** (significant inconsistency), **Medium** (clarity improvement), **Low** (minor documentation)
   - Format each task as:
     ```markdown
     ### Task N: [Brief Description]
     **Priority**: [Critical/High/Medium/Low]
     **Files**: [List of affected files]
     **Issue**: [Concise problem statement]
     **Action Required**: [Specific steps to resolve]
     ```

7. **Generate Summary Report**:
   - Provide issue count by priority level (Critical, High, Medium, Low)
   - List all detailed reports sequentially
   - Include all improvement tasks with clear priorities
   - Add review notes highlighting patterns or systemic issues observed

**Reference**
- Use `openspec show <id>` to review the full change context
- Use `openspec show <id> --json --deltas-only` for structured access to proposal details
- Check `openspec/AGENTS.md` for OpenSpec specification standards and conventions
- Review similar past changes with `openspec list --status completed` to understand established patterns

**Output Format**
```markdown
# QA Review: [Change ID/Name]

## Summary
- Total Issues Found: [number]
- Critical: [number] | High: [number] | Medium: [number] | Low: [number]

## Detailed Reports

### Report 1
[Precise report with file path, line number, issue, and exact correction]

### Report 2
[Continue for all issues found...]

## Improvement Tasks

### Task 1: [Brief Description]
**Priority**: [Level]
**Files**: [Affected files]
**Issue**: [Problem statement]
**Action Required**: [Specific resolution steps]

[Continue for all tasks...]

## Review Notes
[Observations, patterns, or systemic issues noticed]
```
<!-- OPENSPEC:END -->