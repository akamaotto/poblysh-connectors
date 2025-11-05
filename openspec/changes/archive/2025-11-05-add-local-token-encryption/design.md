## Context
We need application‑level protection of third‑party access/refresh tokens at rest. The database already has `BYTEA` columns for ciphertext, but no encryption semantics. We will use a single process key from env to implement authenticated encryption.

## Goals / Non-Goals
- Goals: Encrypt tokens at rest; authenticate on decrypt; minimize blast radius if DB is leaked
- Non‑Goals: HSM/KMS integration, key rotation (will be considered in future changes)

## Decisions
- AES-256-GCM via `aes-gcm` crate for AEAD
- Key material: `POBLYSH_CRYPTO_KEY` base64-encoded 32 bytes
- Nonce: 12 bytes from `OsRng`, unique per encryption
- AAD: UTF-8 string `"{tenant_id}|{provider_slug}|{external_id}"` with raw string components (no URL encoding), using `|` as literal separator
- Payload: stored as `BYTEA` with exact layout `0x01 | nonce(12) | ciphertext(variable_length) | tag(16)` where total length = 29 + ciphertext_length bytes, and the trailing 16 bytes are the authentication tag.
- Helper API: `encrypt_bytes(&key, &aad, pt) -> Vec<u8>`, `decrypt_bytes(&key, &aad, ct) -> Result<Vec<u8>>`
- Startup validation: base64-decode `POBLYSH_CRYPTO_KEY` and assert it yields exactly 32 bytes, aborting startup if validation fails.
- Key handling: store decoded key material in a zeroizing buffer (e.g., `zeroize` crate) and wipe/overwrite bytes once no longer needed to minimize exposure.

Alternatives considered:
- ChaCha20‑Poly1305: similar properties; chose AES‑GCM due to broad HW acceleration and library maturity
- Deterministic AEAD (SIV modes): not needed; we prefer randomized nonces to avoid linkage

## Risks / Trade-offs
- Single key: compromise reveals all tokens → mitigate by strict key handling (never log or echo the key, prohibit exposure in CI/CD output, load from read-only env/secret store on boot, maintain secure backup/restore procedures, and document an incident response playbook for key rotation/re-issuance) and plan future KMS/rotation integration.
- Nonce reuse catastrophe: enforce fresh nonces via RNG (ensure `OsRng` is never seeded deterministically), add concurrency tests verifying nonce uniqueness across parallel encryptions, and keep the API internal-only so callers cannot supply nonces.
- Monitoring & alerting: capture metrics/logs for decrypt failures (e.g., AAD mismatch or tag verification errors) and alert on spikes to detect tampering, context confusion, or misuse.

## Migration Plan
- No schema change. New writes encrypt; existing rows unaffected
- Runtime compatibility: on read, inspect the first byte; if it is not `0x01`, treat the value as legacy plaintext, emit a structured warning for telemetry, and plan deprecation once the column is fully migrated.
- Backfill strategy: run an asynchronous migration job post-deployment that reads plaintext rows, encrypts them in batches, and tracks remaining plaintext count; define a completion deadline (e.g., <2 weeks) and surface job metrics to operations dashboards.
- Rollback: gate new encryption behind a feature flag so we can pause encryption and fall back to plaintext reads if we discover a bug; retain the ability to decrypt encrypted rows back to plaintext through a maintenance command for emergency rollback while auditing all access.
- Monitoring: expose metrics for plaintext vs encrypted row counts, backfill progress, and decryption fallback events to ensure the transition stays within the expected timeline.
- Document required env var and operational key handling

## Open Questions

### Key Rotation Strategy (Future Work)
**Owner**: Security Team Lead  
**Decision Deadline**: Q1 2025 (before production deployment)  
**Consideration**: Versioned multi-key decrypt with single-key encrypt approach to support seamless key rotation without service interruption. Research KMS integration options.

### AAD Component Inclusion (Future Enhancement)  
**Owner**: Crypto Architecture Review  
**Decision Deadline**: Next security review cycle  
**Consideration**: Evaluate adding connection `id` to AAD for additional binding against ID reuse scenarios. Current `(tenant, provider, external_id)` provides sufficient uniqueness for current threat model.

