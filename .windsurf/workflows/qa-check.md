---
description: qa regression pass for connectors changes
auto_execution_mode: 1
---

**Review like a senior engineer: understand context, trace dependencies, explain why issues matter.**

## Process

### 1. Context & Requirements
- Read `changes/<id>/proposal.md`, `design.md`, `tasks.md` for full context
- Map code dependencies and downstream impacts (code graph analysis)
- Review git history for patterns in similar changes
- Verify all tasks marked complete

### 2. Build & Static Analysis
```bash
cargo check --all-targets && cargo clippy -- -D warnings
cargo fmt --check && cargo test -q
cargo audit  # security scan
```

### 3. Deep Code Review
- **Logic**: Validate correctness, error handling, edge cases
- **Security**: Check injection vectors, auth/authz, input validation, secrets exposure
- **Architecture**: Verify patterns, separation of concerns, performance bottlenecks
- **Spec Alignment**: Validate API contracts, response shapes, status codes, headers, trace propagation

### 4. Test Quality
- Verify unit/integration coverage for new code
- Check edge cases and error paths tested
- Identify missing critical test scenarios

### 5. Report Findings
For each issue provide:
- **Severity** (Critical/Important/Minor) + **Category** (Security/Logic/Performance)
- **Location**: `file.rs:line` with context
- **Impact**: How it affects the system
- **Fix**: Specific, actionable recommendation
- **Why**: Educational explanation of the underlying principle

**Final Verdict**: APPROVE / REQUEST CHANGES / REJECT with deployment readiness checklist

---
*Principles: Context over syntax • Education over criticism • Signal over noise • Actionable feedback*