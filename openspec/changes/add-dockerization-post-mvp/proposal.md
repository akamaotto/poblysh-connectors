## Why
Post‑MVP, we need a reproducible, portable runtime to prepare for deployment and team onboarding. Containerizing the API and provisioned Postgres improves parity between local/dev/staging, enables CI smoke tests against containers, and clarifies operational requirements (ports, env, health checks, migrations).

## What Changes
- Add multi‑stage Dockerfile to build and run the `connectors` binary (non‑root runtime, UID/GID 10001, minimal base).
- Add Docker Compose with services: `db` (Postgres 16), `migrate` (one‑off), and `app` (API), using explicit `depends_on` conditions: `db: service_healthy`, `migrate: service_completed_successfully`.
- Wire configuration via `POBLYSH_*` env vars (no config files baked into images).
- Document image build/publish, local compose flows, and health semantics.
- Optional: provide MUSL build variant for static binary and distroless runtime.

## Impact
- Affected specs: add new "ops" capability for containerization and compose orchestration.
- Affected code: no functional changes required; reuse existing `connectors migrate up` CLI and `/healthz` + `/readyz` endpoints.
- CI/CD: future work can build/push image and run containerized tests.

## Acceptance Criteria
- Building the Docker image succeeds on a clean machine with Docker installed.
- `docker compose up` starts `db` and reports `service_healthy`, runs `migrate` to completion (`service_completed_successfully`), and `app` becomes healthy; `GET /readyz` returns 200 within 60s.
- `POBLYSH_DATABASE_URL` uses the compose network host `db` and matches the configured credentials.
- Container runs as non‑root with UID/GID 10001 and exposes only the API port (8080) with structured logs to stdout.
