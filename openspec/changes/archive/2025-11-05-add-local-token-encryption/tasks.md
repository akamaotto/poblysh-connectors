## 1. Implementation
- [x] 1.1 Add `POBLYSH_CRYPTO_KEY` to config: parse base64 → 32 bytes; fail fast if missing/invalid
- [x] 1.2 Create `src/crypto.rs` with AES‑256‑GCM helpers: `encrypt_bytes(plaintext, aad) -> Vec<u8>`, `decrypt_bytes(ciphertext, aad) -> Result<Vec<u8>, CryptoError>`
- [x] 1.3 Use CSPRNG (e.g., `rand::rngs::OsRng`) for 12‑byte nonces; payload format `0x01 | nonce | ct | tag`
- [x] 1.4 Wire repository token paths to use helpers before write and for read/decode where needed
- [x] 1.5 Add unit tests for encrypt/decrypt, wrong key/context failure, and nonce uniqueness
- [x] 1.6 Redact crypto key in config debug logs (ensure no accidental prints)

## 2. Dependencies
- [x] 2.1 Add crates: `aes-gcm = { version = "^0.10", features = ["aes256"] }`, `rand = "^0.8"`, `base64 = "^0.21"`, `zeroize = "^1"` (optional but recommended)

## 3. Migration / Rollout
- [x] 3.1 No DB schema changes; new writes will be encrypted
- [x] 3.2 If existing plaintext tokens exist (should not), add a one‑off migration script to re‑encrypt
- [x] 3.3 Document `POBLYSH_CRYPTO_KEY` in README and add to `.env.example`

## 4. Validation
- [x] 4.1 Run unit tests for crypto module
- [x] 4.2 Manual integration: create a connection, set tokens, verify DB columns hold binary payload starting with `0x01`
- [x] 4.3 Negative tests: wrong key/context produces auth failure
