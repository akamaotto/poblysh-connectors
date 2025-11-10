# Enhanced Mock Authentication & Session Flow — Design

Change ID: `add-nextjs-demo-mock-auth-and-session-flow`

Owner: Next.js Demo (Mock UX Sandbox)

Status: Draft (Design-level; implementation gated on proposal approval)

---

## 1. Context

The `examples/nextjs-demo` app is a Next.js App Router sandbox used to demonstrate Poblysh Connectors flows with a mock domain model. It must:

- Remain mock-only: no real credentials, no real OAuth, no production secrets.
- Educate: clearly show how real-world auth, sessions, and connectors would behave.
- Align: reflect the backend Connectors domain model (Provider, Connection, Tokens, Scopes, Tenant) without enforcing backend implementation constraints.
- Use Bun for all frontend commands.
- Use modern Next.js App Router patterns as a reference (route handlers, layouts, server/client components).

The previous “auth” behavior is basic. This change introduces:

- A structured mock auth/session model.
- Realistic but fake JWT-style tokens.
- Multi-provider “OAuth-style” flows.
- Session lifecycle, cross-tab sync, events, and visualization.
- Clear separation between DEMO behavior and PRODUCTION best practices.

This design file clarifies how to implement the proposal and specs in a consistent, modern, and safe way.

---

## 2. Goals and Non-Goals

### Goals

- Provide a cohesive mock authentication & session architecture for `examples/nextjs-demo`:
  - Mock login and identity.
  - Mock provider-specific consent and token issuance.
  - Session lifecycle: issue, refresh, revoke, expire.
  - Cross-tab synchronization and basic persistence.
  - Auth event logging and educational dashboards.

- Demonstrate patterns aligned with:
  - Next.js App Router conventions (14+/15+ style).
  - Connectors domain: providers, connections, scopes, tenants.
  - Modern auth practices (JWT lifecycles, session separation, error handling).

- Make the demo self-explanatory:
  - Inline educational hints.
  - Explicit warnings about mock/insecure behavior.
  - Mapping between demo flows and real Connectors integration patterns.

### Non-Goals

- Providing production-ready authentication for Poblysh Connectors.
- Implementing real OAuth/OIDC, SAML, SSO, or Auth.js/NextAuth integration.
- Handling multi-tenant production security concerns beyond mock visualization.
- Persisting sensitive data or relying on secure cookies, signed JWTs, or external stores.

---

## 3. High-Level Architecture

### 3.1 Conceptual Overview

We introduce a dedicated “Mock Auth Layer” inside the Next.js demo:

- Purely local to `examples/nextjs-demo`.
- Clearly namespaced with `Demo*` / `Mock*` terminology.
- Driven by:
  - In-memory state (for server runtime inside the dev process).
  - `localStorage` (for browser persistence & cross-tab sync).
  - JSON-structured, unsigned “JWT-like” tokens for inspection.

Key responsibilities:

1. Identity modeling:
   - Demo users (identified by email or generated ID).
   - Demo tenants (org/workspace identifiers).

2. Session modeling:
   - `DemoAuthSession`: session id, user, tenant, issued/expiry, status.
   - Associations to providers and mock tokens.

3. Token modeling:
   - `DemoAuthToken`: header/payload/signature fields for education.
   - Types: access, refresh, id.
   - Scopes: per provider, per connection.

4. Provider auth modeling:
   - `DemoProviderAuth`: provider-specific auth state & tokens.
   - Mimics Connectors `Connection` semantics.

5. Events & analytics:
   - `DemoAuthEvent`: auth-related events for timeline, insights.
   - Used only inside the demo.

All logic is encapsulated in a small internal API under `lib/demo/auth/` and consumed by:

- Route Handlers (for mock endpoints).
- Server Components (for secure-ish checks in a demo context).
- Client Components (for UX and educational overlays).

### 3.2 Filesystem (Proposed Structure)

Under `examples/nextjs-demo`:

- `lib/demo/types.ts` (extended)
  - Added authentication types: `DemoAuthSession`, `DemoAuthToken`, `DemoProviderAuth`, `DemoAuthEvent`
- `lib/demo/mockAuth.ts` (new)
  - Utilities to create/parse mock JWT-like tokens
  - Core mock session management APIs (localStorage integration)
  - Per-provider mock flows, scopes, consent configuration
  - Auth event logging and query helpers
- `lib/demo/authEducation.ts` (new)
  - Strings/tooltips/mappings explaining authentication concepts

- `app/(auth)/login/page.tsx`
  - Mock login form and entry point.

- `app/(auth)/mock-consent/[provider]/page.tsx`
  - Mock consent UIs for GitHub, Zoho, Google Workspace, etc.

- `app/(authenticated)/layout.tsx`
  - Example of a “protected” layout using mock session verification.

- `app/auth-demo/session-dashboard/page.tsx`
  - Visualization of session state, tokens, and events.

- `app/auth-demo/events/page.tsx`
  - Auth event timeline.

- `app/api/mock-auth/login/route.ts`
- `app/api/mock-auth/logout/route.ts`
- `app/api/mock-auth/refresh/route.ts`
- `app/api/mock-auth/providers/[provider]/authorize/route.ts`
- `app/api/mock-auth/providers/[provider]/callback/route.ts`
  - Route Handlers implementing the mock flow.

- Note: No middleware.ts is included to avoid confusion with production authentication patterns. All route protection will be demonstrated through client-side components and explicit educational examples.

This structure keeps concerns local, discoverable, and aligned with App Router conventions.

---

## 4. Domain Model Design

These types are conceptual; exact TS implementations live in `lib/demo/auth/types.ts`.

### 4.1 DemoUser

- `id: string`
- `email: string`
- `displayName?: string`
- `tenantId: string`
- `avatarUrl?: string`
- `roles: string[]` (e.g., `["admin"]`, `["member"]`)

Usage:
- Created on mock login (e.g., by email).
- Tied to `DemoTenant` and sessions.

### 4.2 DemoTenant

- `id: string`
- `name: string`
- `slug: string`
- `plan?: "free" | "pro" | "enterprise"`

Usage:
- Represents the “workspace” context for connectors.
- Used in educational mapping to how real Connectors are tenant-scoped.

### 4.3 DemoAuthToken

Represents a JWT-like token for educational inspection:

- `id: string`
- `type: "access" | "refresh" | "id"`
- `providerId?: string` (null for “core” auth)
- `header: { alg: string; typ: "JWT" }`
- `payload: {`
  - `sub: string` (user id)
  - `email?: string`
  - `tenantId: string`
  - `sessionId: string`
  - `scopes: string[]`
  - `iat: number`
  - `exp: number`
  - `providerId?: string`
  - `meta?: Record<string, unknown>`
  - `tokenUse?: string` (for ID vs access clarity)
  - `}`
- `signature: string` (mock, not cryptographically valid)

Constraints:

- Must be clearly marked as:
  - Unsigned / fake signature.
  - For visualization only.
- No real secret-based signing.

### 4.4 DemoAuthSession

- `id: string`
- `userId: string`
- `tenantId: string`
- `status: "active" | "expired" | "revoked"`
- `createdAt: number`
- `updatedAt: number`
- `expiresAt: number`
- `primaryTokenId: string` (points to access token)
- `refreshTokenId?: string`
- `providerAuthIds: string[]`
- `lastActivityAt?: number`
- `meta?: Record<string, unknown>`

Behavior:

- Represents the core app session.
- Drives:
  - “You are logged in as X” UI.
  - “Expire/refresh/logout” demos.
- Maintains references to provider-specific auths.

### 4.5 DemoProviderAuth

- `id: string`
- `providerId: string` (e.g., `github`, `google_workspace`, `zoho_cliq`)
- `sessionId: string`
- `scopes: string[]`
- `status: "connected" | "expired" | "revoked" | "error"`
- `accessTokenId?: string`
- `refreshTokenId?: string`
- `createdAt: number`
- `updatedAt: number`
- `meta?: {`
  - `displayName?: string`
  - `accountId?: string`
  - `orgName?: string`
  - `}`
  
Usage:

- Models per-provider connection auth.
- Educational mapping to Connectors `Connection` entity:
  - Single source of truth for provider linkage.

### 4.6 DemoAuthEvent

- `id: string`
- `sessionId?: string`
- `userId?: string`
- `tenantId?: string`
- `providerId?: string`
- `type:`
  - `"LOGIN_SUCCESS" | "LOGIN_FAILURE" | "LOGOUT" | "TOKEN_ISSUED" | "TOKEN_REFRESHED" | "TOKEN_REFRESH_FAILED" | "TOKEN_REVOKED" | "PROVIDER_CONNECTED" | "PROVIDER_DISCONNECTED" | "SECURITY_WARNING" | "SCOPE_CHANGED" | "SESSION_EXPIRED"`
- `severity: "info" | "warning" | "error" | "debug"`
- `timestamp: number`
- `details?: Record<string, unknown>`

Usage:

- Backing data for:
  - Auth timeline / analytics UI.
- Teaches:
  - What events real systems would track.

---

## 5. Behavioral Design and Flows

### 5.1 Login Flow (Core Demo Auth)

1. User visits `/login`.
2. Enters email (and optionally name/tenant).
3. Client calls `POST /api/mock-auth/login` with form data.
4. Route Handler:
   - Creates or reuses `DemoUser` + `DemoTenant`.
   - Creates `DemoAuthSession` with:
     - Active status, `expiresAt` = now + 1h (configurable).
   - Issues:
     - One `DemoAuthToken` of type `access`.
     - One `DemoAuthToken` of type `refresh` (optional, for learning).
   - Persists to:
     - In-memory store on server (for route handlers).
     - Returns a structured session payload to client.
5. Client:
   - Stores a stable `sessionId` (and maybe tokens) in `localStorage`.
   - Updates an `AuthProvider`/hook with current session.
   - Broadcasts login via `localStorage` event to other tabs.

Important:

- Demo MUST:
  - Clearly indicate this is NOT secure:
    - Tokens visible in devtools.
    - Stored in `localStorage`.
  - Provide a sidebar/tooltip explaining how real apps would:
    - Use HttpOnly cookies & server validation.
    - Use real JWT signing.

### 5.2 Logout Flow

1. Client calls `POST /api/mock-auth/logout`.
2. Route Handler:
   - Marks `DemoAuthSession.status = "revoked"`.
   - Invalidates associated tokens and provider auths.
   - Logs `LOGOUT` and `TOKEN_REVOKED` events.
3. Client:
   - Removes session/tokens from `localStorage`.
   - Notifies other tabs via storage event.
   - Redirects to `/login` or landing.

### 5.3 Token Expiry & Refresh Simulation

- Background behavior (client-side timer or on-demand check):
  - If `accessToken.exp` is close (e.g., <5m), simulate:
    - Auto-refresh via `POST /api/mock-auth/refresh`.
- Refresh route:
  - If session active and refresh token “valid”:
    - Issue new access token, log `TOKEN_REFRESHED`.
  - With configurable small failure rate:
    - Simulate `TOKEN_REFRESH_FAILED`, require login.

UI:

- Session dashboard shows:
  - Countdown to expiry.
  - Visualization of refresh operations.
  - Educational explanation of:
    - Why refresh tokens exist.
    - How rotation works in real systems.

### 5.4 Multi-Provider OAuth Simulation

For each supported provider (e.g., GitHub, Zoho, Google Workspace):

Flow (simplified):

1. From “Connect Provider” in the demo:
   - Navigate to `/auth/mock-consent/[provider]?sessionId=...`.
2. Consent page:
   - Shows provider-branded mock UI.
   - Offers list of scopes the demo is requesting.
   - Explains what each scope would allow.
3. On “Approve”:
   - Call `POST /api/mock-auth/providers/[provider]/callback` with:
     - `sessionId`
     - selected scopes.
4. Callback:
   - Validates associated session.
   - Creates `DemoProviderAuth` with `status="connected"`.
   - Issues provider-specific `DemoAuthToken` (access/refresh).
   - Logs `PROVIDER_CONNECTED`, `TOKEN_ISSUED`, etc.

Constraints:

- Each provider connection:
  - Is independent: failure or expiry does not break others.
- Cross-provider views:
  - A unified dashboard summarizing all connections.
  - Mapped to the Connectors “Connections” concept.

### 5.5 Cross-Tab Synchronization

Mechanism:

- Session identity (e.g., `sessionId`) stored in `localStorage`.
- All tabs subscribe to `window.addEventListener("storage", ...)`.
- On login/logout/update:
  - Write a small `{"event":"session-updated","sessionId":...}` payload.
  - Other tabs:
    - Refresh their in-memory session from `localStorage`.
    - Update UI accordingly.

Production comparison:

- UI explains that:
  - Real apps often use HttpOnly cookies + server checks + broadcast via:
    - BroadcastChannel, WebSockets, etc.
  - Our approach is intentionally transparent for learning.

---

## 6. Implementation Patterns & Constraints

### 6.1 Next.js App Router Patterns

- Use:
  - Route Handlers under `app/api/mock-auth/**` for all mock auth endpoints.
  - Server Components to read mock session state where feasible.
  - Client Components for:
    - Forms.
    - Real-time dashboards.
    - Tooltips and interactive educational content.
  - Client-side route protection examples to demonstrate concepts (not middleware-based).

### 6.2 Storage & Security

- For this demo:
  - Tokens and sessions are:
    - Stored in JS-accessible locations (e.g., `localStorage`).
    - Passed in JSON responses from route handlers.
  - This is intentional:
    - To enable visualization and understanding.
  - We must:
    - Display explicit warnings in:
      - UI components.
      - Code comments.
      - Documentation tooltips.

- DO:
  - Use simple deterministic “signature” strings (e.g., `MOCK_SIGNATURE`) to drive home that JWTs are fake.
  - Use short expiries (e.g., 1 hour) to showcase lifecycle.
  - Provide config constants in one place (e.g., `DEFAULT_TOKEN_TTL_MS`).

- DO NOT:
  - Embed real secrets.
  - Use real JWT signing or imply real security.
  - Call real third-party OAuth endpoints.

### 6.3 Alignment with Connectors Domain

Design rules:

- Every provider auth simulation:
  - Should conceptually correspond to a `Connection`:
    - Provider
    - Tenant
    - Scopes
    - Status
  - The UI should:
    - Show how “mock provider auth” → “mock connection record”.

- `DemoAuthSession`:
  - Represents the “Poblysh admin/operator” using the demo, not the end-user of a connector.

- Documentation & tooltips:
  - Include explicit mapping sections, e.g.:
    - “In production, this would correspond to Connectors API /connections endpoints.”
    - “This mock token’s scopes correspond to the scopes stored in connection metadata.”

---

## 7. Risks and Mitigations

1. Risk: Demo patterns misinterpreted as production-ready.
   - Mitigation:
     - Prominent warnings in:
       - UI.
       - Comments.
       - Docs.
     - Use `Mock`/`Demo` prefixes everywhere.

2. Risk: Over-complexity for a sandbox.
   - Mitigation:
     - Keep implementation modular and well-documented.
     - Provide a “simple path” view:
       - A single-page explanation of the architecture.
     - Avoid unnecessary dependencies.

3. Risk: Drift from Next.js best practices.
   - Mitigation:
     - Use App Router conventions consistently.
     - Avoid legacy Pages Router patterns.
     - Keep route handlers and layouts idiomatic.

4. Risk: Drift from Connectors specs.
   - Mitigation:
     - Ensure `DemoProviderAuth` and flows match the semantics of Connectors `Provider` and `Connection`:
       - Tenant-scoped.
       - Provider-scoped.
       - Scope-driven.

---

## 8. Future Considerations

These items are explicitly out of scope for the current implementation but may be considered in future changes:

- HttpOnly cookie variant for comparison (would require significant additional security modeling)
- Additional providers beyond the core three (GitHub, Google Workspace, Zoho Cliq)
- Advanced role-based access control scenarios
- Enterprise SSO simulation patterns

## 9. Scope Boundaries

The current implementation is explicitly scoped to:
- localStorage-based transparent storage for educational purposes
- React Context state management integration
- Three core providers representing different auth patterns
- Educational focus on concepts rather than production security
- Mock JWT tokens with visible structure for learning

---

## 10. Summary

This design operationalizes `add-nextjs-demo-mock-auth-and-session-flow` by:

- Defining a clear, contained mock auth/session architecture.
- Choosing idiomatic Next.js App Router patterns.
- Ensuring alignment with the Connectors domain and OpenSpec guidelines.
- Making mock vs production differences explicit and educational.

It provides enough specificity that an engineer can implement the change confidently while avoiding accidental “semi-production” auth behaviors in what is intentionally an educational sandbox.