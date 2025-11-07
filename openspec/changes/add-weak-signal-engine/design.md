## Context
- `plan/signals.md` defines the keyword → signal → grounded signal → idea journey with scoring across relevance, novelty, timeliness, impact, alignment, and credibility.
- Connectors currently emit normalized Signals but no shared layer turns them into grounded signals or PR recommendations.
- Gmail briefly implemented weak-signal heuristics inline, but we removed them for SRP. We now introduce a standalone engine.

## Requirements Recap
1. Consume normalized Signals (from DB or queue) regardless of provider.
2. Cluster/score them using the six-dimension formula.
3. Promote candidates to grounded signals once thresholds are reached; attach evidence/metadata.
4. Notify PR teams (internal event or webhook) with recommended next steps.
5. Persist telemetry so the editorial pipeline can audit decisions.

## Architecture
```
src/signals/
  mod.rs
  weak_engine/
    mod.rs
    scorer.rs
    notifier.rs

db/
  migrations/XXXX_add_grounded_signals.sql
```

### Data Model
`grounded_signals` table:
- `id UUID PK`
- `tenant_id UUID` (tenant owning the originating signal; required for all queries)
- `signal_id UUID FK -> signals`
- `score_relevance FLOAT`
- `score_novelty FLOAT`
- `score_timeliness FLOAT`
- `score_impact FLOAT`
- `score_alignment FLOAT`
- `score_credibility FLOAT`
- `total_score FLOAT`
- `status TEXT` (draft, recommended, actioned)
- `evidence JSONB` (MUST include traceable references to contributing signals and sources; see Evidence & Observability)
- `recommendation TEXT`
- `idempotency_key TEXT` (optional; used to prevent duplicate grounded signals for the same cluster)
- `created_at TIMESTAMP`
- `updated_at TIMESTAMP`

`tenant_signal_configs` table for per-tenant overrides:
- `tenant_id UUID PK`
- `weak_signal_threshold FLOAT DEFAULT 0.7`
- `scoring_weights JSONB DEFAULT NULL` (optional override of default weights)
- `webhook_url TEXT DEFAULT NULL`
- `created_at TIMESTAMP`
- `updated_at TIMESTAMP`

### Engine Flow
1. Background worker fetches new Signals (tail on `signals` table or message queue) scoped by tenant.
2. Similarity features:
   - When available, the engine SHOULD use embeddings from the `signals` table to build keyword/semantic vectors.
   - Embeddings MAY be generated upstream by connectors or a shared service.
   - When embeddings are not available for a signal, the engine MUST fall back to a deterministic TF-IDF (or equivalent) text vectorization of signal content so behavior does not depend on embeddings being present.
3. Clusters by tenant/time window if needed using cosine similarity (e.g., threshold 0.8) to group related weak signals.
4. Scores each candidate using the six-dimension formula: **total_score = 0.25*impact + 0.20*relevance + 0.15*novelty + 0.15*alignment + 0.15*timeliness + 0.10*credibility**.
5. Idempotent promotion:
   - For each candidate cluster, compute an `idempotency_key` (for example, a stable hash over `{tenant_id, related_signal_ids, time_window}`).
   - Before inserting, check for an existing `grounded_signals` row with the same `tenant_id` and `idempotency_key`.
   - Only create a new grounded signal when no matching row exists to avoid duplicates on retries or reprocessing.
6. If `total_score >= threshold` (configurable), create a grounded signal (including evidence and score breakdown) and call the notifier.

### Configuration
- Engine defaults are provided via `WeakSignalEngineConfig` (threshold 0.7, six-hour clustering window, cosine similarity threshold 0.8, webhook timeout 10 seconds).
- Per-tenant overrides live in `tenant_signal_configs` and MAY redefine thresholds, scoring weights, and webhook destinations without impacting other tenants.
- Sensitive configuration (e.g., webhook URLs, auth tokens) MUST be redacted from logs and responses.

### Notifications
- For now, log/tracing is acceptable; if webhook configured, POST a payload with grounded signal summary and recommended action.
- Webhook validation requirements:
  - URL must use HTTPS protocol.
  - URL validation: must be a valid HTTP/HTTPS URL with max 2048 characters.
  - Connection timeout: 10 seconds.
  - Retry policy: 3 attempts with exponential backoff (1s, 2s, 4s).
  - Payload format: JSON with `grounded_signal_id`, `tenant_id`, `total_score`, `recommendation`, `evidence_summary`, `signal_source`.
  - Authentication: Optional Bearer token header from environment variables.
  - Implementations MUST NOT log webhook bearer tokens or other sensitive authentication material and SHOULD avoid logging full webhook URLs with query parameters or secrets.

### API Considerations
- Add dedicated endpoint `/grounded-signals` for querying grounded signals.
- Query parameters:
  - `tenant_id` (required; all results MUST be scoped to this tenant)
  - `status` (optional: draft, recommended, actioned)
  - `min_score` (optional: float filter for minimum total score)
  - `limit` (default: 50, max: 200)
  - `offset` (default: 0)
- The implementation MUST enforce tenant isolation server-side and MUST NOT return grounded signals for other tenants, regardless of client-provided filters.
- Response schema:
  ```json
  {
    "data": [
      {
        "id": "uuid",
        "signal_id": "uuid",
        "tenant_id": "uuid",
        "scores": {
          "relevance": 0.8,
          "novelty": 0.6,
          "timeliness": 0.9,
          "impact": 0.7,
          "alignment": 0.8,
          "credibility": 0.75,
          "total": 0.77
        },
        "status": "recommended",
        "evidence": {
          "keywords": ["keyword1", "keyword2"],
          "sources": ["github", "jira"],
          "related_signals": ["uuid1", "uuid2"]
        },
        "recommendation": "Investigate potential partnership opportunity with Company X",
        "created_at": "2025-01-01T12:00:00Z",
        "updated_at": "2025-01-01T12:00:00Z"
      }
    ],
    "pagination": {
      "total": 125,
      "limit": 50,
      "offset": 0,
      "has_more": true
    }
  }
  ```

### Evidence & Observability
- Evidence stored in `grounded_signals.evidence` MUST:
  - Include contributing Signal IDs and their providers/source systems.
  - Include a summary of why the cluster was promoted (e.g., dominant keywords, themes, or sources).
  - Be sufficient for an operator to audit why a grounded signal exists without querying raw logs.
- Implementations SHOULD emit metrics and traces including (but not limited to):
  - Count of weak signals processed per tenant.
  - Count of grounded signals created per tenant.
  - Promotion rate (grounded / candidate).
  - Webhook notification success/failure counts and latencies.
- These observability signals MUST NOT include secrets or full sensitive payloads; instead use identifiers and high-level attributes.

## Validation Strategy
1. Unit tests for scoring functions with deterministic inputs.
2. Integration tests:
   - Insert sample signals for a tenant, run the engine, and ensure a grounded signal and notification are triggered when thresholds are met.
   - Verify per-tenant thresholds and scoring weights are applied correctly.
   - Verify that duplicate or retried processing does not create duplicate grounded signals for the same cluster (idempotency).
3. Database migration tests to ensure referential integrity and tenant scoping.
4. Security-focused tests to ensure:
   - Webhook authentication configuration and secrets are not logged.
   - `/grounded-signals` enforces strict tenant isolation.
5. Observability checks to confirm required metrics and traces are emitted for scoring, promotion, and notification flows.
