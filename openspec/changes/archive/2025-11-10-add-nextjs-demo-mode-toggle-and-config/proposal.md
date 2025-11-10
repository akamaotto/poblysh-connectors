## Why
The Next.js demo currently operates only in mock mode (Mode A), but developers need a way to experiment with real Connectors API integration (Mode B) without creating a separate project. A clear configuration model will enable seamless switching between mock and real behaviors while maintaining the existing mock UX as the default.

## What Changes
- Add environment variable support for `NEXT_PUBLIC_DEMO_MODE` (default: "mock", allowed: "mock"|"real") and `CONNECTORS_API_BASE_URL` (valid HTTPS URL required for real mode)
- Create a `demoConfig` helper module with validation, error handling, and call routing based on mode
- Add configuration validation with graceful fallback to mock mode and appropriate logging
- Extend the existing Next.js demo architecture to support both mock and real API modes
- Maintain all existing mock functionality as the default behavior

## Impact
- Affected specs: `nextjs-demo`
- Affected code: `examples/nextjs-demo/` (configuration, API routing, demo utilities)
- **BREAKING**: No breaking changes - existing mock behavior remains default