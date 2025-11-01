---
description: qa regression pass for connectors changes
auto_execution_mode: 1
---

# QA Check Workflow

1. Review the relevant OpenSpec proposal, design, and spec deltas to understand intended behaviour.
2. Inspect impacted source files to confirm the implementation lines up with the documented requirements.
3. Validate response shapes and status mappings against the spec (focus on error codes, headers, and trace propagation).
   // turbo
4. Run `cargo test -q` to ensure unit and integration tests pass. Run cargo check to ensure there are no errors.
5. Capture findings by noting regressions, spec drift, or missing coverage, and report with file/line references.