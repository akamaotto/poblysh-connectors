## 1. Implementation
- [ ] 1.1 Add dev dependencies: `insta` (snapshots), `assert_json_diff`, `walkdir` or `glob` (fixture discovery)
- [ ] 1.2 Create fixture layout `tests/fixtures/normalization/<provider>/*.json`
- [ ] 1.3 Implement test harness `tests/normalization/golden_mapping_tests.rs` to:
  - [ ] Discover fixtures by provider
  - [ ] Parse `{ provider, name, input, expected: { kind } }`
  - [ ] Invoke provider mapping function to produce `Signal.kind`
  - [ ] Assert equality for `expected.kind` (and optional shape checks)
- [ ] 1.4 Seed initial fixtures for `example` provider and one per prioritized provider as connectors land
- [ ] 1.5 Document fixture authoring rules in a short README under `tests/fixtures/normalization/`

## 2. Design/Docs
- [ ] 2.1 Record taxonomy and mapping guidance in spec deltas (this change)
- [ ] 2.2 Add mapping table examples to connector docs as they are implemented

## 3. CI/Validation
- [ ] 3.1 Run `cargo test` in CI; fail on mapping regressions
- [ ] 3.2 Validate spec with `openspec validate add-normalization-fixtures --strict`

