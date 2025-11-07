## 1. Data & Storage
- [x] 1.1 Add `grounded_signals` table (id, signal_id, score breakdown, evidence blob, status).
- [x] 1.2 Extend repositories to upsert/query grounded signals with pagination.

## 2. Engine Implementation
- [x] 2.1 Create `weak_signal_engine` module that consumes normalized Signals from the queue/DB tail.
- [x] 2.2 Implement keyword clustering + scoring model (per `plan/signals.md` six-dimension formula).
- [x] 2.3 Promote candidates to grounded signals when thresholds met; attach evidence + recommended next steps.
- [x] 2.4 Send notifications (e.g., webhook or log) when a grounded signal is created.

## 3. API & Observability
- [x] 3.1 Expose `GET /grounded-signals` (or extend `/signals`) to list grounded signals with filters.
- [x] 3.2 Add telemetry/tracing for scoring steps and decision thresholds.
- [x] 3.3 Update docs/specs to describe the keyword → signal → grounded signal → idea pipeline and how teams consume recommendations.
