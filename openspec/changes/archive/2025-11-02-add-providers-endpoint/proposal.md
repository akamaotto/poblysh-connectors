## Why
Operators and UIs need a discoverable list of available integrations with metadata (auth type, scopes, webhook support). A lightweight, public listing endpoint also aids local testing and documentation (Swagger).

## What Changes
- Add HTTP endpoint: `GET /providers` to return provider registry metadata.
- Response: `{ providers: [{ name, auth_type, scopes[], webhooks }] }`.
- Sorting: Stable ascending by `name`.
- OpenAPI: Document endpoint and response schema in Swagger.
- Auth: Public (no tenant scoping); does not require `Authorization`.

## Impact
- Affected specs: `api-providers` (new capability)
- Affected code: likely `src/handlers/providers.rs`, `src/models/providers.rs`, router wiring in `src/server.rs`.
- Related changes: `add-operator-bearer-auth-and-tenant-header` (public endpoint bypass), `add-error-model-and-problem-json` (error envelope, though not used on 200).

