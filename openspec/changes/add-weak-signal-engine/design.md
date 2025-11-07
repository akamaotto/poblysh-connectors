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
- `signal_id UUID FK -> signals`
- `score_relevance FLOAT`
- `score_novelty FLOAT`
- `score_timeliness FLOAT`
- `score_impact FLOAT`
- `score_alignment FLOAT`
- `score_credibility FLOAT`
- `total_score FLOAT`
- `status TEXT` (draft, recommended, actioned)
- `evidence JSONB`
- `recommendation TEXT`
- timestamps

### Engine Flow
1. Background worker fetches new Signals (tail on `signals` table or message queue).
2. Builds keyword vectors using existing embeddings (TF-IDF or BERT).
3. Clusters by tenant/time window if needed.
4. Scores each candidate using weights from plan (0.2R + 0.15N + ...).
5. If `total_score >= threshold` (configurable), create grounded signal and call notifier.

### Configuration
- `POBLYSH_WEAK_SIGNAL_THRESHOLD` default 0.7.
- Optional per-tenant overrides stored in config table.
- `POBLYSH_WEAK_SIGNAL_NOTIFY_WEBHOOK` (optional) to send PR nudges.

### Notifications
- For now, log/tracing is acceptable; if webhook configured, POST payload with grounded signal summary and recommended action.

### API Considerations
- Extend `/signals` with `type=grounded` filter or add `/grounded-signals`.
- Response includes score breakdown, evidence summary, recommendation.

## Validation Strategy
1. Unit tests for scoring functions with deterministic inputs.
2. Integration test: insert sample signals, run engine, ensure grounded signal + notification triggered when threshold met.
3. Database migration tests to ensure referential integrity.
