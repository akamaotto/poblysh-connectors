## 1. Implementation
- [x] 1.1 Add dev dependencies: `assert_cmd` (2.x) and `portpicker` (0.1.x)
- [x] 1.2 Create `tests/e2e_smoke_tests.rs` to spawn the `connectors` binary and probe endpoints
- [x] 1.3 Implement pre-flight environment validation for `POBLYSH_DATABASE_URL` and `POBLYSH_OPERATOR_TOKEN`
- [x] 1.4 Implement port selection with `portpicker` and explicit 127.0.0.1 binding with retry logic
- [x] 1.5 Implement readiness wait loop on `/readyz` with 60s timeout, 200-500ms backoff, and detailed error reporting
- [x] 1.6 Validate core endpoints: `/`, `/healthz`, `/readyz`, `/openapi.json`, `/providers`
- [x] 1.7 Validate protected flow: `/protected/ping` with `Authorization: Bearer <token>` and generated `X-Tenant-Id`
- [x] 1.8 Ensure graceful shutdown of the child process (SIGTERM + force kill on timeout)
- [x] 1.9 Add `make smoke` and `just smoke` shortcuts with comprehensive environment validation
- [x] 1.10 Update README with local DB guidance, environment setup, and smoke test instructions

## 2. Validation & QA
- [ ] 2.1 Postgres path: run via Docker (`postgres:15`), set `POBLYSH_DATABASE_URL`, and verify smoke passes (primary validation)
- [x] 2.2 SQLite path: point `POBLYSH_DATABASE_URL=sqlite://dev.db` and verify migrations + smoke pass (secondary validation)
- [x] 2.3 Environment validation: test with missing `POBLYSH_DATABASE_URL` or `POBLYSH_OPERATOR_TOKEN` to verify clear error guidance
- [ ] 2.4 Failure messages: disconnect DB to confirm clear and actionable error output with DB URL and port details
- [ ] 2.5 Port conflict path: simulate occupied port; confirm retry logic and clear failure guidance
- [ ] 2.6 Migration consistency: verify schema consistency between Postgres and SQLite when both available
- [ ] 2.7 Security validation: confirm 127.0.0.1 binding only (no 0.0.0.0 exposure during testing)

## 3. CI Considerations (Optional follow‑up)
- [ ] 3.1 Add a CI job that spins Postgres via `testcontainers` or service container and runs smoke
- [ ] 3.2 Gate E2E job by tag or env to keep standard PR jobs fast

