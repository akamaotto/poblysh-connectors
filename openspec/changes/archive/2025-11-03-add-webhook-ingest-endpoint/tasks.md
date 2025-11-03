## 1. Implementation
- [x] 1.1 Add route `POST /webhooks/{provider}`; annotate in OpenAPI
- [x] 1.2 Handler: require operator auth and `X-Tenant-Id` (UUID); parse `provider` slug and optional `X-Connection-Id`
- [x] 1.3 Validate provider exists; if `X-Connection-Id` present, validate connection belongs to tenant and provider
- [x] 1.4 Enqueue `sync_jobs` row with `job_type='webhook'` referencing `(tenant_id, provider_slug, connection_id)`; optionally include minimal context in `cursor`
- [x] 1.5 Return `202 Accepted` with `{ status: "accepted" }` body

## 2. Tests
- [x] 2.1 202 for known provider with auth/tenant
- [x] 2.2 404 for unknown provider
- [x] 2.3 404 when `X-Connection-Id` is invalid for tenant/provider
- [x] 2.4 Job enqueued when valid connection provided

