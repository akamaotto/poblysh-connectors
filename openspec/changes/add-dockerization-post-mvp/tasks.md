## 1. Implementation
- [ ] 1.1 Add `.dockerignore` matching `target/`, `**/.git`, `**/.env*`, `**/tests`, `**/docs` (keep `migration/`).
- [ ] 1.2 Create multi‑stage `Dockerfile` (builder: rust slim; runtime: debian slim non‑root UID/GID 10001; expose 8080; copy binary only).
- [ ] 1.3 Add `docker-compose.yml` with services: `db` (postgres:16‑alpine), `migrate` (one‑off `connectors migrate up`), `app`.
      Use explicit conditions: `db` → `depends_on: condition: service_healthy`; `migrate` → depends on `db: service_healthy`; `app` → depends on `db: service_healthy` and `migrate: service_completed_successfully`.
- [ ] 1.4 Define compose health checks: `db` with `pg_isready`; `app` probes `/readyz` with a 60s start‑period.
- [ ] 1.5 Wire env: `POBLYSH_PROFILE=dev`, `POBLYSH_API_BIND_ADDR=0.0.0.0:8080`, `POBLYSH_DATABASE_URL=postgresql://pbl:secret@db:5432/connectors`, `POBLYSH_OPERATOR_TOKEN=dev-operator`.
- [ ] 1.6 Document `docker build`, `docker compose up`, and common troubleshooting in `docs/containerization.md` (or README section).
- [ ] 1.7 CI note: pin base images (rust, debian, postgres) by digest for reproducibility; document digest strategy (optional follow‑up).

## 2. Validation
- [ ] 2.1 Local: fresh `docker compose up --build` reaches healthy `app` in <60s.
- [ ] 2.2 Verify schema present (connect and `SELECT 1` succeeds; `/readyz` returns 200).
- [ ] 2.3 Security: runtime runs as non‑root; no secrets baked into layers; envs passed by compose.
- [ ] 2.4 Confirm container UID/GID = 10001/10001 with `docker exec id -u`/`id -g`.

## 3. Future (follow‑ups, not blocking)
- [ ] 3.1 Add CI job to build image and run containerized smoke tests.
- [ ] 3.2 Consider MUSL static and distroless runtime to reduce image size.
- [ ] 3.3 Publish image to registry with tags (`git sha`, `v{pkg}`).
