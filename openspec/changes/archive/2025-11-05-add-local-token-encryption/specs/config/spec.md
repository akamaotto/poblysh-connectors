## ADDED Requirements

### Requirement: Crypto Key Configuration
The system SHALL require a symmetric crypto key provided via `POBLYSH_CRYPTO_KEY` to encrypt/decrypt tokens at rest using AES‑256‑GCM.

Details:
- `POBLYSH_CRYPTO_KEY` MUST be a base64‑encoded 32‑byte value (256‑bit key) using standard base64 encoding with proper padding.
- Validation MUST check for: missing environment variable, invalid base64 characters, incorrect padding, and exact 32-byte decoded length.
- Startup MUST fail with a clear error if the key is missing or invalid in any profile.
- The key value MUST be treated as a secret and redacted in logs and diagnostics.
- Error messages MUST indicate the specific validation failure without exposing partial key material.

#### Scenario: Missing key causes startup failure
- **GIVEN** `POBLYSH_CRYPTO_KEY` is unset
- **WHEN** the service starts
- **THEN** startup fails with an actionable error indicating the crypto key is required

#### Scenario: Invalid base64 length rejected
- **GIVEN** `POBLYSH_CRYPTO_KEY=Zm9v` (base64 for 3 bytes)
- **WHEN** the service starts
- **THEN** startup fails indicating the key must decode to exactly 32 bytes

#### Scenario: Invalid base64 characters rejected
- **GIVEN** `POBLYSH_CRYPTO_KEY=invalid!@#$%^&*()`
- **WHEN** the service starts
- **THEN** startup fails with a base64 format validation error

#### Scenario: Invalid base64 padding rejected
- **GIVEN** `POBLYSH_CRYPTO_KEY=YWJjZGVm` (missing padding)
- **WHEN** the service starts
- **THEN** startup fails indicating invalid base64 padding

