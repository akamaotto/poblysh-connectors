## Context
This change is documentation-focused: polish PRD/Tech Spec/API docs and add a local crypto rotation runbook. It aligns with the prior `add-local-token-encryption` change which adopts AES-GCM under a single application key. The behavior and crate versions referenced in this design are anchored to the current `connectors/Cargo.toml` and MUST remain consistent with it.

## Goals / Non-Goals
- Goals: Clean, consistent docs; safe local key rotation runbook; capture concrete crate versions from `connectors/Cargo.toml`; outline a repeatable lightweight research method to maintain best practices.
- Non-Goals: Implementing key rotation logic in code (e.g., multi-key decrypt) in this change; production runbook; changing OpenAPI/utoipa schemas or runtime configuration semantics.

## Lightweight Deep Research Algorithm
Purpose-built for crypto/key-management documentation and ops patterns, and repeatable whenever crypto-related dependencies or practices change.

1) Parallel seed searches (web + codebase)

   - Codebase (authoritative):
     - Use ripgrep (or equivalent) to locate:
       - `POBLYSH_CRYPTO_KEY`, `crypto_key`, `aes-gcm`, `aes_gcm`, `encrypt`, `decrypt`, `nonce`, `aad`, `aead`
       - Existing usage in configuration loading, startup validation, and encryption/decryption helpers
       - Relevant OpenSpec changes touching `crypto`, `config`, or `api-core`
     - Confirm that all documentation references match the actual behavior and requirements in the code.

   - Web (official and high-signal sources only):
     - crates.io and docs.rs for `aes-gcm`, `zeroize`, `base64`, and optional `hkdf` to confirm:
       - Latest stable versions compatible with the versions pinned in `connectors/Cargo.toml`
       - Recommended usage patterns (key sizes, nonce sizes, AAD, error handling)
     - RustCrypto AEAD documentation and security notes for AES-GCM misuse pitfalls (especially nonce reuse).
     - Select community sources (e.g., well-maintained GitHub repos, Rust Forum posts) only to validate or refine practices discovered in official docs, never as sole authorities.

2) Synthesize baseline
   - Map existing design decisions (nonce size, AAD composition, payload layout) to crate APIs and ensure they are compatible with the versions in `connectors/Cargo.toml`.
   - Identify operational gaps (startup validation, error mapping, rotation procedures) and ensure documentation clearly describes actual behavior for local/dev usage.

3) Sequential reinforcement searches (targeted follow-ups)
   - If a future rotation runbook needs preserving ciphertext:
     - Search for "multi-key decrypt aead rust", "key id headers aes-gcm" and validate against RustCrypto guidance.
   - If introducing HKDF-based epoch keys in a future change:
     - Search for "hkdf rust best practices context info" and relevant NIST guidance on key derivation.
   - Validate environment variable formatting and decoding:
     - Confirm that `POBLYSH_CRYPTO_KEY` uses standard base64 (not URL-safe) and decodes to exactly 32 bytes.
   - Re-run these targeted queries whenever crypto-related crate versions in `connectors/Cargo.toml` change to ensure docs and runbooks remain current.

4) Cross-check and finalize
   - Verify that crate versions and usage in this change match `connectors/Cargo.toml` exactly.
   - Confirm practices with RustCrypto and crate documentation; check for deprecation notes or breaking changes in the referenced versions.
   - Ensure codebase constraints (edition 2024, tokio/axum versions) are compatible with the crypto and observability stack.
   - Document that this validation step SHALL be repeated whenever crypto-related dependency versions change.

Operational commands (examples):
```bash
rg -n "POBLYSH_CRYPTO_KEY|cipher|encrypt|decrypt|aad" -S
curl -s https://crates.io/api/v1/crates/aes-gcm | jq -r '.crate.max_stable_version'
```

## Tech Selection
- AEAD: `aes-gcm = "0.10.3"` — stable, well-maintained, matches the version in `connectors/Cargo.toml`.
  - Reference: https://crates.io/crates/aes-gcm
- Key hygiene: `zeroize = "1.8.1"` — clears key material on drop and matches the version in `connectors/Cargo.toml`.
  - Reference: https://crates.io/crates/zeroize
- Base64: `base64 = "0.22.1"` — standard base64 decode for a 32-byte key, matches the version in `connectors/Cargo.toml`.
  - Reference: https://crates.io/crates/base64
- Optional KDF (future change only, not required by this proposal): `hkdf = "0.12.4"` — candidate for deriving epoch keys from a root secret in a separate follow-up change.
  - Reference: https://crates.io/crates/hkdf

These selections are descriptive for reviewers and MUST NOT diverge from `connectors/Cargo.toml`. Any future version changes SHALL re-run the lightweight deep research algorithm and update this documentation accordingly.

## Risks / Trade-offs
- Doc/ops drift from implementation:
  - Mitigated by explicitly anchoring crate versions and behavior to `connectors/Cargo.toml` and by requiring re-running the research algorithm on dependency changes.
- Local runbook expectations vs production needs:
  - Mitigated by clearly limiting the runbook scope to local/dev usage, calling out that production rotation requires a separate, stricter spec and implementation.
- Over-interpretation of future ideas (e.g., HKDF, multi-key decrypt) as current behavior:
  - Mitigated by marking them as future follow-ups only, with no implied current runtime change.

## Migration Plan (Docs)
- Add runbook under `docs/runbooks/local-crypto-rotation.md` with:
  - A clear, safe default path for local/dev: rotate the key and recreate local connections/tokens.
  - An explicit note that preserving existing ciphertext via multi-key decrypt or migration tooling is out of scope for this change and would require a separate approved change.
- Cross-link from `docs/configuration.md` and `README.md` so developers can reliably discover the runbook.
- Create a follow-up task for any `config`/`crypto` spec deltas once `add-local-token-encryption` merges, ensuring specs, code, and runbooks stay aligned.

## Open Questions
- Do we want a temporary maintenance command for decrypt→reencrypt to preserve local data? If yes, that requires a small CLI in a future change.
