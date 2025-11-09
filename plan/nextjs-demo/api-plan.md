# Next.js Mock Demo API Plan

This document defines the internal API surface and data flows for the `examples/nextjs-demo` project.

Scope:
- Mode A only: fully mocked UX demo.
- No real calls to the Rust Connectors API.
- No real OAuth, no real secrets.
- API layer exists only to structure demo logic and mirror real-world patterns.

The goal is to:
- Provide a clean, Next.js-native API boundary between UI and demo domain model.
- Model how a “Poblysh Core” or frontend would orchestrate Connectors-style flows.
- Keep all behavior obviously non-production and safe.

## Principles

1. Mock-only:
   - All APIs in this document are internal to `examples/nextjs-demo`.
   - They operate on in-memory or local demo state.
   - They never contact external services.

2. Spec-aligned modeling:
   - Shapes and flows should resemble:
     - Connectors integration guide.
     - Signals and grounded signal concepts.
   - The APIs should feel like a plausible “Core-facing” layer, even though they are fake.

3. Client/server boundary:
   - Prefer using API routes to centralize state transitions (even if backed by in-memory state).
   - Calls mimic what the Next.js app would do to a real backend/Core.
   - Implementation can use either in-memory singleton stores or serialized state; behavior must be deterministic and inspectable.

4. Safety and clarity:
   - No operator tokens.
   - No `X-Tenant-Id` headers on the wire; use properties in mock objects instead.
   - All endpoints and responses clearly scoped as demo-only.

---

## Overall Architecture

- UI:
  - Next.js App Router (React components).
  - Calls demo APIs via `fetch` to `/api/demo/*`.

- Demo API:
  - Route handlers under `/api/demo/...`.
  - Responsible for:
    - Managing mock user sessions.
    - Creating tenants and mapping them to “Connectors tenant IDs”.
    - Managing mock connections (GitHub, Zoho Cliq).
    - Generating and serving mock signals.
    - Generating and serving mock grounded signals.

- Domain layer:
  - Shared TypeScript types and helpers:
    - `DemoUser`, `DemoTenant`, `DemoConnection`, `DemoSignal`, `DemoGroundedSignal`.
  - Provides deterministic generation of mock data.

---

## Data Models (Mock)

These are representative; exact TS definitions live in the demo lib.

- DemoUser:
  - `id: string`
  - `email: string`

- DemoTenant:
  - `id: string`                // “Poblysh tenant id”
  - `name: string`
  - `connectorsTenantId: string` // “X-Tenant-Id”-like mapping

- DemoConnection:
  - `id: string`
  - `tenantId: string`
  - `providerSlug: "github" | "zoho-cliq"`
  - `displayName: string`
  - `status: "connected" | "disconnected"`

- DemoSignal:
  - `id: string`
  - `tenantId: string`
  - `providerSlug: string`
  - `connectionId: string`
  - `kind: string`
  - `title: string`
  - `summary: string`
  - `occurredAt: string`           // ISO timestamp
  - `metadata: Record<string, any>`

- DemoGroundedSignal:
  - `id: string`
  - `sourceSignalId: string`
  - `tenantId: string`
  - `score: number`                // 0-100
  - `dimensions: {
      relevance: number
      impact: number
      confidence: number
    }`
  - `evidence: Array<{
      id: string
      providerSlug: string
      type: "message" | "issue" | "pr" | "other"
      snippet: string
      link?: string
    }>`
  - `explanation: string`

---

## API Surface

All routes are under `/api/demo/*` and are internal to the Next.js demo.

The API is intentionally simple: JSON in/out, standard HTTP verbs, no auth headers beyond a mock session token where needed.

### 1. Auth & Session

These endpoints manage a mock “logged-in user”. They exist only to structure the flow.

1. `POST /api/demo/login`

- Purpose:
  - Create or retrieve a mock `DemoUser` and session.
- Request body:
  - `{ "email": string }`
- Response:
  - `200`:
    - `{ "user": DemoUser, "sessionToken": string }`
- Behavior:
  - Generates a `DemoUser` if new.
  - Returns a `sessionToken` to be stored in a client cookie or memory.
- Notes:
  - This is NOT secure auth, only a demo of “identifying the actor”.

2. `POST /api/demo/logout`

- Purpose:
  - Clear demo session.
- Request:
  - Uses session token (e.g., from cookie or header) if implemented.
- Response:
  - `204` on success.
- Behavior:
  - Clears any server-side session mapping if such mapping exists.

(For initial version, logout is optional and may be a no-op; the API shape is here for completeness.)

---

### 2. Tenant Management & Mapping

Show how a tenant is created and mapped to a Connectors-style tenant id.

1. `POST /api/demo/tenant`

- Purpose:
  - Create a `DemoTenant` for the current user.
- Request body:
  - `{ "companyName": string }`
- Response:
  - `201`:
    - `{ "tenant": DemoTenant }`
- Behavior:
  - Requires a valid session (conceptually; enforced lightly in demo).
  - Creates:
    - `id` (Poblysh tenant id)
    - `connectorsTenantId` (distinct id to visualize mapping)
  - Associates tenant with the current user.
- UX tie-in:
  - After login, UI calls this to “set up your workspace/tenant”.

2. `GET /api/demo/tenant`

- Purpose:
  - Fetch the active tenant for the current session.
- Response:
  - `200`:
    - `{ "tenant": DemoTenant | null }`

---

### 3. Integrations & Connections (Mock)

Endpoints to manage mock connections like GitHub and Zoho Cliq.

1. `GET /api/demo/integrations`

- Purpose:
  - List available providers and current connection status.
- Response:
  - `200`:
    - `{ "providers": Array<{
          providerSlug: string
          name: string
          description: string
          connected: boolean
       }> }`
- Behavior:
  - Does NOT call real `/providers`.
  - Hard-coded providers (e.g., GitHub, Zoho Cliq).

2. `POST /api/demo/connect/github`

- Purpose:
  - Mock “Connect GitHub” action.
- Request body:
  - `{}` (no parameters needed)
- Response:
  - `200`:
    - `{ "connection": DemoConnection }`
- Behavior:
  - Requires active tenant.
  - Creates/updates `DemoConnection` with `providerSlug = "github"` and `status = "connected"`.
  - No real OAuth, no redirects.

3. `POST /api/demo/connect/zoho-cliq` (later)

- Same pattern as GitHub:
  - Creates `DemoConnection` with `providerSlug = "zoho-cliq"`.

4. (Optional) `POST /api/demo/disconnect/:providerSlug`

- Purpose:
  - Toggle connection off for demo purposes.

---

### 4. Scan & Signals (Mock)

Endpoints to simulate scanning providers and listing signals.

1. `POST /api/demo/scan/github`

- Purpose:
  - Simulate a GitHub scan for the active tenant.
- Request body:
  - Optional parameters (ignored or used to tune mock generation).
- Response:
  - `202`:
    - `{ "message": "Scan started", "estimateSeconds": number }`
- Behavior:
  - For simplicity:
    - Immediately (or after a fake delay in UI) generate mock `DemoSignal`s for GitHub and store them in demo state.

2. `POST /api/demo/scan/zoho-cliq` (later)

- Same idea for Zoho Cliq.

3. `GET /api/demo/signals`

- Purpose:
  - List signals for current tenant.
- Query params (mock but spec-aligned):
  - `provider?`: filter by providerSlug.
  - `limit?`: default small (e.g., 25).
  - `cursor?`: mock cursor string.
- Response:
  - `200`:
    - `{ "signals": DemoSignal[], "nextCursor": string | null }`
- Behavior:
  - Works over the in-memory list of mock signals.
  - Cursor can be a simple index encoded as a string.

---

### 5. Signal Detail & Grounding (Mock)

Endpoints to drill into a signal and generate grounded signals.

1. `GET /api/demo/signals/:id`

- Purpose:
  - Fetch a single `DemoSignal` by id.
- Response:
  - `200`:
    - `{ "signal": DemoSignal }`
  - `404` if not found.

2. `POST /api/demo/signals/:id/ground`

- Purpose:
  - Generate a `DemoGroundedSignal` for the given signal.
- Request body:
  - `{}` (no extra parameters for the initial version)
- Response:
  - `200`:
    - `{ "groundedSignal": DemoGroundedSignal }`
- Behavior:
  - Uses deterministic mock logic:
    - Pulls in:
      - Other mock signals for same tenant.
      - Simulated “messages” from GitHub/Zoho Cliq/web.
    - Produces:
      - Score
      - Dimensions (relevance/impact/confidence)
      - Evidence list
      - Text explanation
  - No persistence beyond demo state.

3. `GET /api/demo/grounded-signals/:id` (optional)

- Purpose:
  - Retrieve a previously generated grounded signal.
- Response:
  - `200`:
    - `{ "groundedSignal": DemoGroundedSignal }`
  - `404` if not found.

---

## Error Handling (Demo)

Even as a mock, responses should be well-formed:

- Common error shapes:
  - `{ "error": { "code": string, "message": string } }`
- Example demo codes:
  - `DEMO_UNAUTHORIZED` — no mock session.
  - `DEMO_TENANT_REQUIRED` — tenant not created yet.
  - `DEMO_NOT_FOUND` — signal/grounded signal/connection missing.
- Keep it simple; this is not the production problem+json model, but should be readable and consistent.

---

## Non-Goals (Mode A)

- No real Connectors API calls.
- No real OAuth with GitHub or any provider.
- No storage of secrets or real credentials.
- No guarantee of API stability; this is internal to the demo app.
- No direct coupling to Rust backend code (besides conceptual alignment).

---

## Future: Mode B (For Reference Only)

A future track can introduce:
- A separate “proxy mode” where `/api/demo/*` route handlers call the real Connectors API.
- Real use of `Authorization: Bearer <operatorToken>` and `X-Tenant-Id`.
- This must be a separate, opt-in spec and implementation.

Mode B is explicitly out of scope for this api-plan; included here only so readers understand that Mode A is intentionally self-contained.

---