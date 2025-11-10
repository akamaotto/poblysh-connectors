# Design: Next.js Demo Mock UX Implementation

## Overview

This design document explains the architectural approach for transforming the existing `examples/nextjs-demo` into a comprehensive mock UX sandbox that demonstrates the Poblysh Connectors integration story.

## Architecture Decisions

### 1. Mock-First Architecture

**Decision:** Implement pure mock UX (Mode A) with no real external API calls.

**Rationale:**
- Eliminates setup friction for developers and designers
- Provides deterministic, reliable demo experience
- Removes security concerns around credentials and tokens
- Focuses on teaching concepts rather than implementation complexity

**Trade-offs:**
- Demo behavior may diverge from real API over time
- Requires clear documentation mapping mock → real behavior
- Future real integration (Mode B) will need separate implementation

### 2. Client-Side State Management

**Decision:** Use React Context + hooks for state management instead of external libraries.

**Rationale:**
- Keeps implementation lightweight and easy to understand
- Matches patterns used in many Next.js applications
- Serves as better reference for real implementations
- Reduces bundle size and complexity

**Implementation:**
```typescript
// lib/demo/state.ts
export interface DemoState {
  user: DemoUser | null;
  tenant: DemoTenant | null;
  connections: DemoConnection[];
  signals: DemoSignal[];
  groundedSignals: DemoGroundedSignal[];
}

export const DemoProvider: React.FC<{children: React.ReactNode}> = ({children}) => {
  // Context implementation
};
```

### 3. Deterministic Mock Data Generation

**Decision:** Use seeded generators to create consistent mock data across sessions.

**Rationale:**
- Demo feels stable and predictable
- Easier to test and debug
- Users can rely on consistent experience
- Supports reproducible examples in documentation

**Implementation:**
```typescript
// lib/demo/mockData.ts
export function generateSignalsForConnection(connection: DemoConnection): DemoSignal[] {
  const seed = `${connection.tenantId}-${connection.providerSlug}`;
  // Use seed to generate deterministic mock data
}
```

### 4. Educational Annotations Strategy

**Decision:** Embed inline hints that map mock behavior to real Connectors concepts.

**Rationale:**
- Helps developers understand production equivalents
- Bridges gap between demo and real implementation
- Serves as living documentation
- Reduces confusion about what's mocked vs real

**Implementation:**
- Info icons with tooltips on key UI elements
- Inline explanations on each major screen
- Code comments mapping mock functions to real API endpoints
- References to relevant OpenSpec changes and API documentation

## Component Architecture

### Layout Structure
```
app/
├── layout.tsx              # Global layout + DemoProvider
├── page.tsx               # Landing + mock login
├── tenant/page.tsx        # Tenant creation + mapping
├── integrations/page.tsx  # Connector management
├── signals/page.tsx       # Signals list + filters
└── signals/[id]/page.tsx  # Signal detail + grounding
```

### Component Organization
```
components/
├── ui/                    # shadcn/ui components (existing)
├── demo/
│   ├── DemoNavbar.tsx     # Navigation + demo badge
│   ├── ProviderTile.tsx   # Integration cards
│   ├── SignalList.tsx     # Signals table/list
│   ├── SignalDetail.tsx   # Signal detail view
│   ├── GroundedSignal.tsx # Grounding results
│   └── MockBadge.tsx      # "Mock Demo" indicators
└── layout/
    ├── ProgressBar.tsx    # Flow progress indicator
    └── InfoHint.tsx       # Educational tooltips
```

### Domain Layer Organization
```
lib/demo/
├── types.ts               # Core demo types
├── state.ts               # React Context + hooks
├── mockData.ts            # Deterministic generators
├── id.ts                  # ID generation utilities
└── constants.ts           # Mock configuration
```

## Mock Domain Model

### Core Types
The mock domain model closely mirrors real Connectors concepts but is simplified for demo purposes:

```typescript
interface DemoUser {
  id: string;
  email: string;
}

interface DemoTenant {
  id: string;              // Poblysh tenant ID
  name: string;
  connectorsTenantId: string;  // X-Tenant-Id equivalent
  createdAt: string;
}

interface DemoConnection {
  id: string;
  tenantId: string;
  providerSlug: 'github' | 'zoho-cliq';
  displayName: string;
  status: 'disconnected' | 'connected';
  createdAt: string;
}

interface DemoSignal {
  id: string;
  tenantId: string;
  providerSlug: string;
  connectionId: string;
  kind: string;            // e.g., "pull_request_opened"
  title: string;
  summary: string;
  occurredAt: string;
  metadata: Record<string, unknown>;
}

interface DemoGroundedSignal {
  id: string;
  sourceSignalId: string;
  tenantId: string;
  score: number;           // 0-100 (inclusive)
  dimensions: Array<{label: string; score: number}>; // each score: 0-100 (inclusive)
  evidence: DemoEvidenceItem[];
  createdAt: string;
}
```

### Mock Data Generation Strategy

**Deterministic Seeding:**
- Use `tenantId + providerSlug` as base seed
- Generate consistent sets of signals per connection
- Create plausible cross-connector evidence for grounding

**Content Realism:**
- GitHub signals: PRs, issues, commits, releases
- Zoho Cliq signals: chat messages, thread activity
- Evidence: References to related activities across providers

**Data Relationships:**
- Signals belong to tenant + connection
- Grounded signals reference source signals
- Evidence spans multiple connected providers

## User Flow Implementation

### 1. Mock Login Flow
- Simple email input with "Continue" button
- Creates `DemoUser` in client state
- Immediate navigation to tenant setup
- Inline explanation: "In production, this would be real authentication"

### 2. Tenant Creation Flow
- Company name form when no tenant exists
- Generates paired IDs: `tenantId` + `connectorsTenantId`
- Visual display of mapping with explanations
- Clear annotation about `X-Tenant-Id: <connectorsTenantId>` header usage

### 3. Integration Management Flow
- Provider tiles showing connection status
- Mock OAuth consent modal (visual only)
- Connection state management
- "Scan" triggers signal generation

### 4. Signal Discovery Flow
- List view with provider filters
- Mock scan loading state
- Deterministic signal generation
- Navigation to detail views

### 5. Signal Grounding Flow
- Signal detail display
- "Ground this signal" action
- Score calculation with evidence aggregation
- Cross-connector evidence visualization

## Technical Implementation Details

### State Management Pattern
```typescript
// Context-based state with actions
const [state, dispatch] = useDemoReducer();

// Hooks for convenient access
const user = useDemoUser();
const tenant = useDemoTenant();
const connections = useDemoConnections();
```

### Navigation & Routing
- App Router with file-based routing
- Protected routes using client-side checks
- Progress indicator showing flow completion
- Graceful handling of lost state (page refresh)

### Mock API Simulation
- Functions that mimic API call patterns
- Loading states for realistic UX
- Error handling for educational purposes
- Consistent naming with real endpoints

### Responsive Design
- Mobile-friendly layouts using Tailwind
- shadcn/ui components for consistency
- Progressive disclosure of information
- Accessible navigation and form controls

## Educational Features

### Inline Annotations
- **Info hints**: Small icons with tooltips explaining concepts
- **Code comments**: Mapping mock functions to real API calls
- **Screen-level explanations**: What would happen in production

### Reference Implementation Aspects
- Clean component structure for real-world patterns
- Type safety with TypeScript interfaces
- Separation of concerns (UI, state, domain logic)
- Performance considerations (React optimization patterns)

### Documentation Integration
- Links to relevant OpenSpec changes
- References to integration guide
- API documentation pointers
- Architecture explanations

## Future Considerations

### Mode B (Real Integration) Path
The mock implementation is designed to make future real integration straightforward:

- Mock types can be replaced with real API types
- State management pattern can be extended with real data fetching
- Component structure can be preserved with real data integration
- Educational annotations can remain for context

### Extensibility
- Easy to add new mock providers
- Configurable mock data complexity
- Pluggable state persistence (localStorage optional)
- Component library can be extended

This design ensures the Next.js demo serves as both an effective educational tool and a solid reference implementation for future real integrations.