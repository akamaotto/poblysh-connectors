## ADDED Requirements

### Requirement: Crypto Key Configuration
The system SHALL require a symmetric crypto key provided via `POBLYSH_CRYPTO_KEY` to encrypt/decrypt tokens at rest using AES‑256‑GCM.

Details:
- `POBLYSH_CRYPTO_KEY` MUST be a base64‑encoded 32‑byte value (256‑bit key).
- Startup MUST fail with a clear error if the key is missing or invalid in any profile.
- The key value MUST be treated as a secret and redacted in logs and diagnostics.

#### Scenario: Missing key causes startup failure
- GIVEN `POBLYSH_CRYPTO_KEY` is unset
- WHEN the service starts
- THEN startup fails with an actionable error indicating the crypto key is required

#### Scenario: Invalid base64 length rejected
- GIVEN `POBLYSH_CRYPTO_KEY=Zm9v` (base64 for 3 bytes)
- WHEN the service starts
- THEN startup fails indicating the key must decode to 32 bytes

