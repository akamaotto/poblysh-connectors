## Context
We need a durable, provider‑agnostic taxonomy for `Signal.kind` and an automated way to prevent drift as connectors evolve. Golden tests with curated payload fixtures will gate changes and provide quick feedback for mapping regressions.

## Goals / Non-Goals
- Goals: codify taxonomy; enforce mapping via golden fixtures; keep fixtures minimal; integrate into CI.
- Non-Goals: define complete payload normalization schema for all providers (covered incrementally per connector changes).

## Decisions
- Crates: use `walkdir` or `glob` for fixture discovery; `serde_json` (existing) for parsing; `assert_json_diff` for clean diffs when we later assert partial payload shapes; `insta` optional for future snapshotting of selected normalized subsets.
- Fixture format: JSON with `{ provider, name, input, expected: { kind } }` to keep MVP simple and focused on the mapping.
- Layout: `tests/fixtures/normalization/<provider>/*.json` + a single test harness that scans everything and runs parametrized assertions.
- Naming: adopt action‑first, provider‑agnostic verbs. Align with project conventions (openspec/project.md: Signals use action‑first verbs).

## Risks / Trade-offs
- Over‑specific fixtures may become brittle → mitigate by aiming for minimal fields needed to drive mapping.
- Taxonomy churn → centralize mapping rules and document rationale to reduce debate; add fixtures before changing behavior.
- Early connectors may lack mapping utilities → harness should allow skipping providers without mapping yet (or seed example fixtures only until implemented).

## Migration Plan
1) Add dev‑dependencies: `walkdir` or `glob`, `assert_json_diff`, `insta` (optional; start with direct equality on kind).
2) Create fixture folders and seed with `example` provider cases.
3) Implement harness and wire into CI (`cargo test`).
4) As each provider lands (GitHub/Jira/etc.), add fixtures + mapping utilities.

## Open Questions
- Do we snapshot any normalized payload subset now, or stick to `kind` only for MVP? (Recommend: `kind` only now; expand later.)
- Should we maintain a central registry/table for kind taxonomy to validate against? (Future improvement.)

## Tech Choices (Versions & Docs)
- walkdir = ^2 (docs: https://docs.rs/walkdir/) — stable recursive directory walking for fixtures.
- glob = ^0.3 (docs: https://docs.rs/glob/) — simple globbing alternative; either this or walkdir is sufficient.
- assert_json_diff = ^2 (docs: https://docs.rs/assert_json_diff/) — readable diffs when comparing JSON.
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

