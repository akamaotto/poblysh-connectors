# Poblysh Connectors (v0.1) — Technical Specification

## 1. Architecture
- API Layer: Axum + utoipa (OpenAPI v3) for handlers and docs.
- Core Services: Auth, token storage, sync engine, webhook dispatcher.
- Connectors: One module per provider, implementing a common trait.
- Job Runner: Tokio tasks; optional Redis queue abstraction (future) behind a trait.
- Storage: Postgres via SeaORM; migrations via SeaORM Migrator.
- Infra: Local development only; no Docker; run Postgres locally; no cloud dependencies in MVP.

## 2. Module Layout (proposed)
```
src/
  main.rs
  api/
    mod.rs
    routes.rs
    middleware.rs
  connectors/
    mod.rs
    slack.rs
    github.rs
    jira.rs
    google_drive.rs
    google_calendar.rs
    gmail.rs
    zoho_cliq.rs
    zoho_mail.rs
  core/
    providers.rs     // registry + metadata
    auth.rs          // OAuth flows per provider
    tokens.rs        // encrypt/decrypt + refresh
    sync.rs          // scheduler + executor
    webhooks.rs      // unified ingest + verify
    signals.rs       // normalization + storage
  infra/
    db.rs            // pool + migrations
    kms.rs           // KMS client
    secrets.rs       // Secrets Manager client
  telemetry/
    logging.rs
    metrics.rs
  config.rs
  error.rs
```

## 3. Data Model
Rust structures (serde + SeaORM entities):
```rust
#[derive(Clone)]
pub struct Provider { id: Uuid, name: String, auth_type: AuthType, scopes: Vec<String> }

pub struct Connection {
  id: Uuid,
  tenant_id: Uuid,
  provider_id: Uuid,
  access_token: String,            // encrypted at rest
  refresh_token: Option<String>,   // encrypted at rest
  expires_at: Option<DateTime<Utc>>,
  metadata: serde_json::Value,
}

pub enum JobState { Queued, Running, Succeeded, Failed, Retried(u8) }

pub struct SyncJob {
  id: Uuid,
  connection_id: Uuid,
  job_type: String,                // full | incremental
  state: JobState,
  last_cursor: Option<String>,
  error: Option<String>,
  started_at: DateTime<Utc>,
  finished_at: Option<DateTime<Utc>>,
}

pub struct Signal {
  id: Uuid,
  source: String,                  // github, jira, gmail, etc.
  kind: String,                    // issue_closed, pr_merged, email_sent, etc.
  payload: serde_json::Value,      // raw + normalized fields
  timestamp: DateTime<Utc>,
}
```

### Database Schema (DDL sketch)
```sql
CREATE TABLE providers (
  id uuid PRIMARY KEY,
  name text UNIQUE NOT NULL,
  auth_type text NOT NULL,
  scopes text[] NOT NULL
);

CREATE TABLE connections (
  id uuid PRIMARY KEY,
  tenant_id uuid NOT NULL,
  provider_id uuid REFERENCES providers(id) ON DELETE CASCADE,
  access_token bytea NOT NULL,      -- ciphertext
  refresh_token bytea,              -- ciphertext
  expires_at timestamptz,
  metadata jsonb NOT NULL DEFAULT '{}',
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX ON connections(tenant_id);
CREATE INDEX ON connections(provider_id);

CREATE TABLE sync_jobs (
  id uuid PRIMARY KEY,
  connection_id uuid REFERENCES connections(id) ON DELETE CASCADE,
  job_type text NOT NULL,
  state text NOT NULL,
  last_cursor text,
  error text,
  started_at timestamptz NOT NULL,
  finished_at timestamptz
);
CREATE INDEX ON sync_jobs(connection_id);
CREATE INDEX ON sync_jobs(started_at);

CREATE TABLE signals (
  id uuid PRIMARY KEY,
  source text NOT NULL,
  kind text NOT NULL,
  payload jsonb NOT NULL,
  timestamp timestamptz NOT NULL,
  connection_id uuid REFERENCES connections(id) ON DELETE SET NULL,
  tenant_id uuid NOT NULL
);
CREATE INDEX ON signals(tenant_id, timestamp);
CREATE INDEX ON signals(source, kind, timestamp);
```

## 4. Security & Secrets
- Token Encryption: AES‑GCM 256. Local symmetric key supplied via env `POBLYSH_CRYPTO_KEY` (Base64-encoded 32 bytes). Store ciphertext + metadata (IV, AAD) in Postgres.
- Key Management: Local-only for MVP; manual key rotation by updating `POBLYSH_CRYPTO_KEY` and re-encrypting tokens (runbook TBD).
- Secrets: Client IDs/secrets from `.env`/environment variables for local development.
- Webhook Verification:
  - GitHub: `X-Hub-Signature-256` HMAC verification.
  - Slack: v2 signing secret and timestamp tolerance.
  - Google: channel validation with `X-Goog-` headers; Pub/Sub JWT verification for Gmail.
  - Zoho: provider‑specific shared secret verification.
- API Access: Operator endpoints gated by bearer token (static `OPERATOR_TOKEN` for MVP; OIDC later).

## 5. Connector SDK
```rust
#[async_trait::async_trait]
pub trait Connector {
  async fn authorize(&self, tenant_id: Uuid) -> url::Url;
  async fn exchange_token(&self, code: &str) -> anyhow::Result<Connection>;
  async fn refresh_token(&self, connection: &Connection) -> anyhow::Result<()>;
  async fn sync(&self, connection: &Connection, cursor: Option<String>) -> anyhow::Result<Vec<Signal>>;
  async fn handle_webhook(&self, payload: serde_json::Value) -> anyhow::Result<Vec<Signal>>;
}
```
- Registration: `providers.rs` exposes a registry mapping `name -> Box<dyn Connector>` and provider metadata (scopes, auth type, webhook support, rate limits).
- Cursoring: Connectors specify `cursor_field` semantics (e.g., `updated_at`, `sequence`, `event_id`).
- Rate Limits: Connector can return a typed error `RateLimited(retry_after)` to reschedule jobs.

## 6. API Design (Axum + utoipa)
- Routes
  - `GET /providers`
  - `POST /connect/{provider}` → start OAuth (returns redirect URL)
  - `GET /connect/{provider}/callback` → token exchange
  - `GET /connections` (filter by `tenant_id`)
  - `POST /sync/{provider}` (manual trigger)
  - `POST /webhooks/{provider}` (signature verification + dispatch)
  - `GET /jobs` (filter by tenant, provider, state)
  - `GET /signals` (filter by tenant, source, kind, time range)
- Error Model: JSON problem details `{ code, message, details?, retry_after? }`.
- Pagination: `limit` (default 50, max 200), `cursor` token for forward pagination on time/index.
- Auth: `Authorization: Bearer <token>` for operator endpoints; tenant scoping via header `X-Tenant-Id` or JWT claim.
- Docs: utoipa derive macros on DTOs; swagger served at `/docs`.

## 7. Sync Engine
- Scheduler
  - Interval triggers per provider with jitter; per‑tenant concurrency caps.
  - Webhook notifications enqueue incremental jobs immediately.
- Executor
  - Fetch connection, ensure fresh token (refresh if expiring within threshold).
  - Call `Connector::sync` with current cursor; persist new cursor on success.
  - Emit Signals; write job record; update metrics.
- Retries: Up to 3 with exponential backoff; treat rate limits separately with `retry_after` semantics.
- Idempotency: Job key derived from `(tenant_id, provider, cursor window)` for safe dedupe.

## 8. Observability
- Logging: `tracing` with `Json` formatter; include `tenant_id`, `provider`, `job_id` in spans.
- Metrics: `prometheus` or `opentelemetry` exporter; counters for jobs run, successes, failures; histograms for job duration; gauges for queue depth.
- Health: `GET /healthz` and `GET /readyz` with DB connectivity and config checks (no KMS/Secrets in MVP).

## 9. Configuration
Environment variables (prefix `POBLYSH_`):
- `DATABASE_URL` — Postgres connection string
- `POBLYSH_CRYPTO_KEY` — Base64-encoded 32-byte key for AES-GCM
- `API_BIND_ADDR` — default `0.0.0.0:8080`
- `OPERATOR_TOKEN` — bearer for operator endpoints (MVP)
- `JOB_CONCURRENCY` — per‑node job limit
- Provider‑specific: `GITHUB_APP_ID`, `GITHUB_PRIVATE_KEY`, `SLACK_SIGNING_SECRET`, etc. (from `.env` in local dev)

## 10. Deployment (Post‑MVP)
- To be defined when we are ready for deployment. Dockerization and infrastructure scripts will be introduced at that time.

## 11. Testing Strategy
- Unit: Connector adapters with mocked HTTP (e.g., `wiremock`), token crypto, webhook verification.
- Integration: Spin up Postgres in CI; run migrations; API smoke tests.
- E2E (staging): Real app credentials in sandbox tenants; validation of initial backfill and webhook flow.

## 12. Roadmap Notes
- v0.2+: SQS/Kafka queue for jobs; OIDC for operator auth; real‑time stream; admin UI; SDKs; marketplace.
