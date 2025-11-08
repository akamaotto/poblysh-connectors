## Context
We need a durable, provider‑agnostic taxonomy for `Signal.kind` and an automated way to prevent drift as connectors evolve. Golden tests with curated payload fixtures will gate changes and provide quick feedback for mapping regressions.

## Goals / Non-Goals
- Goals: codify taxonomy; enforce mapping via golden fixtures; keep fixtures minimal; integrate into CI.
- Non-Goals: define complete payload normalization schema for all providers (covered incrementally per connector changes).

## Decisions
- Discovery crate: use `walkdir` (v2) for recursive fixture discovery. Rationale: simple recursion, cross‑platform behavior, avoids dual‑dependency with `glob`.
- Parsing: `serde_json` (existing) for loading fixtures.
- Diffing/Snapshots: do not require `insta` for MVP; do not require `assert_json_diff` for MVP. These remain optional for future expansion when asserting larger JSON shapes.
- Fixture format: JSON with `{ provider, name, input, expected: { kind } }` to keep MVP simple and focused on the mapping.
- Layout: `tests/fixtures/normalization/<provider>/*.json` + a single test harness that scans everything and runs parametrized assertions.
- Naming: adopt action‑first, provider‑agnostic verbs. Align with project conventions (openspec/project.md: Signals use action‑first verbs).
- Shared helpers: introduce `src/normalization/` with canonical `SignalKind` constants and provider-specific helpers (starting with `example`, `jira`, `zoho-cliq`) so connectors and tests reuse the same mapping code.

## Policy: Canonical Kind Registry & Governance
- A canonical registry of allowed `Signal.kind` values is maintained in the normalization spec (see "Canonical Signal.kind Registry").
- Implementations MUST NOT emit kinds outside the registry. Proposals that add kinds MUST update the registry within their spec deltas and include at least one golden fixture.
- The test harness MUST fail when a produced `kind` is not present in the registry.

## Policy: Fixture Coverage
- Each provider integrated into the connectors MUST include at least one fixture per `Signal.kind` it can emit.
- If a provider is present but not yet normalized, the harness MAY allow an explicit, documented skip: a `SKIP.md` with rationale under `tests/fixtures/normalization/<provider>/`. Absence of fixtures without such documentation is a failure.

## Risks / Trade-offs
- Over‑specific fixtures may become brittle → mitigate by aiming for minimal fields needed to drive mapping.
- Taxonomy churn → centralize mapping rules and document rationale to reduce debate; add fixtures before changing behavior.
- Early connectors may lack mapping utilities → harness should allow skipping providers without mapping yet (or seed example fixtures only until implemented).

## Migration Plan
1) Add dev‑dependency: `walkdir` (required). Defer `insta` and `assert_json_diff` until we expand assertions beyond `kind`.
2) Create fixture folders and seed with `example` provider cases.
3) Implement harness and wire into CI (`cargo test`). Harness must validate `kind` against the registry.
4) As each provider lands (GitHub/Jira/etc.), add fixtures + mapping utilities and ensure coverage policy holds.

## Open Questions
- Do we snapshot any normalized payload subset now, or stick to `kind` only for MVP? (Recommend: `kind` only now; expand later.)
- Should we maintain a central registry/table for kind taxonomy to validate against? (Future improvement.)

## Tech Choices (Versions & Docs)
- walkdir = ^2 (docs: https://docs.rs/walkdir/) — recursive directory walking for fixtures (required for MVP).
- assert_json_diff = ^2 (docs: https://docs.rs/assert_json_diff/) — optional for readable diffs when comparing broader JSON.
- insta = ^1 (docs: https://insta.rs/docs/) — optional snapshot testing for normalized subsets later.
- serde/serde_json — already in Cargo.toml; used to parse fixtures and payloads.

Note: Pin with caret constraints to track compatible minor/patch updates within the major version for dev‑only dependencies.

## Research Plan (Lightweight Deep Research Algorithm)
Parallel discovery (kick off together):
- P1: Best practices for Rust golden tests (insta vs custom asserts) from official docs and community posts.
- P2: Patterns for JSON fixture management in Rust test suites (`walkdir`/`glob`, test parametrization).
- P3: Provider mapping conventions in OSS connectors (GitHub, Slack, Jira) — look for event→kind normalization examples.
- P4: Naming taxonomy guidance for event semantics (action‑first conventions across ecosystems).

Sequential reinforcement loops (for each Pn):
1) Read official docs/guides; note recommended patterns and pitfalls.
2) Sample 2–3 community references (issues, blogs) to cross‑check practicality.
3) Inspect a few mature repos implementing similar normalization to see real‑world edge cases.
4) Draft concise rules/conventions; test against a handful of real payloads.
5) Iterate rules if contradictions arise; codify in spec + add fixtures.

Result: consolidated notes + selected crate versions, with links and example snippets ready for review.
