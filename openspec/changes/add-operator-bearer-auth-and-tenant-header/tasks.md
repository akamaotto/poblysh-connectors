## 1. Implementation
- [ ] 1.1 Config: read `POBLYSH_OPERATOR_TOKENS` (comma‑separated) or fallback `POBLYSH_OPERATOR_TOKEN`; require at least one token in `local`/`test`, fail startup otherwise in non‑local profiles.
- [ ] 1.2 Auth: add middleware/extractor in `src/auth.rs`:
      - Parse `Authorization: Bearer <token>` and compare in constant time.
      - Parse `X-Tenant-Id` as UUID; attach to request extensions/state.
      - On failure, return unified errors (401 unauthorized; 400 validation_failed).
- [ ] 1.3 Server wiring: apply guard to protected routes; bypass `/healthz`, `/readyz`, `/docs`, `/openapi.json`.
- [ ] 1.4 OpenAPI: declare HTTP bearer security scheme; annotate protected endpoints; document required `X-Tenant-Id` header.
- [ ] 1.5 Tests: unit tests for header parsing and constant‑time match; integration tests verifying 401/400 and success path.

## 2. Validation
- [ ] 2.1 `openspec validate add-operator-bearer-auth-and-tenant-header --strict` passes.
- [ ] 2.2 `cargo test -q` passes including integration tests for the guard.

## 3. Notes / Non-goals
- No RBAC or per‑tenant authorization model in this change; single operator role only.
- No JWT/OAuth here; simple bearer tokens for MVP.
- Do not log bearer tokens; log tenant ID and trace ID only.

