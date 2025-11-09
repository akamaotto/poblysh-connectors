connectors/openspec/changes/add-connectors-integration-guide/proposal.md
# Change Proposal: add-connectors-integration-guide

## Summary

Introduce an official, implementation-aligned integration guide for the Poblysh Connectors Service that explains:

- What the Connectors Service does in the overall Poblysh architecture.
- How Poblysh Core and frontends should interact with it.
- How tenant scoping works (including use of `X-Tenant-Id`).
- How to initiate and complete OAuth flows for connectors.
- How to manage connections.
- How to query and consume signals.
- How to use the existing OpenAPI and Swagger UI effectively.

The goal is to give product engineers a concise, authoritative document that they can follow without reverse-engineering the OpenAPI, while staying tightly aligned with the current and planned behavior of the Connectors API.

## Motivation

Today, the Connectors Service exposes a well-structured OpenAPI specification with endpoints for:

- Providers catalog
- OAuth-based connection flows
- Webhook ingestion
- Signals and jobs querying
- Health and configuration

However:

- There is no single, canonical guide that explains how Poblysh.com (Core backend + web app) should integrate with this service.
- The relationship between Poblysh tenants and connectors tenants is implicit and easy to misunderstand.
- Frontend engineers need a simple, stepwise “how do I let a tenant connect GitHub and see their signals?” playbook.
- Backend engineers need clarity on which calls must be mediated by Poblysh Core, which headers are mandatory, and how to keep responsibilities clean.

Without a dedicated integration guide:

- Different teams risk inventing slightly incompatible patterns for tenant IDs and headers.
- OAuth and webhook flows become harder to implement consistently.
- The OpenAPI file becomes the only source of truth, which is too low-level for quick onboarding.

This change provides a concise, high-signal guide that sits between product docs and raw API reference.

## Goals

1. Document the Connectors Service as a standalone microservice in the Poblysh ecosystem, with a clear mental model.
2. Define how Poblysh Core should interact with Connectors:
   - Authentication model (operator token).
   - Tenant scoping via `X-Tenant-Id`.
   - Which calls are backend-only vs. safe to expose indirectly to frontend.
3. Provide an end-to-end integration journey:
   - List providers.
   - Start OAuth for a provider (e.g., GitHub).
   - Handle OAuth callback.
   - List/manage connections for a tenant.
   - Retrieve signals for a tenant/connection.
4. Offer a curated endpoint guide:
   - Focus on what Poblysh.com needs (not every internal/ops endpoint).
   - Map each endpoint to UI/UX moments (e.g. “Connect GitHub”, “View signals”).
5. Make the guide durable and spec-driven:
   - Grounded in the existing OpenAPI contract.
   - Easy to update alongside future changes (e.g., new providers, new signals flows).

## Non-Goals

- Defining the low-level implementation of each connector (GitHub, Slack, Jira, etc.).
- Replacing or duplicating the detailed OpenAPI specification.
- Solving all aspects of tenant mapping and UX (those are covered more deeply in the separate `add-tenant-mapping-and-signals-ux` change).
- Introducing new runtime behavior or breaking changes to the existing API.

This proposal is documentation and integration-contract focused.

## Scope

IN SCOPE:

- Create an integration guide document under `docs/` (exact path to be finalized in the tasks/spec) that:
  - Describes the Connectors Service role and boundaries.
  - Defines how to map Poblysh tenants to `X-Tenant-Id`.
  - Describes backend-mediated access using `POBLYSH_OPERATOR_TOKENS`.
  - Provides step-by-step flows:
    - Providers discovery.
    - OAuth start and callback handling.
    - Listing connections.
    - Fetching signals.
  - Includes example request/response snippets aligned with current OpenAPI.
  - Explains how to use Swagger UI (`/docs`) and `/openapi.json` for deeper details.

- Ensure the guide is explicitly targeted at:
  - Poblysh Core/backend engineers integrating with the Connectors Service.
  - Frontend engineers consuming a simplified Poblysh Core API that delegates to Connectors.

OUT OF SCOPE:

- Implementing or modifying API endpoints.
- Adding new authentication mechanisms.
- Changing the semantics of `X-Tenant-Id` or introducing cross-service tenancy synchronization (those belong to related changes).
- Full UX copywriting for every screen in Poblysh.com.

## Approach

1. Align with existing OpenSpec conventions:
   - Follow the OpenSpec change structure with proposal, tasks, and spec deltas.
   - Keep this change small and focused on documentation and contracts.

2. Start from current behavior:
   - Use the existing OpenAPI (`/openapi.json`) and implemented endpoints as the source of truth.
   - Reflect the current security scheme (`bearer_auth`), `X-Tenant-Id` usage, and connector flows.
   - Where implementation is partial or provider-specific behavior is evolving, document the stable integration contract, not speculative details.

3. Define a clear narrative:
   - “What is the Connectors Service?”
   - “How does Poblysh Core call it?”
   - “How does the frontend trigger flows via Poblysh Core?”
   - “How do connections and signals map back to tenants?”

4. Provide practical examples:
   - Example sequence for “Connect GitHub”:
     - Frontend → Poblysh Core → Connectors `/connect/github` → redirect → `/connect/github/callback` → store connection → show success.
   - Example sequence for “Show signals”:
     - Frontend → Poblysh Core → Connectors `/signals` with `X-Tenant-Id` and filters → render list.

5. Keep the document short, specific, and easy to maintain:
   - Avoid duplicating every field from OpenAPI.
   - Link conceptually to relevant endpoints and rely on Swagger/OpenAPI for full reference.

## Risks & Considerations

- Risk: Documentation diverges from implementation or OpenAPI.
  - Mitigation: Base all examples on the current spec and keep changes localized so future proposals can update it atomically.

- Risk: Confusion with related changes (tenant mapping, UX patterns).
  - Mitigation: Clearly scope this guide as:
    - Integration overview + core patterns.
    - Point to `add-tenant-mapping-and-signals-ux` (and other changes) for deeper tenant mapping and UX rules.

- Risk: Over-complication.
  - Mitigation: Emphasize minimal, verb-led flows and concrete examples. No speculative architecture.

## Acceptance Criteria

This change is considered complete when:

1. A new integration guide document exists in the repository under a clear, discoverable path (e.g. `docs/integration/connectors-service.md` or equivalent).
2. The guide:
   - Explains the Connectors Service role and boundaries.
   - Documents how to use `X-Tenant-Id` for tenant scoping in the context of Poblysh.
   - Clarifies that Poblysh Core mediates access using `POBLYSH_OPERATOR_TOKENS`.
   - Provides an end-to-end flow for:
     - Listing providers.
     - Starting OAuth.
     - Handling OAuth callback.
     - Listing connections.
     - Fetching signals.
3. All described flows and endpoints are consistent with the current OpenAPI specification and implemented behavior.
4. The document is understandable by:
   - A Poblysh backend engineer integrating with the Connectors Service.
   - A Poblysh frontend engineer relying on Poblysh Core to expose a simplified API.
5. The change passes OpenSpec validation (structure, references) and is linked from relevant higher-level specs or README sections where appropriate.