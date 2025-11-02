## Context
We need application‑level protection of third‑party access/refresh tokens at rest. The database already has `BYTEA` columns for ciphertext, but no encryption semantics. We will use a single process key from env to implement authenticated encryption.

## Goals / Non-Goals
- Goals: Encrypt tokens at rest; authenticate on decrypt; minimize blast radius if DB is leaked
- Non‑Goals: HSM/KMS integration, key rotation (will be considered in future changes)

## Decisions
- AES‑256‑GCM via `aes-gcm` crate for AEAD
- Key material: `POBLYSH_CRYPTO_KEY` base64‑encoded 32 bytes
- Nonce: 12 bytes from `OsRng`, unique per encryption
- AAD: `tenant_id|provider_slug|external_id` to bind ciphertext to its connection context
- Payload: `0x01 | nonce(12) | ciphertext | tag(16)` stored as `BYTEA`
- Helper API: `encrypt_bytes(&key, &aad, pt) -> Vec<u8>`, `decrypt_bytes(&key, &aad, ct) -> Result<Vec<u8>>`

Alternatives considered:
- ChaCha20‑Poly1305: similar properties; chose AES‑GCM due to broad HW acceleration and library maturity
- Deterministic AEAD (SIV modes): not needed; we prefer randomized nonces to avoid linkage

## Risks / Trade-offs
- Single key: compromise reveals all tokens → mitigate by strict key handling, redaction, and future KMS/rotation
- Nonce reuse catastrophe: enforce fresh nonces via RNG, add tests, keep API simple to avoid callers supplying nonces

## Migration Plan
- No schema change. New writes encrypt; existing rows unaffected
- If legacy plaintext rows exist, run a targeted backfill script during rollout
- Document required env var and operational key handling

## Open Questions
- Key rotation strategy (versioned multi‑key decrypt, single‑key encrypt) for a future change
- Whether to include connection `id` in AAD in addition to `(tenant, provider, external_id)`

