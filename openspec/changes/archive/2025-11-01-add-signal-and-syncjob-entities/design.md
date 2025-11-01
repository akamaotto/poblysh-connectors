## Context
Signals represent normalized events across providers and are the core data consumed downstream. SyncJobs represent scheduled/webhook‑triggered units of work to fetch and normalize provider data. This change introduces schemas and indices optimized for the most common reads and for efficient job picking.

## Goals / Non-Goals
- Goals: minimal, normalized schemas with tenant isolation, practical indices for common queries, and clean FKs to existing tables.
- Non-Goals: scheduler/executor, dedupe semantics, API endpoints.

## Decisions
- Signals require `connection_id` to ensure strong provenance and simplify per‑connection queries.
- JSONB `payload` holds normalized event details to avoid brittle early schema choices.
- `dedupe_key` is optional for now; later changes will define idempotency rules and unique/partial indices if needed.
- SyncJobs track `status`, `priority`, `attempts`, and timing fields to enable backoff and fair scheduling.
- Indices reflect read‑heavy patterns (signals by tenant/kind/time) and queue scans (status/priority/time).

## Index Rationale
- Signals: two primary exploration paths: by provider or by kind, both scoped to tenant and ordered by event time. Per‑connection index supports connector‑level debugging.
- SyncJobs: queue selection benefits from `(status, scheduled_at, priority DESC)` enabling ORDER BY with efficient filtering; additional indices support tenant/provider dashboards and per‑connection maintenance.

## Alternatives Considered
- Enforcing unique dedupe for signals now: postponed to avoid premature coupling; will be addressed with sync executor design.
- Separate partitioning by tenant: deferred until scale signals a need.

## Open Questions
- Do we want a GIN index on `payload` for ad‑hoc querying? Proposed: not in MVP; can be added later for specific fields.
- Should job picking incorporate `retry_after` in an index? Proposed: handle in WHERE clause initially; add composite index if needed later.

