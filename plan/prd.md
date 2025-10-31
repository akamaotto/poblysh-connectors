# Poblysh Connectors (v0.1) — Product Requirements Document (PRD)

## 1. Summary
- Product: Poblysh Connectors (v0.1)
- Purpose: Ingest activity from selected SaaS tools and emit normalized Signals to Poblysh “Story Hunter”.
- Audience: Internal teams building PR automation, and partner teams integrating new providers.
- Value: Turns org activity (GitHub, Jira, Google, Zoho, Slack) into story‑worthy signals with minimal setup.

## 2. Problem & Goals
- Problem: Teams create a lot of traction signals across tools, but they are siloed and hard to monitor. PR teams miss timely moments.
- Goal: Provide a lightweight, reliable, and secure ingestion fabric that normalizes events into Signals for Poblysh.
- Non‑Goal: A full Zapier/IFTTT replacement or general automation engine.

## 3. Users & Personas
- Product Ops/PR Ops: Configure connections and monitor sync status.
- Engineers (internal): Add/maintain connectors and troubleshooting.
- Data/ML (internal): Consume normalized Signals pipeline.

## 4. Scope (MVP)
In scope (v0.1):
- Providers: Slack (existing), GitHub, Jira, Google Drive, Google Calendar, Gmail, Zoho Cliq, Zoho Mail.
- Capabilities: OAuth/token handling, webhook reception, polling where required, incremental sync, normalized Signal emission, job history.
- API: Axum REST API with Swagger (utoipa) for: providers, connect, connections, manual sync, webhooks, jobs, signals.
- Storage: Postgres for tokens (encrypted), connection metadata, job states, logs. Tenant isolation required.
- Infra: Localhost-focused; no Docker in MVP (run Postgres locally).

Out of scope (v0.1):
- Real‑time stream over websockets; multi‑region queues; external SDKs; admin UI; marketplace.

## 5. Success Metrics
- Time‑to‑first‑signal: < 15 minutes from first connection to first Signal.
- Data freshness: 95% of events available within 60s (webhook providers) and within SLA per poll window for polling providers.
- Reliability: < 0.1% job failures per day (auto‑retry 3x exponential backoff).
- Security: 100% tokens stored encrypted; key rotation validated in staging.
- Coverage: All MVP providers produce at least one high‑value Signal type.

## 6. Functional Requirements
- Authentication
  - Provide authorization URL per provider.
  - Handle OAuth callback and store tokens securely.
  - Refresh tokens automatically (hourly cadence; provider‑specific rules).
- Connections
  - Create/List/Delete connections, all scoped to `tenant_id`.
  - Enforce tenant isolation on every request.
- Sync Engine
  - Job scheduler triggers jobs via interval and webhooks.
  - Support full backfill on first connection and incremental sync via cursor per provider.
  - Rate limit policies per provider (sleep/reschedule strategies).
- Webhooks
  - Unified `POST /webhooks/{provider}` endpoint, with provider signature verification.
  - Fan‑out to provider handler; produce Signals.
- Signals
  - Normalize to `{id, source, kind, payload, timestamp}` with provider‑agnostic `kind` naming.
  - Store Signals and expose query/list endpoint for downstream pipeline.
- Observability
  - Structured logs; per‑job status and error context; basic metrics (jobs run, success rate, latency, rate‑limit hits).

## 7. Non‑Functional Requirements
- Security: AES‑GCM token encryption with a local symmetric key (`POBLYSH_CRYPTO_KEY`); secrets via `.env` for local development.
- Privacy: Store only required scopes and minimum PII necessary for operation.
- Performance: API p95 < 150ms for read endpoints; job execution parallelism configurable by tenant.
- Availability: Target 99.9% for API layer in production.
- Resilience: Retries with jitter; idempotent webhook handling.

## 8. Provider Coverage (MVP)
- Slack (done): Messages and reactions in configured channels.
- GitHub: Push, PR, Release via webhook; REST for backfills.
- Jira: Issue created/updated/status changes via webhook; REST for backfills.
- Google Drive: File created/edited (watch + poll fallback).
- Google Calendar: New/updated events (watch).
- Gmail: New threads; journalist correspondence (Pub/Sub push); poll fallback optional in later phase.
- Zoho Cliq: Messages, reactions, mentions via webhook.
- Zoho Mail: New inbound/outbound messages via polling.

## 9. User Stories & Acceptance Criteria
- As PR Ops, I can connect a provider and see a first Signal within SLA.
  - AC: After OAuth + webhook setup, a new event produces a Signal visible via `GET /signals`.
- As PR Ops, I can manually trigger a sync for a provider.
  - AC: `POST /sync/{provider}` enqueues a job; status visible in `GET /jobs` and results in new Signals when applicable.
- As Engineer, I can add a new provider with a standard trait.
  - AC: Implement `Connector` trait; register provider; endpoints auto‑documented in Swagger.
- As Security, I can rotate secrets without downtime (local MVP).
  - AC: Credentials from environment; crypto key rotated locally with a documented runbook.

## 10. Risks & Mitigations
- Webhook reliability (missed events): Mitigate with periodic incremental backfills and provider replay cursors.
- Rate limits: Centralized rate limit policies and backoff; job rescheduling.
- Token expiry: Background refresher and on‑demand refresh upon 401 responses.
- Schema drift: Use provider‑agnostic `Signal.kind`; store raw payload for traceability.

- Postgres (local, no Docker).
- Slack/GitHub/Jira/Google/Zoho developer apps configured per tenant/installation.

## 12. Release Plan (MVP)
- Week 1: Core API, Swagger, Postgres schema, provider registry.
- Week 2: GitHub & Jira connectors.
- Week 3: Google Drive & Calendar connectors.
- Week 4: Gmail & Zoho connectors.
- Week 5: Sync jobs + background runner, reliability polish.
- Week 6: Local dev environment polishing + runbooks + docs.

## 13. Open Questions
- External authentication for this API (operator access): API key vs OIDC/JWT?
- Multi‑tenant throttling policy: per‑tenant concurrency caps default values?
- Data retention policy for Signals: default TTL vs archive to S3?
