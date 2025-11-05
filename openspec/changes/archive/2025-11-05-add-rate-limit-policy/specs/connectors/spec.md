## MODIFIED Requirements
### Requirement: Connector Trait Errors For Rate Limits
Connector `sync` implementations SHALL surface provider rate limits via a structured error to enable centralized policy handling.

#### Scenario: Sync returns rate-limited error with optional hint
- **WHEN** the provider responds with a rate-limit (e.g., HTTP 429 or equivalent)
- **THEN** the connector returns `RateLimited { retry_after_secs?: number, details?: object }`
- **AND** `retry_after_secs` reflects any reliable provider hint (header/body)
- **AND** `message` MAY include a human-readable explanation for observability
- **AND** `details` MAY include provider metadata (e.g., bucket, limit, reset_at)

#### Scenario: Non-rate-limit transient errors
- **WHEN** a transient network or 5xx provider error occurs
- **THEN** the connector returns a transient error variant (implementation-defined), allowing executor generic backoff handling

#### Scenario: Permanent errors are not retried
- **WHEN** the connector indicates a permanent error (e.g., invalid credentials)
- **THEN** the executor MUST mark the job failed without scheduling a retry (out-of-scope behavior MAY be specified in a later change)

### Requirement: Sync Result And Error Contract
Connector `sync` MUST return `Result<SyncResult, SyncError>`.

Definitions:
- `SyncResult` as defined in the executor change (fields: `signals: [Signal]`, `next_cursor?: Cursor`, `has_more: bool`).
- `SyncError` variants (MVP):
  - `Unauthorized { message?: string, details?: object }` (maps provider 401)
  - `RateLimited { retry_after_secs?: number, message?: string, details?: object }`
  - `Transient { message: string, details?: object }`
  - `Permanent { message: string, details?: object }`

#### Scenario: Return type enforces rate-limit handling
- **WHEN** a connector encounters a provider rate limit
- **THEN** it returns `Err(SyncError::RateLimited { retry_after_secs?, message?, details? })` and does not emit partial signals
