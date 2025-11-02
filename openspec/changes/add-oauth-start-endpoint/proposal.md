## Why
Operators need a simple, tenant-scoped way to initiate an OAuth connection flow for a specific provider. Returning an authorize URL from the API enables UIs to redirect users to the provider while the backend manages state, tenant context, and registry lookups.

## What Changes
- Add HTTP endpoint: `POST /connect/{provider}` (tenant-scoped) to start OAuth and return an authorize URL.
- Auth: Requires operator bearer token and `X-Tenant-Id` header.
- Path param: `{provider}` is the snake_case provider ID (e.g., `github`).
- Response: `{ authorize_url: string }` – fully formed provider authorization URL (includes state if applicable).
- OpenAPI: Document path param, security, and response schema in Swagger.

## Impact
- Affected specs: `api-connections` (adds OAuth start endpoint requirement), references `auth` and `api-core` for guards and error model.
- Affected code: new handler (e.g., `src/handlers/connect.rs`), router wiring in `src/server.rs`, registry call via connector `authorize(tenant) -> Url`.
- Dependencies: `add-connector-trait-and-registry` (for `authorize`), `add-operator-bearer-auth-and-tenant-header`, `add-error-model-and-problem-json`.

