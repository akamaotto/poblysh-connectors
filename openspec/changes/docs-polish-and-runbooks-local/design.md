## Context
This change is documentation‑focused: polish PRD/Tech Spec/API docs and add a local crypto rotation runbook. It aligns with the prior `add-local-token-encryption` change which adopts AES‑GCM under a single application key.

## Goals / Non-Goals
- Goals: Clean, consistent docs; safe local key rotation runbook; capture concrete crate versions; outline research method to maintain best practices.
- Non‑Goals: Implementing key rotation logic in code (e.g., multi‑key decrypt) in this change; production runbook.

## Lightweight Deep Research Algorithm
Purpose-built for crypto/key‑management documentation and ops patterns.

1) Parallel seed searches (web + codebase)
   - Web:
     - crates.io for `aes-gcm`, `zeroize`, `base64`, `hkdf` latest stable versions
     - RustCrypto AEAD docs and security notes
     - Community sources: GitHub issues/PRs, Rust Forum posts on AES‑GCM pitfalls and nonce handling
   - Codebase:
     - ripgrep for `crypto`, `encrypt`, `decrypt`, `POBLYSH_CRYPTO_KEY`, `ciphertext`, `aad`
     - scan OpenSpec changes touching `crypto`, `config`, `api-core`

2) Synthesize baseline
   - Map existing design decisions (nonce size, AAD composition, payload layout) to crate APIs
   - Identify operational gaps (startup validation, error mapping, rotation procedures)

3) Sequential reinforcement searches (targeted follow‑ups)
   - If rotation runbook needs preserving ciphertext: search "multi-key decrypt aead rust", "key id headers aes-gcm"
   - If using HKDF for epoch keys: search "hkdf rust best practices context info" and NIST guidance on key derivation
   - Validate env var formatting: "rust base64 standard vs urlsafe decode"

4) Cross‑check and finalize
   - Verify versions against crates.io
   - Confirm practices with RustCrypto docs, check for deprecation notes
   - Ensure codebase constraints (edition 2024, tokio/axum versions) are compatible

Operational commands (examples):
```bash
rg -n "POBLYSH_CRYPTO_KEY|cipher|encrypt|decrypt|aad" -S
curl -s https://crates.io/api/v1/crates/aes-gcm | jq -r '.crate.max_stable_version'
```

## Tech Selection
- AEAD: `aes-gcm = "0.10.3"` — stable, well‑maintained, matches design. (https://crates.io/crates/aes-gcm)
- Key hygiene: `zeroize = "1.8.1"` — clears key material on drop and matches current Cargo.toml. (https://crates.io/crates/zeroize)
- Base64: `base64 = "0.22.1"` — standard base64 decode for 32‑byte key. (https://crates.io/crates/base64)
- Optional KDF: `hkdf = "0.12.4"` — for future epoch keys (rotation follow‑up change). (https://crates.io/crates/hkdf)

## Risks / Trade-offs
- Doc/ops drift from implementation → mitigate with pointers to the owning spec/change and add a validation task to ensure OpenAPI + config docs reference the same env var.
- Local runbook expectations vs production needs → limit scope clearly as local/dev and document production caveats.

## Migration Plan (Docs)
- Add runbook under `docs/runbooks/local-crypto-rotation.md`.
- Cross‑link from `docs/configuration.md` and `README.md`.
- Create follow‑up task for spec deltas once `add-local-token-encryption` merges.

## Open Questions
- Do we want a temporary maintenance command for decrypt→reencrypt to preserve local data? If yes, that requires a small CLI in a future change.
