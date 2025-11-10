# Poblysh Connectors Demo (Next.js)

**🎭 DUAL MODE DEMO** - This demo supports both mock data (Mode A) and real API integration (Mode B) for testing with actual Connectors services.

## Overview

This demo showcases the complete end-to-end flow of integrating with Poblysh Connectors with **dual mode support**:

- **Mode A (Mock)**: Uses locally generated mock data, no API calls required
- **Mode B (Real)**: Connects to actual Connectors API service

The demo flow includes:

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
- Bun (recommended) or npm/yarn/pnpm

### Installation

```bash
# Install dependencies (Bun recommended)
bun install
# or
npm install
# or
yarn install
# or
pnpm install
```

### Configuration

The demo supports two modes via environment variables:

#### Mode A: Mock Mode (Default)
```bash
# Copy the mock example configuration
cp .env.example.mock .env.local

# Start development server
bun dev
```

#### Mode B: Real API Mode
```bash
# Copy the real mode example configuration
cp .env.example.real .env.local

# Edit .env.local with your actual API endpoint
# CONNECTORS_API_BASE_URL=https://your-connectors-api.example.com

# Start development server
bun dev
```

### Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `NEXT_PUBLIC_DEMO_MODE` | No | `mock` | Demo mode: `mock` or `real` |
| `CONNECTORS_API_BASE_URL` | Only for real mode | - | HTTPS URL to your Connectors API service |
| `CONNECTORS_API_TOKEN` | Optional | - | API authentication token (if required) |
| `CONNECTORS_TENANT_ID` | Optional | - | Tenant ID for multi-tenant deployments |

Open [http://localhost:3000](http://localhost:3000) to start the demo. The current mode and configuration status will be displayed at the top of the page.

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
├── types.ts               # Demo domain types + mode configuration
├── state.ts               # React Context + hooks + runtime config
├── mockData.ts            # Deterministic generators
├── constants.ts           # Mock configuration
├── demoConfig.ts          # Environment variable validation
└── apiRouter.ts           # API abstraction layer (mock/real)
```

## Related Documentation

- **Connectors API** - See main project README
- **Integration Guide** - `docs/integration-guide.md`
- **OpenSpec Changes** - `openspec/changes/`
- **Planning Docs** - `plan/nextjs-demo/`

## Mode Configuration

### Mode A: Mock Mode (Default)
- ✅ No external dependencies
- ✅ Works offline
- ✅ Instant setup
- 📊 Uses deterministic mock data
- 🎭 Educational annotations throughout

### Mode B: Real API Mode
- 🌐 Connects to actual Connectors API
- 🔧 Requires API endpoint configuration
- 📡 Makes real HTTP requests
- 🔐 Uses real authentication (if configured)
- 📈 Shows real data and performance

### Mode Indicator

The demo displays a mode indicator at the top of the page showing:
- Current mode (Mock/Real)
- Configuration status (✅ Valid/⚠️ Issues)
- API endpoint (in real mode)
- Any warnings or errors

### Switching Between Modes

Simply update the `NEXT_PUBLIC_DEMO_MODE` environment variable and restart the development server:

```bash
# Switch to mock mode
echo "NEXT_PUBLIC_DEMO_MODE=mock" > .env.local

# Switch to real mode
echo "NEXT_PUBLIC_DEMO_MODE=real" > .env.local
echo "CONNECTORS_API_BASE_URL=https://your-api.example.com" >> .env.local

# Restart server
bun dev
```

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
