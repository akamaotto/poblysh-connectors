## 1. Implementation
- [x] 1.1 Add `GET /healthz` handler returning `{ status: "ok", service: "<service-id>", version: "<semver>" }` with JSON content type
- [x] 1.2 Add `GET /readyz` handler checking DB connectivity and pending migrations, reporting failures via `ApiError` (`code = SERVICE_UNAVAILABLE`, `details.checks`)
- [x] 1.3 Wire routes in `src/server.rs` and ensure auth bypass for both
- [x] 1.4 Add OpenAPI annotations for both endpoints
- [x] 1.5 Unit tests: health 200 no‑auth; readiness 200 with healthy DB; response structure validation with proper schema compliance and `details.checks` format

## 2. Operational
- [ ] 2.1 Update README with probe usage examples (curl, k8s probes)
