---
**name:** OpenSpec: QA Check
**description:** Comprehensive quality assurance with multi-layered contextual analysis inspired by CodeRabbit's review methodology
**category:** OpenSpec
**tags:** [openspec, qa, validation, testing, architecture, security, contextual-review]

---

## Core Philosophy
Perform reviews like a senior engineer who understands the entire codebase—not just the changed code, but how it connects to architecture, follows patterns, and affects downstream dependencies. Focus on delivering high signal-to-noise ratio feedback that is actionable, context-aware, and educational.

---

## Guardrails
- **Simplicity First**: Favor straightforward, minimal implementations. Add complexity only when explicitly requested or clearly required.
- **Tight Scoping**: Keep changes focused on the requested outcome. Avoid scope creep.
- **Context Awareness**: Always consider the broader codebase implications, not just the immediate changes.
- **Learning Orientation**: Make reviews educational—explain *why* something is an issue, not just *what* is wrong.
- **Reference Documentation**: Refer to `openspec/AGENTS.md` for OpenSpec conventions and clarifications.
- **Axum Skill Pack**: If the change touches Axum 0.8 routing, middleware, or extractors, consult `.claude/skills/axum-0-8/SKILL.md` for framework patterns and pitfalls.

---

## Multi-Phase Review Process

### Phase 1: Change Validation & Context Gathering

**Objective:** Establish complete understanding of intent, requirements, and codebase context.

#### 1.1 Requirements Analysis
- [ ] Read `changes/<id>/proposal.md` to understand the problem being solved
- [ ] Review `design.md` (if present) for architectural decisions and rationale
- [ ] Parse `tasks.md` to extract all requirements and acceptance criteria
- [ ] Verify all tasks are marked as completed
- [ ] Confirm the change exists and is not archived

#### 1.2 Context Enrichment
- [ ] **Code Graph Analysis**: Map dependencies and symbol relationships
  - Identify all files that import or depend on modified code
  - Trace downstream effects of interface/API changes
  - Detect files that frequently change together with modified files
- [ ] **Historical Context**: Review related past changes
  - Check git history for patterns in similar changes
  - Identify recurring issues in this area of the codebase
- [ ] **Issue Tracker Context**: Link to relevant tickets (Jira/Linear/GitHub Issues)
  - Validate that implementation addresses the stated problem
  - Check for edge cases mentioned in issue discussions

#### 1.3 Architecture Pattern Recognition
- [ ] Identify existing patterns in the codebase
- [ ] Verify new code follows established conventions
- [ ] Flag deviations from project architecture
- [ ] Document any intentional pattern evolution

---

### Phase 2: Compilation & Lint Analysis

**Objective:** Ensure code meets basic quality standards before deep analysis.

#### 2.1 Build Verification
```bash
# Check compilation
cargo check --all-targets

# Verify release build
cargo build --release

# Check for warnings
cargo clippy -- -D warnings

# Verify formatting
cargo fmt --check

# Check tests
cargo test --all --all-features --no-fail-fast
```

#### 2.2 Static Analysis Integration
- [ ] Run configured linters and capture all findings
- [ ] Execute security scanners (cargo audit, cargo deny if available)
- [ ] Synthesize results into actionable feedback categories:
  - **Critical**: Security vulnerabilities, logic errors
  - **Important**: Potential bugs, performance issues
  - **Minor**: Style inconsistencies, documentation gaps

---

### Phase 3: Deep Code Analysis

**Objective:** Perform semantic analysis using Abstract Syntax Tree (AST) understanding.

#### 3.1 Implementation Verification
- [ ] **Line-by-Line Review**: Validate each change against requirements
  - Compare implementation against `tasks.md` specifications
  - Verify all specified endpoints/functions/features are present
  - Check that modifications serve the intended purpose

- [ ] **Cross-File Impact Analysis**:
  - Trace how changes affect callers and dependencies
  - Identify breaking changes in public APIs
  - Verify backwards compatibility where required

#### 3.2 Logic & Correctness
- [ ] **Algorithm Review**:
  - Validate correctness of business logic
  - Check for off-by-one errors, boundary conditions
  - Verify loop termination and recursion base cases

- [ ] **Error Handling**:
  - Ensure all error paths are handled
  - Verify appropriate error types are returned
  - Check that errors provide actionable context
  - Validate error propagation patterns

- [ ] **Concurrency & Thread Safety**:
  - Identify potential race conditions
  - Verify proper use of locks/mutexes
  - Check for deadlock possibilities

#### 3.3 Data Flow Analysis
- [ ] Trace data inputs through transformation to outputs
- [ ] Verify data validation at boundaries
- [ ] Check for data sanitization before use
- [ ] Validate state management and mutations

---

### Phase 4: Spec Alignment & Contract Validation

**Objective:** Ensure implementation adheres to OpenSpec and API contracts.

#### 4.1 OpenSpec Compliance
- [ ] Compare implementation against OpenSpec requirements
- [ ] Verify all spec scenarios are covered
- [ ] Validate that documented behavior matches implementation

#### 4.2 API Contract Review
- [ ] **Response Validation**:
  - Check response shapes match OpenAPI specs
  - Verify all required fields are present
  - Validate data types and formats

- [ ] **Status Code Mapping**:
  - Ensure HTTP status codes align with REST conventions
  - Verify error codes match specification
  - Check 2xx for success, 4xx for client errors, 5xx for server errors

- [ ] **Header Handling**:
  - Validate required headers (Content-Type, Authorization, etc.)
  - Check trace propagation headers (X-Request-ID, trace-id, etc.)
  - Verify CORS headers if applicable

#### 4.3 Documentation Updates
- [ ] Confirm OpenAPI/Swagger docs are updated
- [ ] Verify inline code comments explain complex logic
- [ ] Check that public APIs have doc comments
- [ ] Validate example usage in documentation

---

### Phase 5: Security Assessment

**Objective:** Identify security vulnerabilities before production.

#### 5.1 Common Vulnerability Checks
- [ ] **Injection Attacks**:
  - SQL injection vectors
  - Command injection possibilities
  - Path traversal vulnerabilities

- [ ] **Authentication & Authorization**:
  - Verify authentication on protected endpoints
  - Check authorization logic (RBAC, permissions)
  - Validate token handling and expiration
  - Ensure no privilege escalation paths

- [ ] **Data Exposure**:
  - Check for exposed sensitive information (passwords, keys, PII)
  - Verify secrets are not hardcoded
  - Validate logging doesn't leak sensitive data

#### 5.2 Input Validation
- [ ] Verify all inputs are validated
- [ ] Check for proper sanitization
- [ ] Validate size/length limits on inputs
- [ ] Ensure type coercion is handled safely

#### 5.3 Dependency Security
- [ ] Review newly added dependencies
- [ ] Check for known vulnerabilities in dependencies
- [ ] Verify dependencies are from trusted sources

---

### Phase 6: Test Coverage & Quality

**Objective:** Validate comprehensive testing strategy.

#### 6.1 Test Execution
```bash
# Run all tests with quiet output
cargo test -q

# Run with verbose output for failures
cargo test -- --nocapture

# Check test coverage if tooling available
cargo tarpaulin --out Xml
```

#### 6.2 Test Quality Review
- [ ] **Unit Test Coverage**:
  - Verify core logic has unit tests
  - Check edge cases are tested
  - Validate error paths are covered

- [ ] **Integration Test Coverage**:
  - Confirm API endpoints have integration tests
  - Verify cross-component interactions are tested
  - Check database operations are covered

- [ ] **Test Assertions**:
  - Review test assertions for correctness
  - Verify tests actually validate behavior
  - Check that tests would fail if code breaks

#### 6.3 Missing Coverage Identification
- [ ] Identify untested code paths
- [ ] Flag critical functionality without tests
- [ ] Suggest additional test scenarios

---

### Phase 7: Architecture & Design Review

**Objective:** Evaluate structural quality and maintainability.

#### 7.1 Design Patterns & Best Practices
- [ ] **Separation of Concerns**:
  - Verify proper layering (presentation, business logic, data)
  - Check that modules have clear responsibilities
  - Validate loose coupling between components

- [ ] **Code Organization**:
  - Review module structure and boundaries
  - Check for proper abstraction levels
  - Validate naming conventions and clarity

#### 7.2 Performance Considerations
- [ ] Identify potential performance bottlenecks
- [ ] Check for N+1 query problems
- [ ] Verify efficient algorithms and data structures
- [ ] Flag expensive operations in hot paths

#### 7.3 Maintainability & Readability
- [ ] Assess code clarity and readability
- [ ] Check for overly complex functions (high cyclomatic complexity)
- [ ] Verify consistent coding style
- [ ] Validate appropriate use of comments

#### 7.4 Database & Schema Changes
- [ ] Review migration scripts for correctness
- [ ] Check for data loss risks
- [ ] Verify proper indexing strategy
- [ ] Validate backwards compatibility of schema changes

---

### Phase 8: Contextual Learning & Adaptation

**Objective:** Capture team-specific patterns for future reviews.

#### 8.1 Pattern Recognition
- [ ] Document coding conventions observed
- [ ] Note architectural decisions and rationale
- [ ] Record team preferences (style, patterns, etc.)

#### 8.2 Feedback Loop
- [ ] Note any spec drift for documentation updates
- [ ] Identify gaps in CI/CD pipeline
- [ ] Suggest process improvements

---

## Final Report Generation

### Report Structure

#### Executive Summary
- **Change Overview**: One-line description of what changed
- **Readiness Assessment**: Ready to merge / Needs work / Blocked
- **Risk Level**: Low / Medium / High
- **Key Metrics**:
  - Lines changed: [number]
  - Files affected: [number]
  - Dependencies impacted: [number]
  - Test coverage: [percentage]

#### Detailed Findings

For each issue identified, provide:

1. **Severity**: Critical / Important / Minor
2. **Category**: Security / Logic / Performance / Style / Documentation
3. **Location**: `file.rs:line_number` with context
4. **Issue Description**: Clear explanation of the problem
5. **Impact**: How this affects the system
6. **Recommendation**: Specific, actionable fix
7. **Explanation**: Educational context on *why* this matters

**Example:**
```
🔴 CRITICAL - Security Vulnerability
Location: src/auth/handler.rs:42
Issue: SQL query constructed with string concatenation using user input
Impact: Enables SQL injection attacks allowing unauthorized data access
Recommendation: Use parameterized queries with prepared statements
Why it matters: String concatenation with untrusted input is the #1 cause
of SQL injection. Parameterized queries ensure the database treats input
as data, not executable code, preventing injection attacks.
```

#### Architecture & Design Observations
- Document significant architectural patterns
- Note areas of technical debt
- Highlight elegant solutions worth replicating

#### Test Coverage Analysis
- Overall coverage percentage
- Critical paths without tests
- Recommended additional test scenarios

#### Spec Alignment Summary
- Compliance with OpenSpec requirements
- Any deviations and their justification
- Suggested spec clarifications

#### Action Items
Prioritized list of required changes:
1. **Must Fix** (blocks merge): [list]
2. **Should Fix** (before release): [list]
3. **Nice to Have** (future improvement): [list]

#### Deployment Readiness
- [ ] All critical issues resolved
- [ ] Tests passing
- [ ] Documentation updated
- [ ] Security validated
- [ ] Performance acceptable
- [ ] Backwards compatibility maintained

**Final Recommendation:** [APPROVE / REQUEST CHANGES / REJECT]

---

## Reference Commands

```bash
# Review change details
openspec show <id>

# Comprehensive compilation check
cargo check --all-targets

# Security scanning
cargo audit
cargo deny check

# Test execution with coverage
cargo test -q
cargo tarpaulin

# Lint and format checks
cargo clippy -- -D warnings
cargo fmt --check

# Release build verification
cargo build --release
```

---

## Quality Principles

1. **Context Over Syntax**: Understand the broader impact, not just local changes
2. **Education Over Criticism**: Explain why, not just what
3. **Signal Over Noise**: Focus on meaningful issues, not nitpicks
4. **Actionable Feedback**: Provide specific solutions, not vague concerns
5. **Consistency**: Apply standards uniformly across the codebase
6. **Continuous Learning**: Adapt to team patterns and preferences

---

## Additional Recommendations

- **Code Refactoring**: Simplify complex logic and improve readability
- **Performance Optimization**: Identify bottlenecks and optimize critical sections
- **Error Handling**: Enhance error messages and logging for better debugging

---

## Continuous Improvement

- **Code Reviews**: Regularly review and discuss code changes
- **Feedback Loops**: Establish channels for continuous feedback and improvement
- **Documentation Updates**: Keep documentation up-to-date with code changes

---

## Continuous Learning

- **Team Training**: Regular training sessions on new technologies and best practices
- **Knowledge Sharing**: Encourage knowledge sharing and collaboration
- **Feedback Mechanisms**: Implement mechanisms for continuous feedback and learning

---
