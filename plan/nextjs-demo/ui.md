# Next.js Demo UX Plan (Mock-Only)

This document defines the UI/UX for the `examples/nextjs-demo` subproject.

Goals:
- Provide a fully mock UX sandbox that demonstrates the end-to-end Connectors integration story.
- Make it easy for Poblysh devs/designers to:
  - Click through the flow.
  - Inspect the Next.js code.
  - Map each step to the real Connectors integration guide and OpenSpec changes.
- Avoid real network calls, real OAuth, or real secrets.
- Use:
  - Next.js App Router
  - Tailwind CSS
  - shadcn/ui components
- Keep the flow opinionated but simple so it feels like a real product journey.

Non-goals:
- No real authentication.
- No real Connectors/OpenAPI integration.
- No production-grade UX polish.
- No persistence guarantees beyond in-memory / client state.

---

## Global UX Principles

1. Explicitly Mock-Only
   - Show a persistent label/badge: “Mock UX Demo — No real data, no real connections”.
   - Use neutral demo content; never imply production behavior.

2. Mirror Real Architecture Concepts
   - Every major step maps to a real-world concern:
     - User identity → Poblysh user session.
     - Tenant setup → `X-Tenant-Id` mapping.
     - Connect provider → Connectors OAuth and connection lifecycle.
     - Scan → Connectors sync and `/signals`.
     - Ground → Weak signal → grounded signal model.
   - Use concise inline hints to show where real integration points would live.

3. Simple, Legible UI
   - Minimal navigation.
   - Clear steps and status indicators.
   - Use shadcn/ui primitives (Button, Card, Input, Badge, Tabs, etc.).
   - Layout optimized for desktop; mobile-friendly is a nice-to-have, not required.

4. Deterministic Behavior
   - Mock data should be deterministic where possible so reloading doesn’t feel random or confusing.
   - Allow multiple signals/evidence items so flows feel meaningful.

---

## Information Architecture

Top-level pages (App Router):

- `/` — Landing / Step 1: Sign in (mock)
- `/tenant` — Step 2: Create/select tenant and visualize mapping
- `/integrations` — Step 3: Manage mock connections (GitHub, Zoho Cliq)
- `/signals` — Step 4: View mock signals for active tenant
- `/signals/[id]` — Step 5: Signal detail and grounding demo

Optional:
- A simple top nav or progress indicator present on all pages after login.

### Navigation Behavior

- Before sign-in:
  - Only `/` accessible.
- After sign-in but before tenant:
  - Redirect `/integrations` or `/signals` attempts back to `/tenant`.
- After tenant creation:
  - All pages accessible.
- When state is missing (e.g., hard refresh on deep link):
  - Show a small “demo state lost” message with a button to restart from `/`.

---

## Page-by-Page UX

### 1. `/` — Landing & Mock Sign-In

Purpose:
- Introduce the demo.
- Create a mock user session using only an email.

Layout:
- Centered panel (shadcn `Card`):
  - Title: “Poblysh Connectors UX Demo”
  - Subtitle: “Explore the integration flow with mock data. No real OAuth. No external calls.”
  - Badge: “Mock-only”
- Form:
  - `Input` (email)
  - Primary `Button`: “Continue”
- Behavior:
  - On submit:
    - Create `DemoUser` in client state.
    - Route to `/tenant`.

Notes:
- No password field required; if present, auto-filled or clearly mocked.
- Copy should explicitly say “This is a simulated sign-in for the demo only.”

---

### 2. `/tenant` — Tenant Creation & Mapping Visualization

Purpose:
- Demonstrate tenant creation and mapping to a Connectors tenant ID (`X-Tenant-Id` concept).

Layout:
- If no tenant:
  - Card:
    - Title: “Set up your tenant”
    - Fields:
      - Company name (Input)
    - Button: “Create tenant”
- On submit:
  - Generate:
    - `tenantId` (e.g., “pbl-tenant-1234” style ID)
    - `connectorsTenantId` (different UUID-style ID)
- Show summary:
  - Two-column Card:
    - Left:
      - “Poblysh Tenant ID”
      - Value: `tenantId`
      - Short hint: “Owned by Poblysh Core.”
    - Right:
      - “Connectors Tenant ID (X-Tenant-Id)”
      - Value: `connectorsTenantId`
      - Short hint: “Sent in headers to scope Connectors calls.”
  - Inline annotation:
    - “In production, the frontend never sets X-Tenant-Id directly. Poblysh Core does.”
- CTA:
  - Button: “Continue to Integrations” → `/integrations`

Notes:
- Visually emphasize 1:1 mapping but distinct IDs to reduce future confusion.

---

### 3. `/integrations` — Mock Integrations Hub

Purpose:
- Show how a tenant sees connectable integrations and connection states.

Layout:
- Header:
  - Title: “Integrations”
  - Subtitle: “Connect mock providers for this tenant.”
  - Show small pill: “Tenant: <company>”
- Grid of integration cards (shadcn `Card`):
  - GitHub
  - Zoho Cliq (added later; see below)
- Each card:
  - Provider logo/icon (simple).
  - Short description:
    - GitHub: “Mock GitHub integration for repositories and PR activity.”
    - Zoho Cliq: “Mock Chat integration for internal conversations.”
  - Connection state:
    - If not connected: “Not connected” (Badge).
    - If connected: “Connected” (Badge).
  - Actions:
    - `Connect` button if not connected.
    - `View details` or `Disconnect` (mock) if connected.

GitHub Connect Flow (mock):
- On `Connect` click:
  - Show modal:
    - “Authorize Poblysh (Mock) to access your GitHub data?”
    - Buttons: “Cancel”, “Authorize”
  - On “Authorize”:
    - Create `DemoConnection` with:
      - `providerSlug = "github"`
      - `status = "connected"`
    - Close modal, update card state.
    - Toast: “GitHub connected (mock). In production, this would use Connectors OAuth endpoints.”
- When GitHub connected:
  - Show secondary CTA:
    - “Scan GitHub for signals” → triggers mock scan (see next page).

Zoho Cliq Connect Flow (mock, when implemented):
- Same pattern as GitHub; text explains chat-style signals.

---

### 4. `/signals` — Mock Signals List

Purpose:
- Simulate the `/signals` endpoint experience and how a product UI might visualize it.

Entry points:
- From `/integrations` after connect:
  - “Scan GitHub for signals” → generate signals → redirect to `/signals`.
- From nav:
  - “Signals” tab/link (enabled after tenant + at least one connection).

Layout:
- Header:
  - Title: “Signals”
  - Subtitle: “Mock signals for your connected integrations.”
  - Filters row:
    - Provider filter (multi-select chips: GitHub, Zoho Cliq).
    - Simple search/filter input (client-only).
- Content:
  - If no connections:
    - Empty state:
      - Icon
      - “Connect an integration to see signals.”
      - Button: “Go to Integrations”
  - If connected but no scan yet:
    - Empty state:
      - “Run a mock scan to populate signals.”
      - Button: “Scan now” (same generator).
  - If signals exist:
    - Table or list (use shadcn `Table` or Cards):
      - Columns:
        - Provider (icon + name)
        - Kind (e.g., `pull_request_opened`, `chat_message`)
        - Title / short summary
        - Occurred at
      - Each row clickable → `/signals/[id]`.

Mock Scan Behavior:
- Triggering scan:
  - Uses mock utilities to generate deterministic signals for:
    - Current tenant ID.
    - Each connected provider.
  - Show a short loading skeleton (not required but nice).
  - On completion, show toast: “Scan complete (mock signals generated).”
- No real API calls.

Notes:
- Add a tiny caption:
  - “In production, this list is populated via Connectors `GET /signals` scoped by X-Tenant-Id. Here, we generate mock data with the same shape.”

---

### 5. `/signals/[id]` — Signal Detail & Grounding Demo

Purpose:
- Demonstrate drilling into a signal and converting a weak signal into a grounded one.

Layout:
- Header:
  - Back link: “← Back to Signals”
  - Title: from signal (e.g., “PR opened on repo X”).
  - Provider badge.
- Sections:

1) Signal Summary
   - Key fields:
     - Kind
     - Provider
     - Occurred at
     - Repo / channel / entity
   - Short text: “This is a mock representation of a normalized signal stored by Connectors.”

2) Raw/Metadata
   - Collapsible panel:
     - Show JSON-ish view of `metadata` fields.
     - Emphasize shape similar to real API but simplified.

3) Ground Signal (Mock)
   - Card:
     - Title: “Ground this signal”
     - Description:
       - “Gather related activity to understand if this is noteworthy.”
     - Button:
       - “Ground this signal”
   - On click:
     - Generate `DemoGroundedSignal`:
       - Score (e.g., 78 / 100).
       - Dimensions:
         - Relevance, Impact, Recency, Support, etc.
       - Evidence list:
         - Items referencing:
           - Other mock GitHub signals.
           - (When Zoho Cliq integrated) mock chat messages.
           - Optional “web snippet” style text.
     - Display results inline:
       - Score:
         - Use a progress bar or badge with color coding.
       - Evidence:
         - List grouped by source (GitHub, Zoho Cliq, “web”).
       - Explanation:
         - 1–3 bullet points explaining why the score is high/low.

Copy guidance:
- Add a note:
  - “This grounding behavior is mocked to illustrate the concept. Real implementations use live data from Connectors and other sources.”

---

## Visual Style & Components

Use shadcn/ui + Tailwind for:

- Layout:
  - `Container` / `max-w-4xl` central layouts.
- Navigation:
  - Simple top nav once signed in:
    - “Tenant”
    - “Integrations”
    - “Signals”
- Cards:
  - For grouped content (tenant summary, integration tiles, ground signal).
- Buttons:
  - Primary: solid for main actions.
  - Secondary: outline for navigation/less critical.
- Badges:
  - For statuses (Mock-only, Connected, Not connected).
- Tables:
  - For signals list.
- Modals/Dialogs:
  - For connect confirmation; minimal copy.

Accessibility:
- Reasonable defaults:
  - Proper button semantics.
  - Label inputs.
  - Clear text for color-coded statuses.

---

## Annotations & Teaching Hooks

Throughout the UI, we should embed short, low-noise hints that link UX to architecture:

Examples:
- On `/tenant`:
  - “In real Poblysh, this mapping drives the `X-Tenant-Id` header sent from the backend to the Connectors API.”
- On `/integrations` (after connect):
  - “In production, this step corresponds to starting OAuth with `/connect/{provider}` and handling the callback.”
- On `/signals`:
  - “This list mimics `GET /signals` results. Here we keep everything local and fake.”
- On `/signals/[id]` (grounded view):
  - “Grounding uses multiple signals/evidence sources. In production, this would query real connectors and internal indexes.”

These hints should be:
- Short.
- Placed near relevant components.
- Clearly styled as “info” (muted text or small info icon + tooltip).

---

## Future: Real Integration Mode (Out of Scope for Now)

We anticipate a future “Mode B” where:

- Route handlers call the real Connectors API.
- Operator tokens are stored server-side only.
- The mock UI doubles as an integration reference.

For this UI plan:
- That mode MUST be a separate, explicitly toggled track.
- The current document is strictly about Mode A: pure mock UX.

---