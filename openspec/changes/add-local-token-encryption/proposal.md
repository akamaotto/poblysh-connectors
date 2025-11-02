## Why
Access and refresh tokens for provider connections must be protected at rest. Today, the `connections` table holds opaque `*_ciphertext` fields with no defined encryption behavior. Introducing authenticated encryption using a single application key improves security immediately without changing external APIs or database schema.

## What Changes
- Add local token encryption module using AES‑256‑GCM.
- Add required `POBLYSH_CRYPTO_KEY` (base64‑encoded 32 bytes) to derive the AEAD key.
- Define binary payload format: `0x01 | 12‑byte nonce | ciphertext | 16‑byte tag`.
- Use AAD binding (`tenant_id | provider_slug | external_id`) to context‑lock tokens to their connection.
- Update repository/token paths to encrypt before write and decrypt on read via helpers.
- Validation and errors for missing/invalid crypto key on startup.

## Impact
- Affected specs: `config`, `crypto` (new capability)
- Affected code: `src/config` (new field + validation), `src/crypto.rs` (new), `src/repositories/connection.rs` (use helpers for token fields)
- Dependencies: `aes-gcm` (Rust), `rand` (CSPRNG), optional `zeroize`
- Breaking: None at API level; startup fails if `POBLYSH_CRYPTO_KEY` invalid/missing.

