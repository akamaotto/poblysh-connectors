## 1. Implementation
- [ ] 1.1 Add route `POST /connect/{provider}` in router (`src/server.rs`).
- [ ] 1.2 Implement handler `start_oauth` (e.g., `src/handlers/connect.rs`) that:
  - [ ] Extracts `{provider}` and tenant ID from headers/context
  - [ ] Resolves connector from registry; returns 404 via `ApiError` if unknown
  - [ ] Calls `connector.authorize(tenant)` and returns `{ authorize_url }`
- [ ] 1.3 Document endpoint with utoipa (path param, security, responses).

## 2. Tests
- [ ] 2.1 Happy path returns 200 with `authorize_url`.
- [ ] 2.2 Unknown provider returns 404 `not_found`.
- [ ] 2.3 Missing/invalid bearer token returns 401.
- [ ] 2.4 Missing tenant header returns 400.

## 3. Docs
- [ ] 3.1 Verify Swagger shows `/connect/{provider}` with security and schema.

