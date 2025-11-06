## 1. Implementation
- [x] 1.1 Add `TokenRefreshConfig { tick_seconds=3600, lead_time_seconds=600, concurrency=4, jitter_factor=0.1 }` to application config with env vars (`POBLYSH_TOKEN_REFRESH_TICK_SECONDS`, `POBLYSH_TOKEN_REFRESH_LEAD_TIME_SECONDS`, `POBLYSH_TOKEN_REFRESH_CONCURRENCY`, `POBLYSH_TOKEN_REFRESH_JITTER_FACTOR`).
- [x] 1.2 Implement a background refresher loop: select active connections with `expires_at <= now() + lead_time` and refresh them, applying jitter across the batch to avoid spikes.
- [x] 1.3 Define connector `refresh_token(connection)` to return updated tokens and `expires_at`; update connection securely (encrypt fields) via repository.
- [x] 1.4 On-demand: in executor/webhook paths, when receiving a provider 401, trigger `refresh_token`, persist, and retry the failing call once.
- [x] 1.5 Error handling: if refresh fails permanently (e.g., invalid_grant), set connection `status='error'` (or `revoked` if detectable) and emit structured error.
- [x] 1.6 Metrics: counters for attempts/success/failure, histogram for refresh latency, and gauge for connections nearing expiry.

## 2. Validation
- [x] 2.1 Unit tests for due-for-refresh selection logic and jitter distribution bounds.
- [x] 2.2 Integration test for on-demand refresh on 401: stub connector returns 401 first, refresh succeeds, retry succeeds.
- [x] 2.3 Verify encryption path updates both access and refresh tokens and `expires_at` atomically.

## 3. Implementation Summary

**✅ FULLY COMPLETED** - All 9 tasks successfully implemented

### Key Features Delivered:
- **Background Service**: TokenRefreshService with configurable intervals, jitter, and concurrency
- **On-Demand Refresh**: 401 error detection with automatic retry in sync executor and webhook handlers
- **Single-Flight Protection**: Prevents concurrent refresh attempts for same connection
- **Smart Error Handling**: Classifies permanent vs transient failures with appropriate connection status updates
- **Comprehensive Metrics**: Full observability with counters, histograms, and gauges
- **Security**: Proper encrypted token storage and handling

### Files Modified:
- `src/config/mod.rs` - TokenRefreshConfig with environment variables
- `src/token_refresh.rs` - Complete background service implementation
- `src/sync_executor.rs` - On-demand refresh integration
- `src/connectors/trait_.rs` - refresh_token method definition
- `src/server.rs` - AppState integration and service startup

## 4. Notes / Non-goals
- No changes to API surface.
- No provider-specific rate limit handling beyond existing Rate Limit Policy.
