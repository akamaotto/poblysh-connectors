# Poblysh Connectors (v0.1) — API Plan

## 1. Conventions
- Base URL: `/` (single service). Consider versioning prefix `/v1` in v0.2.
- Auth: `Authorization: Bearer <OPERATOR_TOKEN>` for operator endpoints; tenant scoping via `X-Tenant-Id` header.
- Content: JSON UTF-8; `application/json`.
- Pagination: `limit` (default 50, max 200), `cursor` string. Responses include `next_cursor` when more data exists.
- Errors: Problem+JSON style `{ code, message, details?, retry_after? }`; HTTP codes 4xx/5xx.
- Idempotency: `Idempotency-Key` header supported for `POST /sync/{provider}`.

## 2. Endpoints

### GET /providers
- Purpose: List available integrations with metadata.
- Request: none
- Response 200:
```json
{
  "providers": [
    {"name": "github", "auth_type": "oauth2", "scopes": ["repo", "admin:repo_hook"], "webhooks": true},
    {"name": "jira", "auth_type": "oauth2", "scopes": ["read:jira-work"], "webhooks": true}
  ]
}
```

### POST /connect/{provider}
- Purpose: Start OAuth flow; returns authorization URL for redirection.
- Request headers: `X-Tenant-Id`
- Response 200:
```json
{ "authorize_url": "https://provider.com/oauth/authorize?..." }
```

### GET /connect/{provider}/callback
- Purpose: OAuth callback handler; exchanges code for tokens.
- Query: `code`, `state`
- Response 302: Redirect to success page (MVP: JSON 200 is acceptable)
- Response 200 (MVP JSON):
```json
{ "connection_id": "...", "provider": "github" }
```

### GET /connections
- Purpose: List active connections for tenant.
- Headers: `X-Tenant-Id`
- Query: `provider?`
- Response 200:
```json
{
  "connections": [
    {"id": "...", "provider": "github", "expires_at": "2025-01-01T00:00:00Z", "metadata": {"installation_id": "..."}}
  ]
}
```

### POST /sync/{provider}
- Purpose: Manually trigger a sync job for all connections of provider (or specific connection if provided).
- Headers: `X-Tenant-Id`, `Idempotency-Key?`
- Body (optional): `{ "connection_id": "...", "mode": "incremental|full" }`
- Response 202:
```json
{ "job_id": "...", "state": "queued" }
```

### POST /webhooks/{provider}
- Purpose: Receive webhook events; verify signature; dispatch to connector handler.
- Security: Provider‑specific headers validation (e.g., `X-Hub-Signature-256`, Slack `X-Slack-Signature`).
- Response 200: `{ "accepted": true }`

### GET /jobs
- Purpose: Inspect job history.
- Headers: `X-Tenant-Id`
- Query: `provider?`, `state?`, `since?`, `until?`, `limit?`, `cursor?`
- Response 200:
```json
{
  "jobs": [
    {"id": "...", "provider": "github", "state": "succeeded", "started_at": "...", "finished_at": "..."}
  ],
  "next_cursor": "..."
}
```

### GET /signals
- Purpose: Query normalized Signals.
- Headers: `X-Tenant-Id`
- Query: `source?`, `kind?`, `since?`, `until?`, `limit?`, `cursor?`
- Response 200:
```json
{
  "signals": [
    {"id": "...", "source": "github", "kind": "pr_merged", "timestamp": "...", "payload": {"repo": "...", "pr": 123}}
  ],
  "next_cursor": "..."
}
```

### GET /healthz, GET /readyz
- Purpose: Liveness/readiness (DB connectivity and config checks only in MVP).
- Response 200: `{ "status": "ok" }`

## 3. Data Contracts

### Signal
```json
{
  "id": "uuid",
  "source": "github|jira|gmail|...",
  "kind": "issue_closed|pr_merged|email_sent|...",
  "timestamp": "RFC3339",
  "payload": {"raw": {}, "normalized": {}}
}
```

Normalization guidance:
- `kind` uses product‑agnostic verbs: `created`, `updated`, `deleted`, `closed`, `merged`, `sent`, `received`.
- Put provider specifics in `payload.raw`; expose common fields (repo, issue_key, subject, actor) in `payload.normalized` when possible.

### Job
```json
{ "id": "uuid", "connection_id": "uuid", "provider": "string", "job_type": "full|incremental", "state": "queued|running|succeeded|failed|retried", "started_at": "RFC3339", "finished_at": "RFC3339?", "error": "string?" }
```

### Connection
```json
{ "id": "uuid", "tenant_id": "uuid", "provider": "string", "expires_at": "RFC3339?", "metadata": {} }
```

## 4. Auth & Tenanting
- All tenant‑scoped endpoints require `X-Tenant-Id`.
- Operator token: Static bearer secret in MVP; rotate by updating `OPERATOR_TOKEN` env.
- Future: JWT/OIDC with tenant claims and fine‑grained roles.

## 5. Pagination & Filtering
- Time‑bounded queries recommended for `GET /signals` to ensure efficient index scans.
- `cursor` encodes last `(timestamp, id)`; stable ordering by `(timestamp asc, id asc)`.

## 6. Rate Limits & Backoff
- 429 for overload; include `Retry-After` header.
- Provider backoff: Job records persist `retry_after` and reschedule automatically.

## 7. Webhook Setup Notes
- GitHub: Create app; set webhook secret; subscribe to `push`, `pull_request`, `release`; deliver to `/webhooks/github`.
- Jira: Configure webhook filters for issue events; deliver to `/webhooks/jira`.
- Slack: Bot OAuth + Event Subscriptions; verify signing secret.
- Google Drive/Calendar: Channel watch creation; store `resource_id`, `channel_id`, `expiration`; renew before expiry.
- Gmail: Pub/Sub push endpoint; verify JWT; acknowledge events; optional poll fallback.
- Zoho: App auth + webhook with shared secret.

## 8. OpenAPI (utoipa) Outline
- Derive schemas on DTOs above; annotate endpoints with `#[utoipa::path(...)]`.
- Serve Swagger UI at `/docs` with CORS enabled for local testing.
