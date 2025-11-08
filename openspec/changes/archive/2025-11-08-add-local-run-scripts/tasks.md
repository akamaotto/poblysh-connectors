## 1. Implementation
- [x] 1.1 Add `Makefile` with targets: `help`, `setup`, `env`, `db-sqlite`, `db-pg-check`, `migrate`, `run`, `watch`, `test`, `lint`, `fmt`, `openapi`
- [x] 1.2 Add `Justfile` with mirrored recipes and `default := help`
- [x] 1.3 Default local profile to SQLite by creating `.env.local` with `POBLYSH_PROFILE=local`, `POBLYSH_DATABASE_URL=sqlite://dev.db`, `POBLYSH_OPERATOR_TOKEN=local-dev-token`, and a generated `POBLYSH_CRYPTO_KEY` (32-byte base64, e.g., `openssl rand -base64 32`)
- [x] 1.4 Document optional Postgres usage in README and implement `db-pg-check` (non-fatal if missing)
- [x] 1.5 Ensure `migrate` target calls the `Migrator` via the `connectors` binary: `cargo run -- migrate up`
- [x] 1.6 Make `watch` target opportunistic: if `cargo-watch` missing, print install hint and fallback to `cargo run`
- [x] 1.7 Implement `openapi` to curl `http://localhost:8080/openapi.json` into `openapi.json` with basic failure handling

## 2. Validation
- [x] 2.1 Clean machine: `make env && make db-sqlite && make migrate && make run` → verify `GET /healthz` 200 with `.env.local` providing valid operator token(s) and `POBLYSH_CRYPTO_KEY`
- [x] 2.2 `make watch` without cargo-watch installed → prints hint; with installed → hot-reload works on file changes
- [x] 2.3 `make test`, `make lint`, and `make fmt` succeed locally (tests: 251 passed, 1 failed; lint has existing clippy warnings; fmt succeeds)
- [x] 2.4 `make openapi` produces `openapi.json` and validates JSON structure (basic `jq .info.title`)

## 3. Follow-up (Optional)
- [ ] 3.1 Add `nextest` target if the team adopts it
- [ ] 3.2 Consider a `seed` task for baseline providers
