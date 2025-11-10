# Poblysh Connectors Demo (Next.js)

**⚠️ MOCK DEMO ONLY** - This is a demonstration sandbox that shows how the Poblysh Connectors integration works. No real API calls or authentication occur.

## Overview

This demo showcases the complete end-to-end flow of integrating with Poblysh Connectors:

1. **User Authentication** (mock)
2. **Tenant Creation & Mapping** 
3. **Connector Management** (GitHub, Zoho Cliq)
4. **Signal Discovery & Scanning**
5. **Signal Grounding** with cross-connector evidence

## What This Demonstrates

- **Tenant Mapping:** How `tenantId` (Poblysh Core) and `connectorsTenantId` (Connectors service) relate
- **Integration Flow:** What the OAuth and connection lifecycle looks like
- **Signal Discovery:** How signals are discovered and organized across providers
- **Multi-Connector Grounding:** How evidence from multiple sources strengthens weak signals

## Tech Stack

- **Next.js 16** with App Router
- **TypeScript** for type safety
- **Tailwind CSS** for styling
- **shadcn/ui** for component library
- **React Context** for state management

## Getting Started

### Prerequisites

- Node.js 18+ 
- npm, yarn, pnpm, or bun

### Installation

```bash
# Install dependencies
npm install
# or
yarn install
# or
pnpm install
# or
bun install
```

### Running the Demo

```bash
# Start development server
npm run dev
# or
yarn dev
# or
pnpm dev
# or
bun dev
```

Open [http://localhost:3000](http://localhost:3000) to start the demo.

### Demo Flow

1. **Landing Page** - Enter any email to begin (mock authentication)
2. **Tenant Setup** - Create your organization and see tenant mapping
3. **Integrations** - Connect GitHub and/or Zoho Cliq (mock OAuth)
4. **Signals** - Scan for signals and explore discovered data
5. **Signal Detail** - Ground signals to see cross-connector evidence

## Educational Features

- **Inline Annotations** - Look for ℹ️ icons to learn about real API equivalents
- **Code Comments** - See how mock functions map to real Connectors endpoints
- **Progress Tracking** - The navigation bar shows your completion progress
- **Mock Data** - Deterministic, realistic data that demonstrates key concepts

## Project Structure

```
app/
├── layout.tsx              # Global layout + DemoProvider
├── page.tsx               # Landing + mock login
├── tenant/page.tsx        # Tenant creation + mapping
├── integrations/page.tsx  # Connector management
├── signals/page.tsx       # Signals list + filters
└── signals/[id]/page.tsx  # Signal detail + grounding

components/
├── demo/                  # Demo-specific components
└── ui/                    # shadcn/ui components

lib/demo/
├── types.ts               # Mock domain types
├── state.ts               # React Context + hooks
├── mockData.ts            # Deterministic generators
└── constants.ts           # Mock configuration
```

## Related Documentation

- **Connectors API** - See main project README
- **Integration Guide** - `docs/integration-guide.md`
- **OpenSpec Changes** - `openspec/changes/`
- **Planning Docs** - `plan/nextjs-demo/`

## Mode A vs Mode B

This is **Mode A** (Mock UX Only). It demonstrates concepts without real integration. **Mode B** (Real Integration) would connect to actual Connectors APIs and require real credentials.

## Development

```bash
# Type checking
npm run type-check

# Linting  
npm run lint

# Build
npm run build
```

## Contributing

When making changes:

1. Keep it clearly labeled as a mock demo
2. Ensure educational annotations remain accurate
3. Test the complete flow end-to-end
4. Update documentation as needed

## Support

For questions about:
- **Demo functionality** - Check inline annotations and comments
- **Real Connectors integration** - See main project documentation
- **Architecture decisions** - Refer to design documents in `plan/nextjs-demo/`
