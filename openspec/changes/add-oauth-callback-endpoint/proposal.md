## Why
After initiating OAuth, providers redirect back to our service with an authorization `code` (and a `state` we issued). The API must handle this callback, validate state to resolve the tenant, exchange the code for tokens via the connector, and persist a tenant-scoped connection.

## What Changes
- Add HTTP endpoint: `GET /connect/{provider}/callback` to complete OAuth by exchanging `code` for tokens and creating a Connection.
- Public endpoint: Does not require operator bearer token or `X-Tenant-Id`; uses `state` to recover tenant context.
- Query: `code` (required), `state` (required for tenant + CSRF protection).
- Response (MVP): JSON `{ connection: { id, provider, expires_at?, metadata } }` on success.
- Errors: 400 for invalid/missing state or code; 502 for upstream provider HTTP failures; unified error envelope.
- OpenAPI: Document path/query params and responses in Swagger.

## Impact
- Affected specs: `api-connections` (adds OAuth callback requirement); relies on `auth` (public bypass) and `api-core` error model.
- Affected code: new handler (e.g., `src/handlers/connect.rs`), router wiring in `src/server.rs`, state validation util, call `connector.exchange_token(code, state)` and persist connection.
- Dependencies: `add-connector-trait-and-registry`, `add-error-model-and-problem-json`.

