## Why
Per `plan/signals.md`, Poblysh turns raw keywords into signals, then grounded signals, then publishable ideas. Today, each connector only emits normalized Signals; there is no shared layer to detect weak signals, attach evidence, score them, or notify PR/ops teams. Gmail temporarily embedded weak-signal heuristics into the connector, but that was removed to keep connectors focused. We now need a dedicated weak-signal engine that consumes all normalized Signals (GitHub, Jira, Google Drive, Gmail, Zoho Mail, etc.) and produces grounded signals + recommendations for PR teams.

## What Changes
- Introduce a background service/module (`weak_engine`) that:
  - Listens to new Signals (via DB or async queue).
  - Clusters keywords, applies the scoring model (relevance, novelty, timeliness, impact, alignment, credibility).
  - Promotes candidates to **Grounded Signals** once thresholds are met, attaching evidence and recommended next actions.
  - Emits PR recommendations (e.g., webhook/notification) so teams can act.
- Persist Grounded Signals + telemetry (score breakdown, evidence links).
- Provide API surfaces / repositories to query grounded signals and their status.
- Update docs/specs to reflect the new pipeline stage.

## Impact
- Specs: connectors spec gains weak-signal engine requirements; potentially signals spec as well (if one exists later).
- Code: new module (likely `src/signals/weak_engine/mod.rs`), DB migrations for grounded signals table, background worker wiring.
- APIs: new `GET /grounded-signals` endpoint with filtering and pagination; notifications to PR teams.
