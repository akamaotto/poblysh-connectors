# PRD: Poblysh Connectors Next.js Mock Demo UX

## 1. Summary

Build a self-contained Next.js demo application under `examples/nextjs-demo` that simulates the Poblysh ↔ Connectors integration experience using only mock data.

This demo:

- Helps engineers and designers understand the end-to-end UX and mental model.
- Mirrors real-world flows (login, tenant, connectors, signals, grounding) without:
  - Real OAuth
  - Real Connectors/OpenAPI calls
  - Real secrets or tokens

The demo serves as a UX sandbox and reference implementation for how a Next.js-based Poblysh frontend could integrate with the Connectors API in production.

Mode for this PRD: Mode A — Pure Mock UX.

Mode B (real integration via server-side proxy to Connectors) is explicitly out of scope and will be handled by future changes.

---

## 2. Goals

1. Demonstrate the full conceptual journey:
   - User “logs in”
   - Creates/selects a tenant (company)
   - Connects GitHub (mock)
   - Triggers a “scan”
   - Views a list of (mock) signals
   - Views signal details
   - Grounds a signal into a grounded signal with score/evidence
   - (Later) Adds Zoho Cliq mock integration to illustrate multi-connector flows

2. Mirror real architecture and semantics:
   - Make the demo feel like a Next.js version of Poblysh.com integrating with Connectors:
     - Tenant-scoped behavior
     - Connection lifecycle
     - Signals listing model
     - Grounded signal concept
   - But implement everything using deterministic mock data and state.

3. Be safe and self-contained:
   - No calls to production services.
   - No OAuth or third-party credentials.
   - No operator tokens, no real `X-Tenant-Id` headers.
   - Clearly labeled as a mock/demo.

4. Serve as reference-quality code:
   - Use idiomatic Next.js App Router.
   - Use Tailwind and shadcn for simple, readable components.
   - Code structured so engineers can easily map patterns to real production integration.

---

## 3. Non-Goals

- Do NOT:
  - Call the real Connectors API.
  - Implement real OAuth flows.
  - Persist real data or manage real authentication.
  - Introduce or depend on backend services beyond what Next.js provides locally.
  - Define authoritative production contracts (OpenAPI remains the source of truth).

- Mode B (realistic integration via server route handlers that proxy to Connectors) is:
  - Out of scope for this PRD.
  - Will be defined separately when Mode A is stable.

---

## 4. Target Users

1. Poblysh frontend engineers
   - Want to see how a Next.js UI could structure Connectors flows.
   - Need a safe playground to reason about UX and state transitions.

2. Poblysh backend engineers
   - Need a visual, concrete representation of how tenants, connections, and signals are experienced.
   - Want to sanity-check integration flows.

3. Designers / PMs
   - Need a click-through experience of:
     - Tenant onboarding
     - Integration setup
     - Signals discovery
     - Grounding flows
   - Without waiting for production wiring.

4. External contributors (future)
   - Need a clear, minimal reference to understand Connectors semantics.

---

## 5. Core User Flows (Mocked)

All flows are implemented fully in the Next.js app using mock state.

### 5.1. Demo Login

- Entry: `/`
- Flow:
  - User enters an email.
  - Clicks “Generate demo password & sign in” or simply “Continue”.
  - App creates a `DemoUser` in client-side state.
- Behavior:
  - No real auth.
  - This step teaches: “We have an identified user context.”

### 5.2. Tenant Creation and Mapping

- Entry: `/tenant` (or next step after login).
- Flow:
  - If no tenant exists for this user:
    - Form: “Company Name”.
    - On submit:
      - Create `DemoTenant`:
        - `id` (Poblysh tenant id style, e.g. UUID-like).
        - `connectorsTenantId` (a separate UUID-like).
  - Display:
    - Company name.
    - `tenant_id`.
    - `connectorsTenantId`.
- Purpose:
  - Teach: Poblysh Core owns tenant mapping.
  - Show visually how `tenant_id` ↔ `X-Tenant-Id`-like ID concept works.

### 5.3. Connect GitHub (Mock)

- Entry: `/integrations`
- Flow:
  - Show GitHub card with:
    - Status: “Not connected” / “Connected”.
    - “Connect GitHub” button.
  - On click:
    - Show a simple UI resembling an OAuth consent step.
    - On confirm:
      - Create a `DemoConnection`:
        - `providerSlug: "github"`
        - `status: "connected"`.
- Purpose:
  - Teach: Connectors manage provider connections.
  - Clarify: This is mock only; no real redirect or tokens.

### 5.4. Scan GitHub (Mock)

- Entry: from `/integrations` or `/signals`.
- Flow:
  - Button: “Scan GitHub for signals”.
  - On click:
    - Generate a deterministic set of `DemoSignal`s associated with:
      - Current `DemoTenant`.
      - `github` connection.
- Purpose:
  - Teach: There is often a “sync/scan” phase.
  - Mirror `GET /signals` semantics without making real calls.

### 5.5. View Signals List

- Entry: `/signals`
- Flow:
  - List all `DemoSignal`s for active tenant.
  - Columns:
    - Provider
    - Kind (e.g., `pull_request_opened`, `issue_opened`)
    - Title/summary
    - Occurred at
  - Simple filters:
    - Provider filter (GitHub vs others).
    - Optional fake “Load more” button to hint at cursor pagination.
- Purpose:
  - Teach: How signals look and are queried conceptually.

### 5.6. Signal Detail

- Entry: `/signals/[id]`
- Flow:
  - Show:
    - All fields of the `DemoSignal`.
    - Richer description/metadata (repo, URL-like strings, etc.).
  - Provide contextual copy:
    - Explaining how this maps to Connectors’ `/signals` response.
- Purpose:
  - Teach: What a single signal represents.

### 5.7. Ground Signal (Mock Grounded Signal)

- Entry: from `/signals/[id]`
- Flow:
  - Button: “Ground this signal”.
  - On click:
    - Generate a `DemoGroundedSignal`:
      - `sourceSignalId`
      - `score` (0–100)
      - `dimensions` (e.g., “Relevance”, “Impact”, “Confidence”)
      - `evidence[]`:
        - Fake messages from:
          - “Other repos”
          - “Chat threads”
          - “Issues”
    - Show:
      - Score.
      - Evidence list.
      - Short explanation text.
- Purpose:
  - Teach: How weak signals are enriched into grounded ones.
  - Anchor: `add-weak-signal-engine` and related specs conceptually.

### 5.8. Zoho Cliq Mock Integration (Later in this PRD Scope)

- Entry: `/integrations`
- Flow:
  - Add Zoho Cliq card with same connect/scan model as GitHub.
  - Generate Zoho Cliq-style `DemoSignal`s (messages, conversations).
  - Update grounding logic:
    - Use Zoho Cliq signals as cross-connector evidence.
- Purpose:
  - Teach: Multi-connector story and cross-source grounding.

---

## 6. Functional Requirements

1. The demo app MUST run entirely offline against mock data (no external API calls).
2. All state MUST be handled in a way that is:
   - Easy to understand.
   - Clearly resettable (e.g., refresh or “Reset demo”).
3. Core types MUST be defined explicitly (for discoverability).
4. UI MUST:
   - Use Tailwind + shadcn.
   - Be minimal but coherent (cards, buttons, simple forms).
5. All screens MUST include subtle guidance copy:
   - Explaining what would happen in a real integration.
   - Explicitly calling out: “This is mocked for demo.”

---

## 7. Non-Functional Requirements

- Simplicity:
  - Keep implementation small, readable, and opinionated.
- Discoverability:
  - Linked from main README or integration docs so engineers can find it.
- Consistency:
  - Align naming and flows with:
    - `docs/integration/connectors-service.md`
    - Relevant OpenSpec changes (integration guide, signals-ux).
- Safety:
  - No secrets in client code.
  - No risk of accidentally calling real services.

---

## 8. Risks & Mitigations

1. Risk: Developers confuse mock behavior with real API guarantees.
   - Mitigation:
     - Prominent “Mock Demo” labeling.
     - Inline notes on each screen pointing to real specs/OpenAPI.

2. Risk: Mock flow diverges from actual Connectors behavior over time.
   - Mitigation:
     - Keep domain model minimal.
     - Document mapping to specs.
     - Treat updates to Connectors as triggers to adjust the demo.

3. Risk: Scope creep into Mode B (real API calls).
   - Mitigation:
     - This PRD explicitly forbids real calls.
     - Real integration gets its own future PRD and change set.

---

## 9. Success Criteria

The Next.js mock demo is successful if:

- An engineer can:
  - Run `examples/nextjs-demo` locally.
  - Click through:
    - Login → Tenant → Connect GitHub → Scan → View Signals → Ground.
  - Understand, from the UI and code:
    - Tenants vs connectorsTenantId.
    - Connection lifecycle.
    - Signals and grounded signals.
- A designer/PM can:
  - Use the demo to reason about UX states and flows.
- None of the above require:
  - Setting up external services.
  - Reading Connectors implementation code first.
- The demo is stable and small enough to maintain alongside the main project.

---