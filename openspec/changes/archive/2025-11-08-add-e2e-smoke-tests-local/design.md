## Context
We want a minimal, robust E2E smoke layer that exercises the true binary startup (config, DB init, auto migrations) and asserts core HTTP surfaces without introducing runtime complexity.

## Goals / Non-Goals
- Goals: spawn real binary, deterministic bind, readiness‑gated HTTP checks, clean teardown
- Non-Goals: load testing, broad scenario coverage, connector/provider integration depth

## Decisions
- Spawn binary with `assert_cmd` to avoid manual path resolution and ease cross‑platform execution.
- Use `portpicker` to reduce port collision risk and allow deterministic `POBLYSH_API_BIND_ADDR`.
- Gate readiness with `/readyz` to ensure DB dependency is live before assertions.
- Keep DB flexible: prefer Postgres (Docker) but support SQLite for frictionless local runs.

### Alternatives considered
- Using `tokio::process::Command` directly: workable but less ergonomic than `assert_cmd` for locating the built binary under `cargo test`.
- Binding to port 0: the binary prints the configured address, not the actual OS‑assigned ephemeral port; complicates discovery. Using `portpicker` to pre-select the port eliminates this complexity.
- Binding to 0.0.0.0: would expose the service to the network during local testing; rejected in favor of explicit 127.0.0.1 binding for security.

## Risks / Trade-offs
- Race conditions in port selection: acceptable for local runs; mitigate with retry once on bind failure.
- Smoke test flakiness due to slow DB start or migrations: keep generous (but bounded) readiness timeout and clear logs on failure.

## Migration Plan
- Add dev‑only deps and a single test file; no production code changes.
- Add make/just tasks to lower activation energy for contributors.

## Open Questions
- Should the protected endpoint check be required or optional if operator token not present? Proposed: require token and fail fast with guidance, to surface config gaps early.

