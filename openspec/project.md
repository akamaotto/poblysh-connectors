# Project Context

## Purpose
Poblysh Connectors (v0.1) is a lightweight, Rust-based framework for ingesting activity from collaboration and productivity tools (GitHub, Jira, Google, Zoho, Slack) and emitting normalized Signals into Poblysh’s “Story Hunter” pipeline. It is not a general-purpose Zapier clone; scope is ingestion, normalization, and reliability for a curated set of providers.

Goals:
- Provide secure OAuth flows, token storage, and webhook/polling ingestion per provider.
- Normalize events to a provider-agnostic Signal model for downstream ranking/scoring.
- Offer a small, well-documented Axum REST API with Swagger for ops and debugging.

## Tech Stack
- Language/Runtime: Rust (stable), Tokio async runtime
- Web/API: Axum; OpenAPI docs via `utoipa` + Swagger UI
- Serialization: `serde`, `serde_json`
- Database: Postgres (local for development)
- DB Access: SeaORM (+ SeaORM Migrator)
- Crypto/Security: AES-GCM (via `aes-gcm`/`ring`) with a locally provided key (`POBLYSH_CRYPTO_KEY`)
- Secrets: `.env`/environment variables for local development
- Background Jobs: Tokio tasks (MVP), trait for pluggable queue (future: Redis/SQS)
- Observability: `tracing` structured logs; optional Prometheus/OpenTelemetry exporter for local
- Packaging/Infra: Localhost only for MVP; no Docker until deployment
- Testing: `cargo test`, `wiremock` for HTTP mocking, `testcontainers` for Postgres

## Project Conventions

### Code Style
- Enforce `rustfmt` formatting; CI runs `cargo fmt -- --check`.
- Lint with `clippy` (`-D warnings` in CI for core crates).
- Error handling: `thiserror` for libraries; `anyhow` for app-level errors; return structured API errors.
- Naming:
  - Providers: snake_case identifiers (e.g., `github`, `jira`, `google_drive`).
  - Signals: `kind` uses action-first verbs (e.g., `issue_created`, `pr_merged`, `email_sent`).
  - Modules: `connectors/<provider>.rs` implement the `Connector` trait.
- DTOs derive `Serialize`, `Deserialize`, `utoipa::ToSchema` where applicable.

### Architecture Patterns
- Layered services:
  - API layer (Axum routes + middleware + Swagger)
  - Core services (auth, tokens, sync engine, webhooks, signals)
  - Connectors (one module per provider implementing a common trait)
  - Infra (DB, crypto, config)
- Connector SDK: common trait with methods for `authorize`, `exchange_token`, `refresh_token`, `sync`, `handle_webhook`.
- Sync engine: scheduler + executor with per-provider rate-limit policy and exponential backoff.
- Tenant isolation: every connection and query scoped to `tenant_id`.
- Config via environment variables (`POBLYSH_*`) suitable for localhost. Future production secret stores are out of scope for MVP.
- API error model: problem+json-like shape `{ code, message, details?, retry_after? }`.

### Testing Strategy
- Unit tests for provider adapters (HTTP mocked via `wiremock`), token crypto, webhook verification.
- Integration tests boot Postgres (via `testcontainers`), run migrations, and hit API endpoints.
- Normalization tests: golden test fixtures for Signals to avoid schema drift.
- Staging E2E: real app credentials for sandbox tenants; validate backfill + webhook flows.

### Git Workflow
- Branching: trunk-based with short-lived branches:
  - `feature/<short-desc>`, `fix/<short-desc>`, `chore/<short-desc>`
- Commits: Conventional Commits (`feat:`, `fix:`, `chore:`, `docs:`, `refactor:`, `test:`). Reference OpenSpec change IDs when relevant.
- PRs: require CI green (lint, test), at least one reviewer, and updated docs/specs when behavior changes.
- OpenSpec: for new capabilities or breaking changes, create a proposal under `openspec/changes/<change-id>/` and validate with `openspec validate --strict`.

## Domain Context
- Entities
  - Provider: metadata and scopes for an integration (GitHub, Jira, Google Drive, etc.).
  - Connection: tenant-scoped authorization (tokens + metadata), encrypted at rest.
  - SyncJob: scheduled or webhook-triggered unit of work (full/incremental) with state + cursor.
  - Signal: normalized event record with `source`, `kind`, `payload`, `timestamp` consumed by Story Hunter.
- Data Flow
  1) Operator initiates OAuth → connection saved with encrypted tokens.
  2) Webhooks or scheduler trigger `SyncJob`.
  3) Connector fetches changes (with cursor), emits Signals.
  4) Signals stored and exposed via API for downstream pipeline.
- Providers (MVP): Slack (existing), GitHub, Jira, Google Drive/Calendar/Gmail, Zoho Cliq/Mail.

## Important Constraints
- Security
  - Tokens encrypted with AES-GCM; local symmetric key provided via `POBLYSH_CRYPTO_KEY` (Base64-encoded 32 bytes).
  - Webhook signature verification (GitHub HMAC, Slack v2, Google validation, Zoho shared secret).
  - Tenant isolation enforced at API and DB query layers.
- Reliability & Performance
  - Retries (3x) with exponential backoff and jitter; rate-limit aware rescheduling.
  - Idempotent webhook handling and job dedupe by `(tenant, provider, cursor window)`.
  - Target p95 < 150ms for read endpoints; first Signal within ~15 minutes from first connection (MVP).
- Scope
  - Ingestion + normalization only; no general automation workflows in v0.1.

## External Dependencies
- Local services: Postgres database (installed locally), optional ngrok for webhook development.
- Third-party APIs: Slack, GitHub, Atlassian Jira, Google Workspace (Drive, Calendar, Gmail), Zoho (Cliq, Mail).
- Libraries/Tools: Axum, utoipa, SeaORM, serde, tracing, prometheus/otel, wiremock, testcontainers.
