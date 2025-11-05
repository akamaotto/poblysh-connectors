## Why
Access and refresh tokens for provider connections must be protected at rest. Today, the `connections` table holds opaque `*_ciphertext` fields with no defined encryption behavior. Introducing authenticated encryption using a single application key improves security immediately without changing external APIs or database schema.

## What Changes
- Add local token encryption module using AES‑256‑GCM via `aes-gcm` crate v0.10.3 with proven RustCrypto implementation.
- Add required `POBLYSH_CRYPTO_KEY` (base64‑encoded 32 bytes with strict validation) to derive the AEAD key using `zeroize` crate for secure memory handling.
- Define binary payload format: `0x01 | 12‑byte nonce | ciphertext(variable_length) | 16‑byte tag` with CSPRNG nonces from existing `rand` crate.
- Use AAD binding (`"{tenant_id}|{provider_slug}|{external_id}"` UTF-8 string) to context‑lock tokens to their connection.
- Update repository/token paths to encrypt before write and decrypt on read via helpers that handle legacy plaintext migration.
- Validation and errors for missing/invalid crypto key on startup with detailed error messaging.

## Impact
- Affected specs: `config`, `crypto` (new capability)
- Affected code: `src/config` (new field + validation), `src/crypto.rs` (new), `src/repositories/connection.rs` (use helpers for token fields)
- Dependencies: `aes-gcm = "0.10.3"`, `zeroize = "1.8.1"`, `rand = "0.8.5"` (existing), `base64 = "0.22.1"` (existing)
- Breaking: None at API level; startup fails if `POBLYSH_CRYPTO_KEY` invalid/missing.

## Acceptance Criteria

### Security Requirements
- [ ] All token fields are encrypted at rest using AES-256-GCM with 12-byte random nonces
- [ ] Encryption keys are derived from `POBLYSH_CRYPTO_KEY` using zeroizing memory management
- [ ] AAD binding prevents token reuse across different connections (tenant/provider/external_id)
- [ ] Authentication tag verification prevents ciphertext tampering
- [ ] Startup fails fast with clear error messages when crypto configuration is invalid

### Compatibility Requirements  
- [ ] Existing plaintext tokens remain readable during migration period
- [ ] Legacy rows emit structured telemetry warnings for migration tracking
- [ ] New writes are always encrypted, legacy rows are gradually migrated
- [ ] No API changes or breaking external behavior changes
- [ ] Database schema remains unchanged (existing `*_ciphertext` BYTEA fields)

### Operational Requirements
- [ ] `POBLYSH_CRYPTO_KEY` is base64-encoded 32-byte key with strict validation
- [ ] Key material is never logged or exposed in diagnostics
- [ ] Encryption/decryption performance meets application requirements (<5ms per operation)
- [ ] Monitoring tracks encryption vs plaintext row counts and migration progress
- [ ] Decrypt failures are logged as generic authentication errors without plaintext leakage

### Technical Implementation
- [ ] Uses `aes-gcm` crate v0.10.3 with proper AAD support
- [ ] Implements helper functions `encrypt_bytes()` and `decrypt_bytes()` 
- [ ] Repository layer abstracts encryption details from business logic
- [ ] Comprehensive test coverage for encryption, decryption, and error paths
- [ ] Integration tests verify end-to-end token persistence and retrieval

