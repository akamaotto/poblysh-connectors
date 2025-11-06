## Why
Golden tests prevent schema drift in how provider events map into normalized `Signal.kind` values. As we add connectors, a stable taxonomy and verified mappings ensure downstream consumers (Story Hunter) can reliably filter and score Signals without breaking on provider‑specific semantics.

## What Changes
- Add a normalization capability spec defining the `Signal.kind` taxonomy and provider event→kind mapping rules.
- Establish golden test fixtures and a test harness that runs provider payload samples and asserts the expected `Signal.kind`.
- Select test crates and conventions for fixtures, discovery, and assertions.
- Document naming rules (action‑first verbs) and include examples per provider.

## Impact
- Affected specs: normalization (new), connectors (implicitly, mapping responsibilities), database/signals (no schema change).
- Affected code: tests (`tests/fixtures/normalization/**`, test harness module), connector mapping utilities.
- Tooling: add dev‑dependencies for fixture discovery and assertions.
- Non‑breaking: no API surface changes; test‑only with guidance for connectors.

