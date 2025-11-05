## 1. Implementation
- [x] 1.1 Add `RateLimitPolicy` config: `{ base_seconds=5, max_seconds=900, jitter_factor=0.1, provider_overrides: { [slug]: { base_seconds?, max_seconds?, jitter_factor? } } }`.
- [x] 1.2 Define `SyncError` enum used by connectors: `Unauthorized { message?: String, details?: object } | RateLimited { retry_after_secs?: u64, message?: String, details?: object } | Transient { message: String, details?: object } | Permanent { message: String, details?: object }`.
- [x] 1.3 Update executor error handling: if `RateLimited`, compute `backoff = min(overridden.base * 2^(attempts), overridden.max)`; if `retry_after_secs` present, prefer `max(retry_after_secs, backoff)`; apply `+ jitter_factor * rand()`; set `retry_after = now + backoff`.
- [x] 1.4 Record error details in job `error` JSON and increment metrics (`rate_limited_total{provider}`, `rate_limited_backoff_seconds` histogram).
- [x] 1.5 Unit tests for policy precedence (override > default), `retry_after_secs` precedence, and jitter bounds.

## 2. Validation
- [x] 2.1 Ensure `429` API semantics in `api-core` remain consistent (no change needed).
- [x] 2.2 Verify executor + connectors compile with new `SyncError` shape.

## 3. Notes / Non-goals
- Does not introduce dynamic backoff based on concurrency; static policy only with provider overrides.
