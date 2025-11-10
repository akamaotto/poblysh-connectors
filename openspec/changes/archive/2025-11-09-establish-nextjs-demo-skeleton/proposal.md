# Establish Next.js Demo Skeleton

**Status:** Draft
**Created:** 2025-11-09
**Author:** AI Assistant
**Change ID:** `establish-nextjs-demo-skeleton`

## Summary

Transform the existing basic Next.js project in `examples/nextjs-demo` into a comprehensive mock UX sandbox that demonstrates the end-to-end Poblysh Connectors integration story.

This change establishes:
- A fully functional mock demo showing login → tenant → connectors → signals → grounded signals flows
- Educational UI and code that maps directly to real Connectors API concepts
- Safe, self-contained demo with no external dependencies or real API calls
- Reference implementation using Next.js App Router, Tailwind CSS, and shadcn/ui

**Mode:** Pure mock UX (Mode A) - no real Connectors integration

## Why

Engineers, designers, and PMs need a concrete, interactive way to understand the Connectors integration story without the complexity of setting up real services. The current documentation explains concepts but lacks a tangible, hands-on experience that demonstrates:

1. **Tenant Mapping Concepts:** How `tenantId` and `connectorsTenantId` relate to each other
2. **Integration Flow:** What the OAuth and connection lifecycle looks like
3. **Signal Discovery:** How signals are discovered and organized across providers
4. **Multi-Connector Grounding:** How evidence from multiple sources strengthens weak signals

A mock demo removes barriers to understanding while providing a reference implementation that guides real development efforts.

## Problem Statement

Engineers, designers, and PMs need a concrete, interactive way to understand:
- How tenants map between Poblysh Core and Connectors service
- What the Connectors integration flow looks like from a user perspective
- How signals are discovered, listed, and transformed into grounded signals
- How multiple connectors (GitHub, Zoho Cliq) work together

Currently, understanding these concepts requires reading documentation or having access to real services, creating barriers to effective development and design work.

## Solution Overview

Convert the existing `examples/nextjs-demo` from a basic Next.js starter into a fully mocked Connectors integration demo that:

1. **Demonstrates Key Concepts:**
   - Mock user authentication
   - Tenant creation and mapping visualization
   - Connector management (GitHub, Zoho Cliq)
   - Signal discovery and listing
   - Signal grounding with cross-connector evidence

2. **Educational Architecture:**
   - Clear mapping between mock flows and real API concepts
   - Inline annotations explaining what would happen in production
   - Code structure that serves as reference for real implementation

3. **Safe Mock Environment:**
   - All data generated locally with deterministic patterns
   - No external API calls or OAuth flows
   - Clearly labeled as mock-only throughout

## Scope

### IN SCOPE
- Transform existing Next.js project into mock UX demo
- Implement all core flows defined in `plan/nextjs-demo/`
- Use existing Tailwind + shadcn/ui setup
- Add mock domain types and state management
- Create responsive, accessible UI components
- Include comprehensive inline documentation

### OUT OF SCOPE
- Real Connectors API integration (Mode B)
- Real OAuth or authentication
- Production deployment configuration
- Advanced features beyond core demo flows

## Design Considerations

1. **Deterministic Mock Data:** Use seeded generators so demo feels stable across reloads
2. **Clear Mapping to Real Concepts:** Every mock feature should map to a real Connectors API concept
3. **Educational Annotations:** Inline hints explain production equivalents
4. **Simple State Management:** Use React Context + hooks, avoid heavy libraries
5. **Reference-Quality Code:** Clean, well-structured code that serves as implementation guide

## Success Criteria

- Engineers can run `npm run dev` in `examples/nextjs-demo` and experience the complete mock flow
  - **Validation**: Demo runs without console errors, loads in <3 seconds, completes full flow
- Users can understand tenant mapping, connector lifecycle, and signal grounding through interaction
  - **Validation**: User testing shows 90%+ comprehension through interactive walkthrough
- Code serves as clear reference for real Next.js + Connectors integration
  - **Validation**: Code review checklist confirms patterns match production integration guidelines
- Demo is discoverable from main project documentation
  - **Validation**: Direct links exist in README.md and integration guide with setup instructions
- All functionality works without external services or credentials
  - **Validation**: Demo functions with network disabled and no .env files required

## Dependencies

- Existing `examples/nextjs-demo` Next.js project with Tailwind + shadcn/ui
- Requirements defined in `plan/nextjs-demo/prd.md`, `plan/nextjs-demo/tech-specs.md`, `plan/nextjs-demo/ui.md`
- Related OpenSpec changes: `add-connectors-integration-guide`, `add-tenant-mapping-and-signals-ux`

## Risks & Mitigations

1. **Risk:** Demo behavior diverges from real Connectors API over time
   **Mitigation:** Keep domain model minimal, document mappings clearly

2. **Risk:** Users confuse mock guarantees with real API behavior
   **Mitigation:** Prominent "Mock Demo" labeling, explicit disclaimers

3. **Risk:** Scope creep toward real integration (Mode B)
   **Mitigation:** Strict scope definition, separate future changes for real integration

## Related Changes

- References `add-connectors-integration-guide` for integration patterns
- References `add-tenant-mapping-and-signals-ux` for UX concepts
- Provides concrete example for concepts in various API specs

## Validation

- Validate demo runs locally without external dependencies
- Confirm all flows work end-to-end
- Verify code serves as clear reference implementation
- Check documentation is discoverable and helpful