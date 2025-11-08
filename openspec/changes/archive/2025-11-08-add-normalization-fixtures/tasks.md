## 1. Implementation
- [x] 1.1 Add dev dependency: `walkdir` (fixture discovery). Defer `insta` and `assert_json_diff` until broader JSON assertions are required.
- [x] 1.2 Create fixture layout `tests/fixtures/normalization/<provider>/*.json`
- [x] 1.3 Implement test harness `tests/normalization_integration_tests.rs` to:
  - [x] Discover fixtures by provider using `walkdir`
  - [x] Parse `{ provider, name, input, expected: { kind } }`
  - [x] Invoke provider mapping function to produce `Signal.kind`
  - [x] Validate produced `kind` against the canonical registry (see spec)
  - [x] Assert equality for `expected.kind`
- [x] 1.4 Enforce coverage: at least one fixture per emitted `Signal.kind` for each integrated provider. If a provider lacks fixtures, require a `tests/fixtures/normalization/<provider>/SKIP.md` with rationale; otherwise fail.
- [x] 1.5 Seed initial fixtures for `example` provider and one per prioritized provider as connectors land
- [x] 1.6 Document fixture authoring rules and coverage policy in `tests/fixtures/normalization/README.md`

## 2. Design/Docs
- [x] 2.1 Record taxonomy and mapping guidance in spec deltas (this change)
- [x] 2.2 Add canonical registry to the normalization spec and reference governance rules for adding new kinds
- [ ] 2.3 Add mapping table examples to connector docs as they are implemented

## 3. CI/Validation
- [x] 3.1 Run `cargo test` in CI; fail on mapping regressions, out‑of‑registry kinds, and missing coverage without `SKIP.md`
- [x] 3.2 Validate spec with `openspec validate add-normalization-fixtures --strict`
