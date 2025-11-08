## Why
Golden tests prevent schema drift in how provider events map into normalized `Signal.kind` values. As we add connectors, a stable taxonomy and verified mappings ensure downstream consumers (Story Hunter) can reliably filter and score Signals without breaking on provider‑specific semantics.

## What Changes
- Add a normalization capability spec defining the `Signal.kind` taxonomy and provider event→kind mapping rules.
- Establish golden test fixtures and a test harness that runs provider payload samples and asserts the expected `Signal.kind`.
- Select test crates and conventions for fixtures, discovery, and assertions (standardize on `walkdir` for discovery; defer `insta`/`assert_json_diff` until broader payload assertions).
- Document naming rules (action‑first verbs) and include examples per provider.
- Introduce a canonical `Signal.kind` registry in the spec with governance rules: implementations MUST NOT emit kinds outside the registry; new kinds require spec updates and fixtures.
- Enforce fixture coverage: each integrated provider MUST supply at least one fixture per emitted `Signal.kind`, or an explicit documented skip.

## Impact
- Affected specs: normalization (new), connectors (implicitly, mapping responsibilities), database/signals (no schema change).
- Affected code: tests (`tests/fixtures/normalization/**`, test harness module), connector mapping utilities.
- Tooling: add dev‑dependency for fixture discovery (`walkdir`). Snapshot/diff tooling deferred until needed.
- Non‑breaking: no API surface changes; test‑only with guidance for connectors.
