## 1. Implementation
- [ ] 1.1 Add `GET /healthz` handler returning `{ status: "ok", service, version }`
- [ ] 1.2 Add `GET /readyz` handler checking DB connectivity and pending migrations
- [ ] 1.3 Wire routes in `src/server.rs` and ensure auth bypass for both
- [ ] 1.4 Add OpenAPI annotations for both endpoints
- [ ] 1.5 Unit tests: health 200 no‑auth; readiness 200 with healthy DB; 503 when DB down or migrations pending

## 2. Operational
- [ ] 2.1 Update README with probe usage examples (curl, k8s probes)

