# Change Proposals Backlog (Chronological)

Each item will be created as an OpenSpec change under `openspec/changes/<change-id>/` with proposal, tasks, and deltas. Scope favors tiny, incremental changes.

1) add-project-skeleton-axum-utoipa — Boot API, hello route, Swagger.
2) add-config-and-env-loading — `POBLYSH_*` vars, local profiles, validation.
3) add-seaorm-setup-and-migrations — DB connection, Migrator, baseline schema.
4) add-provider-and-connection-entities — SeaORM entities + repo layer.
5) add-signal-and-syncjob-entities — SeaORM entities + indices.

6) add-error-model-and-problem-json — Unified API errors and mappers.
7) add-operator-bearer-auth-and-tenant-header — `Authorization` + `X-Tenant-Id` guard.
8) add-providers-endpoint — `GET /providers` returns registry metadata.
9) add-connector-trait-and-registry — Trait + in-memory registry wiring.
10) add-connections-list-endpoint — `GET /connections` (tenant scoped).


11) add-oauth-start-endpoint — `POST /connect/{provider}` returns authorize URL.
12) add-oauth-callback-endpoint — `GET /connect/{provider}/callback` token exchange.
13) add-local-token-encryption — AES-GCM with `POBLYSH_CRYPTO_KEY`.
14) add-health-and-readiness-endpoints — `GET /healthz`, `GET /readyz`.
15) add-logging-and-tracing-basics — `tracing` JSON logs with spans.

16) add-webhook-ingest-endpoint — `POST /webhooks/{provider}` base handler.
17) add-webhook-signature-verification — GitHub HMAC, Slack v2, etc.
18) add-sync-engine-scheduler — Interval triggers with jitter.
19) add-sync-executor-and-cursoring — Execute jobs, persist cursors.
20) add-jobs-endpoint — `GET /jobs` with filters + pagination.
21) add-signals-endpoint — `GET /signals` with filters + cursor pagination.
22) add-rate-limit-policy — Central backoff + retry-after semantics.
23) add-token-refresh-background — Hourly refresher + on-demand on 401.

24) add-github-connector — OAuth app install, webhook events, REST backfill.
25) add-jira-connector — OAuth, webhook filters, incremental sync.
26) add-google-drive-connector — Watch channels + poll fallback.
27) add-google-calendar-connector — Watch channels, event sync.
28) add-gmail-connector — Pub/Sub push ingest, ack + fetch.
29) add-zoho-cliq-connector — Webhook ingest, message signals.
30) add-zoho-mail-connector — Polling + dedupe window.

31) update-api-pagination-and-cursors — Stable ordering, `next_cursor`.
32) add-normalization-fixtures — Golden tests for `Signal.kind` mapping.
33) add-local-run-scripts — Makefile/justfile tasks (no Docker).
34) add-e2e-smoke-tests-local — Boot API, hit core endpoints against local DB.
35) docs-polish-and-runbooks-local — Update PRD/Tech Spec/API; local crypto rotation runbook.

36) add-dockerization-post-mvp — Containerize service and Postgres for deployment prep.

Note: Dockerization is intentionally scheduled late (post-MVP) per project preference.
