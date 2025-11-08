## Why
We need to polish product/tech/API documentation and provide a concrete local crypto key rotation procedure. Recent work introduced local token encryption (AES‑GCM) behind a single application key but left operational rotation as a follow‑up. Clear PRD notes, tech spec clarifications, and a local runbook reduce operational risk and streamline review/implementation.

## What Changes
- PRD/Tech Spec/API docs polish and alignment across files.
- Add a local crypto key rotation runbook with safe, step‑by‑step guidance.
- Capture crate/tech selections and doc versions to unblock review.
- Identify minimal spec deltas required (if any) and create tasks to implement them.

## Impact
- Affected docs: `README.md`, `docs/configuration.md`, new `docs/runbooks/local-crypto-rotation.md`.
- Related change: `add-local-token-encryption` (clarify env var, error handling, and ops procedures).
- Affected specs: This change is strictly documentation/runbook-only and does not modify existing `config` or `crypto` specifications.
- OpenAPI/utoipa annotations and API schemas remain unchanged in this change; any OpenAPI/schema/config-spec edits will be proposed as a separate OpenSpec change once `add-local-token-encryption` is finalized. No API surface changes are included in this documentation-focused change.
- No API surface change in this change; only documentation and runbooks.

## Tech Inventory (from Cargo.toml)
The `connectors/Cargo.toml` file is the canonical source of truth for all crate and version selections referenced by this change. The values below are derived from that file at the time of this proposal and MUST be kept in sync by re-running the research/validation steps when versions change.

- Core: `axum 0.8.6`, `tokio 1.48.0`, `utoipa 5.4.0`, `sea-orm 1.1.17`, `serde 1.0.217`, `thiserror 2.0.11`
- Observability: `tracing 0.1.41`, `tracing-subscriber 0.3.19`
- Utilities: `uuid 1.11.0`, `chrono 0.4.38`, `rand 0.8.5`, `base64-url 3.0.0`, `subtle 2.6.1`

## Selected Crypto Crates (for review readiness)
These selections are aligned with the current `connectors/Cargo.toml` and validated against official documentation. They are documentation-facing only in this change; no new crypto implementation is introduced here.

- `aes-gcm = "0.10.3"` (AEAD, AES–256–GCM)
  - Ref: https://crates.io/crates/aes-gcm
- `zeroize = "1.8.1"` (key material hygiene)
  - Ref: https://crates.io/crates/zeroize
- `base64 = "0.22.1"` (decode `POBLYSH_CRYPTO_KEY` as standard base64 to 32 bytes)
  - Ref: https://crates.io/crates/base64
- Optional (future follow-up only, not part of this change): `hkdf = "0.12.4"` for deriving epoch keys to support non-destructive rotations
  - Ref: https://crates.io/crates/hkdf

## Acceptance
- A documented runbook exists at `docs/runbooks/local-crypto-rotation.md`, clearly scoped to local/dev usage and explicitly stating that production rotation guidance will be handled separately.
- Proposal/design lists crate versions and rationale based on the authoritative `connectors/Cargo.toml`, with references to official crate documentation.
- The lightweight deep research algorithm:
  - Begins with parallel codebase and external documentation searches,
  - Performs targeted, sequential reinforcement searches for crypto/key-management specifics,
  - Is re-runnable and MUST be executed whenever relevant crate versions or crypto design decisions change, to keep documentation and runbooks aligned with best practices.
- Tasks are enumerated (in `tasks.md` and related specs) to propagate documentation polish into the repo, ensure OpenAPI/config/docs consistency, and comply with OpenSpec conventions (clarity, correctness, consistency).
- The change meets strict review expectations (including automated and human QA) by:
  - Maintaining a documentation-only scope with no hidden API or behavior changes,
  - Ensuring terminology, env vars (including `POBLYSH_CRYPTO_KEY`), and referenced paths are consistent,
  - Providing enough precision and references for reviewers to validate quickly.
