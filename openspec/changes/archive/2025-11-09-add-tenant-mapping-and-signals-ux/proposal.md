connectors/openspec/changes/add-tenant-mapping-and-signals-ux/proposal.md
# Change Proposal: add-tenant-mapping-and-signals-ux

## Summary

This change formally defines how Poblysh Core tenants map to the Poblysh Connectors service tenancy model and how frontend and backend flows should use that mapping to manage connections and retrieve signals.

Goals:

- Establish a clear, stable contract between:
  - Poblysh Core (primary product, primary tenant system)
  - Poblysh Connectors (integration + signals microservice)
- Define how `X-Tenant-Id` is derived, persisted, and used for all tenant-scoped operations.
- Specify UX and API flows for:
  - Connecting third-party providers (e.g. GitHub)
  - Listing connections
  - Surfacing signals in Poblysh UI
- Ensure all usage is consistent, debuggable, and backwards compatible with future providers and environments.

This proposal is tightly scoped to contracts and UX flows; implementation details for specific providers (e.g. GitHub connector, Jira connector) are handled in their respective changes.

## Motivation

Today:

- The Connectors service exposes:
  - Tenant-scoped endpoints that rely on `X-Tenant-Id`.
  - Operator-level authentication using a bearer token.
  - Connection, webhook, job, and signals endpoints.
- Poblysh Core already has a canonical tenant identifier but no formalized contract with Connectors.
- Frontend and backend developers need a clear mental model for:
  - How to associate a Poblysh tenant with Connectors.
  - How to initiate OAuth flows.
  - How to fetch signals in a safe, consistent way.

Without a precise spec:

- Different services or engineers might invent their own mappings.
- It is unclear whether to reuse Poblysh tenant IDs directly or maintain a separate Connectors tenant space.
- UX flows for “Connect GitHub”, “View signals”, etc. may drift or break multi-tenant isolation.

This change provides a concrete contract and UX flow so that:

- Frontend devs can integrate safely without reading all backend code.
- Backend devs have an authoritative mapping model.
- Future connectors and signals features align with the same patterns.

## Outcomes

If accepted and implemented, this change will result in:

1. A documented tenancy contract:
   - Canonical rules for `X-Tenant-Id` in the Connectors API.
   - Recommended mapping strategy between Poblysh tenant IDs and Connectors tenant IDs.
   - Clear ownership: Poblysh Core is responsible for resolving/issuing the tenant identifier used with Connectors.

2. Standardized integration flows:
   - “Connect provider” UX:
     - Poblysh Core mediates all calls to `/connect/{provider}`.
     - Frontend never handles operator tokens or constructs `X-Tenant-Id` itself.
   - “List connections” UX:
     - Consistent, tenant-scoped API usage for settings pages.
   - “View signals” UX:
     - Consistent use of filters (provider, connection, kind) tied to the same tenant mapping.

3. Reduced ambiguity for future work:
   - Provider-specific specs (e.g. `add-github-connector`) can reference this change instead of redefining tenant rules.
   - Signals-related changes can assume a stable tenant/connection model.

## Scope

In scope:

- Documenting tenant mapping semantics between Poblysh Core and Connectors.
- Defining how `X-Tenant-Id` must be used by:
  - Internal services
  - Gateway/API layer
  - Background workers that call Connectors
- Describing UX flows for:
  - Connecting a provider
  - Listing connections
  - Querying signals for a tenant
- Ensuring compatibility with existing endpoints:
  - `/providers`
  - `/connect/{provider}`
  - `/connect/{provider}/callback`
  - `/connections`
  - `/signals`
  - `/webhooks/...` (at least at the contract level)

Out of scope:

- Implementing specific provider connectors (GitHub, Jira, etc.).
- Implementing new UI components in Poblysh frontend (this change informs them).
- Changing authentication primitives (still uses operator bearer token for Connectors).
- Low-level schema migrations (unless needed by the finalized design in this change’s specs).

## Proposed Approach (High-Level)

1. Tenancy Contract:
   - Prefer using Poblysh tenant UUID directly as `X-Tenant-Id` when safe.
   - If not suitable, define:
     - A 1:1 mapping table in Poblysh Core
     - Guidance for generating and persisting `connectors_tenant_id`
   - Document that:
     - `X-Tenant-Id` is mandatory for all tenant-scoped operations.
     - All connections, jobs, webhooks, and signals are logically isolated per `X-Tenant-Id`.
     - Frontend must never invent or guess `X-Tenant-Id`; it receives a resolved value from Poblysh Core.

2. Integration Patterns:
   - Poblysh frontend interacts only with Poblysh Core.
   - Poblysh Core:
     - Handles auth for end-users.
     - Injects `Authorization: Bearer <POBLYSH_OPERATOR_TOKENS>` and `X-Tenant-Id` when calling Connectors.
   - Flows to be documented as first-class:
     - List providers → Start OAuth → Handle callback → List connections → Fetch signals.

3. Signals UX:
   - Specify how the frontend queries signals via Poblysh Core:
     - Required/optional filters (provider, connection_id, kind, occurred_after/before).
   - Ensure that examples show:
     - Which identifiers are stable (connections, providers).
     - How pagination (`next_cursor`) is consumed by the frontend via Core.

4. Documentation Artifacts:
   - Create specs and runbooks under `openspec/changes/add-tenant-mapping-and-signals-ux/` that:
     - Include concrete request/response examples.
     - Include sequence diagrams (textual) for critical flows.
     - Cross-reference existing changes (e.g., OAuth endpoints, signals endpoint) instead of duplicating logic.

## Risks & Considerations

- Risk: Using Poblysh tenant IDs directly could expose internal identifiers broadly.
  - Mitigation: Confirm they are safe to treat as stable UUIDs, or fall back to an internal mapping strategy.
- Risk: Inconsistent usage of `X-Tenant-Id` across services.
  - Mitigation: This proposal introduces a single authoritative contract and examples.
- Risk: Confusion between user auth and operator auth.
  - Mitigation: Documentation will clearly separate:
    - End-user auth (Poblysh Core concern)
    - Connectors operator token (infrastructure/backend concern)

## Next Steps

If approved:

1. Draft detailed specs in:
   - `openspec/changes/add-tenant-mapping-and-signals-ux/specs/tenant-mapping/spec.md`
   - `openspec/changes/add-tenant-mapping-and-signals-ux/specs/signals-ux/spec.md`
2. Add `tasks.md` describing:
   - Documentation updates
   - Any required API doc tweaks (e.g., clarifying `X-Tenant-Id`)
   - Examples for frontend/backend integration.
3. Run strict OpenSpec validation and adjust for consistency with existing changes.
4. Share with frontend and backend teams as the canonical reference for integrations involving tenants, connections, and signals.