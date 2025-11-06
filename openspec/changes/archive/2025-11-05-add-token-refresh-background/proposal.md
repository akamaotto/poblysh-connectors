## Why
- Access tokens expire and require refresh to maintain uninterrupted sync and webhook handling.
- Without proactive refresh, jobs may fail until the next scheduler run, and on-demand refresh logic may be inconsistent across code paths.
- Centralizing a background refresher and on-demand refresh semantics ensures resilience and consistent behavior.

## What Changes
- Add a background Token Refresh Service that periodically scans active connections and refreshes tokens nearing expiry.
- Add on-demand refresh semantics: when a provider returns 401 unauthorized during sync/webhook processing, refresh tokens immediately and retry once.
- Introduce configuration for refresh interval, lead time threshold, jitter, and concurrency.
- Add structured metrics and tracing for refresh attempts, successes, failures, and retry behavior.

## Impact
- Specs: `sync-engine` gains Token Refresh Service; `connectors` defines `refresh_token` contract and error mapping; `config` gains refresh settings.
- Code: background task loop, repository helpers to select due-for-refresh connections, encryption-aware token update, retry-once logic after refresh on 401.
- No database schema changes; uses existing `connections` fields and repository update.

## Out of Scope
- UI or API endpoints for manual refresh.
- Provider-specific token rotation strategies beyond standard OAuth2 refresh_token.
- Long-lived tokens without refresh support (treated as no-op by refresher).
