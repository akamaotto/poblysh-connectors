## Why
Operators need a simple, tenant-scoped way to initiate an OAuth connection flow for a specific provider. Returning an authorize URL from the API enables UIs to redirect users to the provider while the backend manages state, tenant context, and registry lookups.

## What Changes
- Add HTTP endpoint: `POST /connect/{provider}` (tenant-scoped) to start OAuth and return an authorize URL.
- Auth: Requires operator bearer token and `X-Tenant-Id` header.
- Path param: `{provider}` is the snake_case provider ID (e.g., `github`).
- Response:
  - `200 OK` – Returns `{ authorize_url: string }`, a fully formed provider authorization URL (includes state when applicable). The URL MUST be HTTPS, valid according to RFC 3986, with maximum length 2048 characters, and MUST NOT include a fragment component per OAuth 2.0 RFC 6749 section 3.1.
  - `400 Bad Request` – `ApiError` for invalid payloads or unsupported providers.
  - `401 Unauthorized` – `ApiError` when the bearer token is missing or invalid.
  - `403 Forbidden` – `ApiError` if the operator is not authorized for the tenant.
  - `404 Not Found` – `ApiError` when the provider slug does not exist.
  - `500 Internal Server Error` – `ApiError` for unexpected backend failures.
- OpenAPI: Document path param, security, success schema, and reference `#/components/schemas/ApiError` for the 400/401/403/404/500 responses in Swagger.

### OAuth Flow Details

- **State generation & storage:** Generate a cryptographically secure random state token per request, persist it in cache/DB keyed by `(tenant_id, provider, state)` with expiration, and associate any PKCE/verifier data.
- **Tenant context preservation:** State entry maps back to `tenant_id` and provider; callback validation must resolve the state, ensure tenant match, and reject mismatched or expired entries.
- **Callback endpoint:** Add `POST /connect/{provider}/callback` accepting provider callback payload (including `state` and provider-specific code/token); handler validates state, completes connector exchange via registry, and returns success or problem response.
- **Security protections:** Enforce CSRF/state validation, require PKCE where providers support it, sign/encrypt state payloads or use secure cookies when applicable, enforce expiration/replay protection, and audit all state consumption events.

## Impact
- Affected specs: `api-connections` (adds OAuth start endpoint requirement), references `auth` and `api-core` for guards and error model.
- Affected code: new handler (e.g., `src/handlers/connect.rs`), router wiring in `src/server.rs`, registry call via connector `authorize(tenant) -> Url`.
- Dependencies: `add-connector-trait-and-registry` (for `authorize`), `add-operator-bearer-auth-and-tenant-header`, `add-error-model-and-problem-json`.

