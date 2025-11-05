# crypto Specification

## Purpose
TBD - created by archiving change add-local-token-encryption. Update Purpose after archive.
## Requirements
### Requirement: Token Encryption (AES-256-GCM)
The system SHALL encrypt and authenticate access/refresh tokens at rest using AES‑256‑GCM with a 256‑bit key derived from `POBLYSH_CRYPTO_KEY`.

Operational details (v1):
- Key: base64‑decoded 32 bytes from `POBLYSH_CRYPTO_KEY`.
- Nonce: 12‑byte random nonce from a CSPRNG, unique per encryption operation.
- AAD: UTF-8 encoded string `"{tenant_id}|{provider_slug}|{external_id}"` where components are raw strings (no URL encoding), using `|` as literal separator, to bind ciphertext to its connection context.
- Payload format: `0x01 | nonce(12) | ciphertext(variable_length) | tag(16)` stored verbatim in `BYTEA` columns, where total length = 29 + ciphertext_length bytes.
- Decryption MUST verify authentication tag; failures MUST not leak plaintext and SHOULD map to a generic decryption error.

#### Scenario: Encrypts on write with unique nonce
- **GIVEN** a plaintext token `"xoxp-abc"` and a valid key
- **WHEN** encrypting with AAD from a connection `(tenant, provider, external_id)`
- **THEN** the result bytes are non‑empty and begin with `0x01`, contain a 12‑byte nonce, and differ across two encryptions of the same plaintext

#### Scenario: Decrypts to original with correct context
- **GIVEN** ciphertext produced for `(tenant=T1, provider='slack', external_id='org:42')`
- **WHEN** decrypting with the same key and AAD from that connection
- **THEN** the original plaintext token is returned

#### Scenario: Wrong key or context fails authentication
- **GIVEN** ciphertext produced under key K and AAD bound to `(T1,'slack','org:42')`
- **WHEN** decrypting with a different key or with AAD for `(T1,'slack','org:99')`
- **THEN** decryption fails with an authentication error and no plaintext is revealed

### Requirement: Repository Token Handling
Token fields MUST be encrypted before persisting and decrypted via helpers when needed by business logic. Raw plaintext tokens SHALL NOT be persisted or logged.

#### Scenario: Repository update encrypts tokens
- **WHEN** calling a token update path that persists `access_token` or `refresh_token`
- **THEN** the repository stores AES‑GCM ciphertext bytes in `access_token_ciphertext` and/or `refresh_token_ciphertext`

#### Scenario: No accidental plaintext persistence
- **GIVEN** instrumented logging at debug level
- **WHEN** persisting or loading connections
- **THEN** no plaintext token values appear in logs; only ciphertext or redacted placeholders

