# Next.js Demo Tech Specs (Mode A: Pure Mock UX)

This document defines the technical specifications for the `examples/nextjs-demo` subproject.

The goal of this demo is to provide a realistic, educational UX walkthrough of the Poblysh Connectors flow using a fully mocked implementation:
- No real OAuth
- No real Connectors backend calls
- No real secrets or operator tokens
- Clear mapping to how a real Poblysh.com + Connectors integration would work

This is Mode A: Pure UX Sandbox. A potential Mode B (real integration proxy) will be defined separately and MUST NOT be mixed into this implementation.

---

## 1. Objectives

1. Help developers and designers:
   - Understand the end-to-end flow:
     - Login → Tenant → Connectors → Scan → Signals → Grounded Signal
   - See how frontends might interact with a Poblysh Core-like backend pattern.
   - Explore multi-connector behavior (GitHub, Zoho Cliq) safely.

2. Keep implementation:
   - Lightweight, readable, and idiomatic Next.js (App Router).
   - Strictly mock-only: all data generated locally.
   - Strongly aligned with:
     - `docs/integration/connectors-service.md`
     - OpenSpec changes around tenant mapping, signals, and weak/grounded signals.

3. Make the code a reference:
   - Show how to structure flows, components, and types.
   - Embed hints where real Connectors calls would be made (without implementing them).

---

## 2. Scope

IN SCOPE (Mode A):

- Next.js App Router app under `examples/nextjs-demo`.
- Tailwind CSS for layout and basic styling.
- shadcn/ui components for consistent, modern UI primitives.
- Pure mock domain model and state management:
  - `DemoUser`
  - `DemoTenant`
  - `DemoConnection`
  - `DemoSignal`
  - `DemoGroundedSignal`
- UX flows:
  - Mock login.
  - Tenant creation and mapping visualization.
  - GitHub mock connect.
  - Mock scan to generate fake signals.
  - Signal list with basic filters.
  - Signal detail view.
  - Grounded signal generation with fake evidence & scoring.
  - Zoho Cliq mock integration including cross-connector grounding.
- Inline documentation and hints to map mock behavior → real architecture.

OUT OF SCOPE (Mode A):

- Real API calls to the Rust Connectors service.
- Real OAuth with GitHub, Zoho, or others.
- Managing real secrets, operator tokens, or `X-Tenant-Id` headers.
- Persisting data in a real database.
- Implementing a production authentication system.

Any real integration behavior belongs to a future Mode B spec.

---

## 3. Architecture Overview

### 3.1 Application Type

- Next.js 13+ App Router application.
- Deployed as a standalone example within the monorepo under `examples/nextjs-demo`.
- All logic is self-contained and does not modify or depend on server-side Rust components.

### 3.2 Layers

- UI Layer:
  - React Server Components (RSC) for layout and static scaffolding.
  - Client Components for interactive flows and mock state.
  - Tailwind + shadcn for styling and consistent components.

- State & Domain Layer:
  - Pure TypeScript modules under `lib/demo/`.
  - Client-side state for:
    - Current user.
    - Current tenant.
    - Connections.
    - Signals.
    - Grounded signals.
  - State is ephemeral:
    - Reset on hard reload.
    - Optionally serialized to `localStorage` for convenience (non-normative).

- No External IO Layer (Mode A):
  - No `fetch` to real Connectors.
  - No calls to external OAuth providers.
  - No server-side environment secrets required.

---

## 4. Directory & File Structure (Target)

Within `examples/nextjs-demo`:

- `app/`
  - `layout.tsx`
    - Global layout, navigation, and demo banner.
  - `page.tsx`
    - Landing (Intro + mock login).
  - `tenant/page.tsx`
    - Tenant creation + mapping visualization.
  - `integrations/page.tsx`
    - List of available mock integrations (GitHub, Zoho Cliq).
  - `signals/page.tsx`
    - List view of generated signals.
  - `signals/[id]/page.tsx`
    - Signal detail + grounding UI.

- `lib/demo/`
  - `types.ts`
    - Core demo types (see 5.1).
  - `state.ts`
    - Hooks/contexts to manage mock state on client.
  - `mockData.ts`
    - Deterministic generators for signals, connections, grounding evidence.
  - `id.ts`
    - Helpers to generate UUID-like IDs for demo.

- `components/`
  - UI building blocks using shadcn:
    - `navbar.tsx`
    - `card.tsx` (or re-exports)
    - `button.tsx`
    - `input.tsx`
    - `badge.tsx`
    - `provider-tile.tsx`
    - `signal-list.tsx`
    - `grounded-signal-view.tsx`
  - All components MUST be simple, readable, and documented.

- `README.md` (in this directory)
  - Run instructions.
  - Clear explanation of Mode A.

Names are indicative; exact naming can be tuned, but architecture should follow this intent.

---

## 5. Domain Model (Mock)

### 5.1 Types

Types below are conceptual; exact fields may be extended, but must stay close to Connectors semantics.

- `DemoUser`
  - `id: string`
  - `email: string`

- `DemoTenant`
  - `id: string` (Poblysh/Core-style tenant id)
  - `name: string`
  - `connectorsTenantId: string` (what would be used as `X-Tenant-Id`)
  - `createdAt: string`

- `DemoConnection`
  - `id: string`
  - `tenantId: string`
  - `providerSlug: 'github' | 'zoho-cliq' | string`
  - `displayName: string`
  - `status: 'disconnected' | 'connecting' | 'connected' | 'error'`
  - `createdAt: string`

- `DemoSignal`
  - `id: string`
  - `tenantId: string`
  - `providerSlug: string`
  - `connectionId: string`
  - `kind: string` (e.g. `pull_request_opened`, `chat_message`, etc.)
  - `title: string`
  - `summary: string`
  - `occurredAt: string`
  - `metadata: Record<string, unknown>` (e.g. repo, URL, author)

- `DemoGroundedSignal`
  - `id: string`
  - `sourceSignalId: string`
  - `tenantId: string`
  - `score: number` (0–100)
  - `dimensions: { label: string; score: number; }[]`
  - `evidence: DemoEvidenceItem[]`
  - `createdAt: string`

- `DemoEvidenceItem`
  - `id: string`
  - `source: 'github' | 'zoho-cliq' | 'web' | 'other'`
  - `summary: string`
  - `url?: string`
  - `occurredAt?: string`

These types are intentionally close to real concepts but MUST NOT be considered authoritative schemas.

### 5.2 Mock Generators

`mockData.ts` SHOULD provide:

- `generateTenantIds(companyName): { tenantId, connectorsTenantId }`
- `generateGithubConnection(tenantId): DemoConnection`
- `generateZohoCliqConnection(tenantId): DemoConnection`
- `generateSignalsForConnection(connection): DemoSignal[]`
  - Use deterministic seeds based on `tenantId` and `providerSlug` so the demo feels stable.
- `generateGroundedSignal(signal, state): DemoGroundedSignal`
  - Derive a repeatable score and evidence set.

---

## 6. Flows & Screens

### 6.1 Landing & Login (Mock)

- Route: `/`
- Behavior:
  - Show introduction copy:
    - What this demo is.
    - Explicit labels: “Mock-only”, “No external calls”.
  - Email input:
    - On submit:
      - Create `DemoUser` in client state.
      - Navigate to tenant step.
- Implementation:
  - Client Component with local state and context provider.
- Notes:
  - No passwords or real auth.

### 6.2 Tenant Creation & Mapping

- Route: `/tenant`
- Preconditions:
  - `DemoUser` exists; otherwise redirect to `/`.
- Behavior:
  - If tenant absent:
    - Show company name form.
    - On submit:
      - Generate `DemoTenant` with `id` and `connectorsTenantId`.
  - Always:
    - Display both IDs and brief explanation:
      - “In real Poblysh: `connectorsTenantId` is used as `X-Tenant-Id` for Connectors.”
- State:
  - Store `DemoTenant` in context keyed to current `DemoUser`.

### 6.3 Integrations (Mock Connect GitHub & Zoho Cliq)

- Route: `/integrations`
- Preconditions:
  - Tenant exists; otherwise redirect to `/tenant`.
- Behavior:
  - Show connector tiles:
    - GitHub
    - Zoho Cliq
  - Each tile:
    - Shows current status (from `DemoConnection`).
    - “Connect” button:
      - For this demo:
        - Immediately create a `DemoConnection` with `status: 'connected'`.
      - Modal may mimic OAuth consent purely visually.
- Notes:
  - Annotate in code:
    - Where `/connect/{provider}` + callbacks would exist in a real app.

### 6.4 Scan & Signals List (Mock)

- Route: `/signals`
- Entry points:
  - From `/integrations`: “Scan and View Signals” button.
- Behavior:
  - “Scan” button:
    - If no connected providers:
      - Show hint: connect one first.
    - If connected:
      - Generate `DemoSignal`s for each connected provider and active tenant.
  - List view:
    - Display signals in a table/list:
      - Provider, kind, title, occurredAt.
    - Filters:
      - Provider dropdown (GitHub / Zoho Cliq / All).
      - Optional basic search over title/summary.
- Notes:
  - UX should loosely resemble what a real signals UI could look like.

### 6.5 Signal Detail & Grounding

- Route: `/signals/[id]`
- Behavior:
  - Lookup `DemoSignal` in client state.
  - Show:
    - All fields.
    - Minimal “raw payload” style view.
  - “Ground this signal” CTA:
    - On click:
      - Generate `DemoGroundedSignal` via `generateGroundedSignal`.
      - Show:
        - Overall score.
        - Per-dimension scores.
        - Evidence items.
- Notes:
  - Evidence should include:
    - Data that looks like it came from GitHub and/or Zoho Cliq.
    - Maybe a “web snippet”-style entry.
  - Explain:
    - “This is mock grounding to illustrate the concept; production logic lives elsewhere.”

---

## 7. State Management

### 7.1 Approach

- Use a top-level client-side provider in `app/layout.tsx` or a nested provider:
  - `DemoAppProvider`:
    - Holds:
      - `currentUser`
      - `tenant`
      - `connections`
      - `signals`
      - `groundedSignals`
    - Exposes hooks:
      - `useDemoUser()`
      - `useDemoTenant()`
      - `useDemoConnections()`
      - `useDemoSignals()`
      - `useDemoGroundedSignals()`

### 7.2 Persistence

- Mode A:
  - All state MAY live purely in memory.
  - Optional:
    - Serialize minimal state to `localStorage` to avoid losing context on reload.
  - If `localStorage` is used:
    - Keep schema simple and documented.
    - Clearly label any persistence as demo-only.

---

## 8. UI & UX Guidelines

- Use Tailwind and shadcn/ui:
  - For consistency and to mirror poblysh.com design direction.
- Keep components:
  - Small, composable, and well-named.
- Provide inline hints:
  - Each major step/page should have a short explanation box:
    - Mapping what is happening to real Poblysh + Connectors behavior.
- Accessibility:
  - Basic semantics:
    - Proper headings.
    - Labels on inputs and buttons.
- Branding:
  - Include “Poblysh Connectors UX Demo” consistently.
  - Include a clearly visible “Mock Environment”/“No real data” tag.

---

## 9. Non-Functional Requirements

- Simplicity first:
  - No heavy state libraries for Mode A (React context + hooks is enough).
  - No complicated build pipeline changes.
- Isolation:
  - Does not affect the Rust service runtime.
  - Does not require the Connectors API or database to be running.
- Safety:
  - No secrets.
  - No real tokens.
  - No outbound network calls required for core behavior.

---

## 10. Future Work (Mode B Preview, Not Implemented Here)

A separate spec may introduce:

- Real Connectors integration from Next.js server route handlers.
- Loading real providers, connections, and signals.
- Using operator tokens and `X-Tenant-Id` correctly on server side.

Constraints:

- Mode B MUST:
  - Be opt-in.
  - Live behind clear configuration flags.
  - Not alter the guarantees of Mode A.

Mode B details are intentionally excluded from this tech spec to keep Mode A focused, safe, and easy to understand.

---