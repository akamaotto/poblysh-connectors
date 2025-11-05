## 1. Implementation
- [x] 1.1 Add route `GET /jobs` in API router; require `Authorization` and `X-Tenant-Id`.
- [x] 1.2 Define query struct with `status?`, `provider?`, `connection_id?`, `job_type?`, `started_after?`, `finished_after?`, `limit?` (default 50, max 100), and `cursor?`; enforce enum bounds (`status` in `queued|running|succeeded|failed`, `job_type` in `full|incremental|webhook`) and validate UUID/timestamp formats.
- [x] 1.3 Ensure `started_after` and `finished_after` filters can be applied independently or together (`started_at >=` value, `finished_at >=` value).
- [x] 1.4 Implement repository method to list tenant-scoped jobs with ordering `scheduled_at DESC, id DESC` and cursor continuation.
- [x] 1.5 Implement opaque cursor encoding/decoding (e.g., base64 of `{ scheduled_at, id }`) and return 400 unified errors for malformed or stale cursors.
- [x] 1.6 Return `{ jobs: [...], next_cursor }` always (string when more data, `null` otherwise); map validation errors via unified error envelope.
- [x] 1.7 OpenAPI: add schemas for `Job`, `JobsResponse`, document query params (including enums), and list possible error responses.
- [x] 1.8 Tests: auth/tenant enforcement, filters (`status`, `provider`, `connection_id`, `job_type`, `started_after`, `finished_after`, combined time filters), empty list, pagination continuation, malformed cursor, invalid params (limit > 100, bad enums, bad UUID/timestamp).

## 2. Notes / Non-goals
- No mutation endpoints in this change.
- Sorting and fields are fixed; sorting customization is out of scope.
