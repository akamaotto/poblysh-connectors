## Context
Add a GitHub connector supporting OAuth web app flow, webhook event ingestion (signed), and REST backfill for issues and pull requests. MVP focuses on normalized Signals for select events and incremental sync based on `updated_at`.

Constraints and notes:
- OAuth App (not GitHub App) for MVP. Tokens are user-scoped.
- Manual webhook configuration is expected; public path variant `POST /webhooks/github/{tenant}` is used when signature verification is enabled (separate change).
- Backfill scope: issues and pull requests updated since `cursor.since` with pagination.
- Rate limits: REST calls respect `X-RateLimit-Remaining/Reset` and map 429s to `RateLimited`.

## Goals / Non-Goals
- Goals: OAuth authorize/callback, token refresh, HMAC verification, map issues/PR events to Signals, incremental backfill with cursors.
- Non-Goals: Automatic webhook provisioning; GraphQL; full event coverage; cross-tenant dedupe.

## Decisions
- Scopes: request `repo`, `read:org` for MVP. If private repos aren’t needed for a tenant, scopes may be reduced in future changes.
- Connection identity: store authenticated user id/login in `connections.metadata.user` and mark the latest GitHub connection for a tenant as `primary=true` unless specified otherwise. Webhooks for a tenant map to the primary connection.
- Cursor model: store `{ "since": RFC3339 }` and advance using `updated_at` of the last processed item; when equal timestamps occur, use stable pagination and `has_more`.
- Webhook mapping: transform `issues` and `pull_request` events to `issue_*` and `pr_*` Signal kinds with consistent payload shapes.
- Webhook event ordering: GitHub webhooks may deliver events out of order or with duplicates; implement deduplication based on `occurred_at` timestamp and event ID to ensure signal processing order matches delivery sequence.
- Rate limits: on 429 compute backoff from headers; otherwise use central policy with jitter.

Alternatives considered:
- GitHub App instead of OAuth App (deferred; simplifies org/repo scoping and webhooks, but adds installation flow complexity).
- GraphQL for richer queries (deferred; REST suffices for MVP).

## Risks / Trade-offs
- OAuth App tokens are user-scoped; some org/repo data may be inaccessible depending on permissions.
- Webhook-to-connection mapping via “primary” is simplistic; future multi-connection selection may be needed.
- REST backfill may miss historical data outside the initial `since` window; mitigate by configurable bootstrap horizon.

## Migration Plan
1) Introduce config variables and secret loading (client id/secret, webhook secret).
2) Register `github` in registry with metadata and connector implementation behind feature flag if needed.
3) Roll out webhook ingestion in operator-protected mode; enable public signed variant once verification change is active.
4) Validate backfill on sandbox tenants; monitor rate limits and adjust concurrency.
5) Document manual webhook setup for tenants.

## Open Questions
- Should OAuth start support optional scoping hints (org/repo) for future multi-scope connections?
- Do we need GraphQL for efficient PR review events beyond REST limits in MVP?
- How should we select among multiple GitHub connections per tenant (beyond “primary”)?

