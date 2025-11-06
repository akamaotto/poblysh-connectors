# Local Crypto Key Rotation (Dev/Local)

Scope: Local/dev environments only. For production, draft a separate runbook with key escrow, audit, and downtime planning.

## Prerequisites
- App uses AES‑256‑GCM to encrypt tokens with a single key from `POBLYSH_CRYPTO_KEY` (base64, 32 bytes post‑decode).
- You have a working backup or can recreate local connections/tokens.

## Option A (Recommended for local/dev): Recreate tokens
Simple, safe, minimal effort when preserving tokens is not needed.

1) Stop the app
2) Generate a new key (standard base64; 32 bytes after decode):
   - macOS/Linux: `head -c 32 /dev/urandom | base64`
   - OpenSSL alt: `openssl rand -base64 32`
3) Update `.env.local` or environment: `POBLYSH_CRYPTO_KEY="<base64-string>"`
4) Wipe or recreate connections/tokens locally:
   - Either reset local DB rows for connections, or remove/reseed dev data
   - Reconnect providers through the normal OAuth flow to mint new tokens
5) Start the app and verify decrypt/encrypt paths work (no errors on connect/list APIs)

Notes
- This avoids multi‑key decrypt logic and is the fastest way to rotate locally.
- Ensure the key is standard base64 (not URL‑safe) so the decoder matches.

## Option B (Advanced): Preserve tokens (future maintenance command)
Use when you want to keep local tokens and avoid reconnecting.

Current state: The app uses a single key; preserving tokens requires either a temporary multi‑key decrypt path or a one‑off maintenance utility to decrypt with the old key and re‑encrypt with the new key.

Proposed approach (to be implemented in a future change):
1) Stop the app and set two env vars:
   - `POBLYSH_CRYPTO_KEY_OLD` = old base64 key
   - `POBLYSH_CRYPTO_KEY` = new base64 key (used for encryption)
2) Run a maintenance command:
   - Iterates all rows with ciphertext
   - Decrypts using OLD key + AAD (tenant|provider|external_id)
   - Re‑encrypts with NEW key, fresh nonce, same AAD
   - Updates ciphertext atomically per row; tracks progress and failures
3) Start the app using only `POBLYSH_CRYPTO_KEY` (new key)

Operational safeguards
- Abort if either key fails base64 decode (not 32 bytes)
- Log only row ids and counts, never plaintext tokens or raw keys
- On failure, leave the row unchanged and continue; output a CSV of pending failures for retry

## Validation Checklist
- [ ] App boots with new key (no startup validation errors)
- [ ] Reads/writes token fields without decryption failures
- [ ] No plaintext token values appear in logs
- [ ] For Option B (future), re‑encryption job reports 0 pending rows

## References
- Change: `openspec/changes/add-local-token-encryption/`
- Crates: `aes-gcm 0.10.3`, `zeroize 1.8.1`, `base64 0.22.1`

