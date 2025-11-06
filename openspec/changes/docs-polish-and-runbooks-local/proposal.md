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
- Affected specs: none directly (doc/runbook only). Follow‑on deltas may update `config`/`crypto` once encryption merges.
- OpenAPI/utoipa annotations remain unchanged in this change; any API schema edits will be proposed separately if needed.
- No API surface change in this change; only documentation and runbooks.

## Tech Inventory (from Cargo.toml)
- Core: `axum 0.8.6`, `tokio 1.48.0`, `utoipa 5.3.1`, `sea-orm 1.1.17`, `serde 1.0.217`, `thiserror 2.0.11`
- Observability: `tracing 0.1.41`, `tracing-subscriber 0.3.19`
- Utilities: `uuid 1.11.0`, `chrono 0.4.38`, `rand 0.8.5`, `base64-url 3.0.0`, `subtle 2.6.1`

## Selected Crypto Crates (for review readiness)
- `aes-gcm = "0.10.3"` (AEAD, AES‑256‑GCM)
- `zeroize = "1.8.1"` (key material hygiene)
- `base64 = "0.22.1"` (decode `POBLYSH_CRYPTO_KEY`)
- Optional: `hkdf = "0.12.4"` (future: derive epoch keys for rotations)

## Acceptance
- A documented runbook exists at `docs/runbooks/local-crypto-rotation.md`.
- Proposal/design lists crate versions and rationale for review.
- Tasks enumerated to propagate doc polish into the codebase where needed.
