## 1. Implementation
- [x] 1.1 Add route `GET /connect/{provider}/callback` in router (`src/server.rs`) and mark as public (bypass auth/tenant guards).
- [x] 1.2 Implement handler `oauth_callback` (e.g., `src/handlers/connect.rs`) that:
  - [x] Extracts `{provider}`, `code`, `state` from query/path
  - [x] Validates and decodes `state` to resolve `tenant_id` and CSRF nonce
  - [x] Resolves connector from registry; returns 404 via `ApiError` if unknown
  - [x] Calls `connector.exchange_token(code, state)` and persists a `Connection` for the tenant
  - [x] Returns `{ connection: { id, provider, expires_at?, metadata } }`
- [x] 1.3 Document endpoint with utoipa (path + query params, responses).

## 2. Tests
- [x] 2.1 Happy path returns 200 with `connection` object and persists row.
- [x] 2.2 Unknown provider returns 404 `not_found`.
- [x] 2.3 Missing/invalid state returns 400 `validation_failed`.
- [x] 2.4 Missing code returns 400 `validation_failed`.
- [x] 2.5 Upstream provider 503 during exchange returns 502 `provider_error` with details.

## 3. Docs
- [x] 3.1 Verify Swagger shows `/connect/{provider}/callback` with parameters and schema.

