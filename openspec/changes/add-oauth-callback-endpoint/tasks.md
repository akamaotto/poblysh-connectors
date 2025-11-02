## 1. Implementation
- [ ] 1.1 Add route `GET /connect/{provider}/callback` in router (`src/server.rs`) and mark as public (bypass auth/tenant guards).
- [ ] 1.2 Implement handler `oauth_callback` (e.g., `src/handlers/connect.rs`) that:
  - [ ] Extracts `{provider}`, `code`, `state` from query/path
  - [ ] Validates and decodes `state` to resolve `tenant_id` and CSRF nonce
  - [ ] Resolves connector from registry; returns 404 via `ApiError` if unknown
  - [ ] Calls `connector.exchange_token(code, state)` and persists a `Connection` for the tenant
  - [ ] Returns `{ connection: { id, provider, expires_at?, metadata } }`
- [ ] 1.3 Document endpoint with utoipa (path + query params, responses).

## 2. Tests
- [ ] 2.1 Happy path returns 200 with `connection` object and persists row.
- [ ] 2.2 Unknown provider returns 404 `not_found`.
- [ ] 2.3 Missing/invalid state returns 400 `validation_failed`.
- [ ] 2.4 Missing code returns 400 `validation_failed`.
- [ ] 2.5 Upstream provider 503 during exchange returns 502 `provider_error` with details.

## 3. Docs
- [ ] 3.1 Verify Swagger shows `/connect/{provider}/callback` with parameters and schema.

