## Why
- Background sync jobs encounter provider rate limits; without a central policy, backoff behavior is inconsistent and may overload providers.
- Providers often return retry hints (headers/bodies). We need a single place to interpret them and schedule retries.
- Our public API already defines `429` with `Retry-After`; the sync engine needs parallel semantics via job `retry_after` and consistent jitter.

## What Changes
- Define a central Rate Limit Policy for the sync engine to interpret provider rate-limit responses and schedule retries using `retry_after` with jitter.
- Extend connector error semantics to surface rate-limit signals (with optional retry-after seconds) in a structured way to the executor.
- Add configuration for base/max backoff and jitter factor, plus provider-specific overrides.
- Emit metrics and tracing for rate-limit events (counts, backoff milliseconds, provider breakdown).

## Impact
- Specs: `sync-engine` gains a Rate Limit Policy; `connectors` modified to return structured rate-limit errors.
- Code: executor maps provider rate-limit signals into `sync_jobs.retry_after`; config gains `RateLimitPolicy` with overrides.
- No database schema changes; `sync_jobs.retry_after` is already defined and used.

## Out of Scope
- Distributed coordination or adaptive token bucket algorithms.
- Per-tenant fairness; initial scope is provider-wide hints + static policy.
