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

31) add-mail-spam-filtering — Shared spam detection/filter traits for Gmail/Zoho connectors.
32) add-weak-signal-engine — Cross-connector pipeline turning normalized Signals into grounded PR opportunities.

33) update-api-pagination-and-cursors — Stable ordering, `next_cursor`.
34) add-normalization-fixtures — Golden tests for `Signal.kind` mapping.

35) add-local-run-scripts — Makefile/justfile tasks (no Docker).
36) add-e2e-smoke-tests-local — Boot API, hit core endpoints against local DB.
37) docs-polish-and-runbooks-local — Update PRD/Tech Spec/API; local crypto rotation runbook.
38) add-dockerization-post-mvp — Containerize service and Postgres for deployment prep.

Note: Dockerization is intentionally scheduled late (post-MVP) per project preference.

39) add-connectors-integration-guide — Document Poblysh ↔ Connectors service integration model, tenant mapping, and frontend/backend usage patterns.
40) add-tenant-mapping-and-signals-ux — Specify tenant ID mapping rules, connection lifecycle UX, and signals retrieval flows for Poblysh tenants.



# Change Backlog for Examples/ NextJS Demo Sub Project

1) add-nextjs-demo-skeleton — Establish `examples/nextjs-demo` as a dedicated mock UX sandbox using Next.js App Router, Tailwind, and shadcn; add a landing page clearly labeled as mock-only.

2) add-nextjs-demo-mock-domain-model — Introduce a small TypeScript domain model (`DemoUser`, `DemoTenant`, `DemoConnection`, `DemoSignal`, `DemoGroundedSignal`) and mock data utilities that mirror real Connectors concepts without any network calls.

3) add-nextjs-demo-mock-auth-and-session-flow — Implement a simple email-based “sign in” flow that creates a demo session client-side, illustrating user identity without real authentication.

4) add-nextjs-demo-tenant-creation-and-mapping — Add a tenant setup step where a company name is entered and mapped to a generated tenant id and a separate connectorsTenantId, visually demonstrating the `X-Tenant-Id` mapping concept.

5) add-nextjs-demo-github-mock-connect-flow — Build a mock “Connect GitHub” experience on an integrations page that creates a `github` connection object (no real OAuth), annotated with where real `/connect/github` and callback handling would occur.

6) add-nextjs-demo-mock-scan-and-signals-list — Add a “Scan GitHub” action that populates a mock `/signals` list for the active tenant, using realistic fields and filters to mirror the real `GET /signals` contract.

7) add-nextjs-demo-signal-detail-and-grounding-mock — Provide a signal detail view with a “Ground this signal” action that generates a mock grounded signal (score, evidence, explanation) to illustrate the weak→grounded signal concept.

8) add-nextjs-demo-zoho-cliq-mock-integration — Extend the demo with a Zoho Cliq mock connector, including connect/scan flows and cross-connector signals to show multi-provider behavior.

9) add-nextjs-demo-docs-and-spec-alignment — Document how the Next.js demo maps to the Connectors integration guide and related OpenSpec changes, emphasizing that all behavior is mock-only and intended as a learning/reference tool.
