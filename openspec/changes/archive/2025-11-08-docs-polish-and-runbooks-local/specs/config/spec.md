## ADDED Requirements
### Requirement: Crypto Key Documentation Coverage
The documentation SHALL describe the `POBLYSH_CRYPTO_KEY` configuration, including the base64-encoded 32-byte requirement, and provide a clear pointer to the local crypto rotation runbook for rotation steps.

#### Scenario: Developer understands key format
- **WHEN** a developer reads `docs/configuration.md`
- **THEN** they learn that `POBLYSH_CRYPTO_KEY` must be a standard base64 string that decodes to 32 bytes

#### Scenario: Developer locates rotation procedure
- **WHEN** a developer needs to rotate their local crypto key
- **THEN** the documentation links to `docs/runbooks/local-crypto-rotation.md`, providing the step-by-step process
