---
name: OpenSpec: QA Check
description: Comprehensive quality assurance check for an OpenSpec change implementation.
category: OpenSpec
tags: [openspec, qa, validation, testing]
---
<!-- OPENSPEC:START -->
**Guardrails**
- Favor straightforward, minimal implementations first and add complexity only when it is requested or clearly required.
- Keep changes tightly scoped to the requested outcome.
- Refer to `openspec/AGENTS.md` (located inside the `openspec/` directory—run `ls openspec` or `openspec update` if you don't see it) if you need additional OpenSpec conventions or clarifications.

**Steps**
1. **Change Validation**:
   - Read `changes/<id>/proposal.md`, `design.md` (if present), and `tasks.md` to understand requirements
   - Verify all tasks in `tasks.md` are marked as completed
   - Check that the change exists and is not already archived

2. **Code Quality Checks**:
   - Run `cargo check` to ensure the project compiles without errors
   - Run `cargo clippy` for lint checks and resolve any warnings
   - Run `cargo fmt --check` to verify code formatting
   - Run `cargo test` to ensure all tests pass
   - Run `cargo build --release` to verify release build works

3. **Implementation Verification**:
   - Compare implemented code against tasks.md requirements
   - Verify all specified endpoints, functions, and features are implemented
   - Check that OpenAPI documentation is properly updated if required
   - Validate error handling and edge cases

4. **Security Assessment**:
   - Check for common security vulnerabilities (injection, authentication bypass, etc.)
   - Verify proper input validation and sanitization
   - Review authentication and authorization implementation
   - Check for exposed sensitive information

5. **Architecture Review**:
   - Verify code follows project architecture patterns
   - Check separation of concerns and modularity
   - Review database schema changes and migrations
   - Ensure proper error handling and logging

6. **Spec Alignment**:
   - Compare implementation against OpenSpec requirements
   - Verify all scenarios in specs are covered
   - Check API contract compliance
   - Validate response formats and status codes

7. **Final Report**:
   - Provide comprehensive summary of findings
   - List any issues found and recommendations
   - Confirm readiness for deployment or identify remaining work

**Reference**
- Use `openspec show <id>` to review change details during validation
- Use `cargo check --all-targets` for comprehensive compilation checking
- Use `cargo audit` or `cargo deny` for additional security scanning if available
<!-- OPENSPEC:END -->