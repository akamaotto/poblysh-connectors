## Context
Introduce a Connector SDK and a simple in-memory provider registry to enable standardized provider integrations and centralized discovery. This is a cross-cutting foundation used by OAuth flows, webhooks, sync jobs, and the `/providers` endpoint.

## Goals / Non-Goals
- Goals: common trait, metadata type, registry for lookup and listing; stub wiring.
- Non-Goals: real provider implementations, network calls, persistence.

## Decisions
- Trait surface includes `authorize`, `exchange_token`, `refresh_token`, `sync`, `handle_webhook` to match roadmap and API plan.
- In-memory registry stored as a static, lazily-initialized map; read-only at runtime.
- Metadata includes `name`, `auth_type`, `scopes`, `webhooks`; can expand in future (rate limits, docs URL).

## Risks / Trade-offs
- Static registry limits dynamic loading; acceptable for MVP simplicity.
- Stub connectors may drift without tests; mitigated by unit tests around registry behavior.

## Migration Plan
1) Add modules: `connectors/trait.rs`, `connectors/metadata.rs`, `connectors/registry.rs`.
2) Implement stub provider and register.
3) Add tests; ensure stable sorting for metadata listing.
4) Wire registry into app state when needed by API changes.

## Open Questions
- Should registry support hot-reload in dev? (Out of scope for MVP.)
- Which providers to seed as stubs initially? (Start with `example` only.)

