## Why
We need a unified way to implement providers behind a stable interface and expose a single in-memory registry that can resolve connectors by provider name and surface provider metadata. This enables upcoming API work (e.g., `/providers`, OAuth flows, webhooks) and keeps implementations swappable while sharing common behavior.

## What Changes
- Introduce a `Connector` trait defining core lifecycle methods: `authorize`, `exchange_token`, `refresh_token`, `sync`, `handle_webhook`.
- Add an in-memory provider registry mapping `name -> { connector: Box<dyn Connector>, metadata }`.
- Define provider metadata struct: `{ name, auth_type, scopes[], webhooks, rate_limits? }`.
- Expose read-only APIs on the registry: `list_metadata()`, `get(name)`.
- Seed registry with placeholder stubs for at least one provider to prove wiring (no real API calls).
- Document behavior, errors (unknown provider), and expectations for stable ordering.

## Impact
- Affected specs: `connectors` (new capability)
- Affected code: new modules under `src/connectors/*` (trait, registry, metadata).
- Enables: `/providers` (separate change), OAuth endpoints, webhook dispatch.

