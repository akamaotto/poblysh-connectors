# Implementation Tasks: Establish Next.js Demo Skeleton

## Overview
Ordered list of small, verifiable work items to transform the existing `examples/nextjs-demo` into a comprehensive mock UX sandbox.

## Tasks

### Phase 1: Foundation Setup

1. ✅ **Update demo README with mock UX context**
   - [x] Replace generic Next.js README with demo-specific documentation
   - [x] Add clear explanation of mock-only nature
   - [x] Include setup instructions and educational purpose
   - [x] Reference plan documents and related OpenSpec changes

2. ✅ **Create mock domain types**
   - [x] Define `DemoUser`, `DemoTenant`, `DemoConnection`, `DemoSignal`, `DemoGroundedSignal` types
   - [x] Add supporting types like `DemoEvidenceItem`
   - [x] Include comprehensive TypeScript interfaces with JSDoc comments
   - [x] Place in `lib/demo/types.ts`

3. ✅ **Implement React Context state management**
   - [x] Create `DemoProvider` context component
   - [x] Implement state reducer for user/tenant/connections/signals/groundedSignals
   - [x] Add custom hooks: `useDemoUser()`, `useDemoTenant()`, etc.
   - [x] Include state persistence helpers (localStorage optional)
   - [x] Place in `lib/demo/state.ts`

4. ✅ **Create mock data generators**
   - [x] Implement deterministic ID generation utilities
   - [x] Create seeded generators for each entity type
   - [x] Add realistic signal generators for GitHub and Zoho Cliq
   - [x] Include evidence generation for grounding scenarios
   - [x] Place in `lib/demo/mockData.ts`

### Phase 2: Core UI Components

5. ✅ **Create demo layout and navigation**
   - [x] Update `app/layout.tsx` to include `DemoProvider`
   - [x] Add `DemoNavbar` component with progress indicator
   - [x] Create `MockBadge` component for "Mock Demo" labeling
   - [x] Implement `ProgressBar` showing flow completion
   - [x] Add responsive layout structure

6. ✅ **Implement landing page with mock login**
   - [x] Replace default `app/page.tsx` with demo landing
   - [x] Create centered login form with email input
   - [x] Add mock login flow with user creation
   - [x] Include educational annotations about mock authentication
   - [x] Add navigation to tenant setup

7. ✅ **Create tenant creation and mapping page**
   - [x] Implement `app/tenant/page.tsx` with tenant creation form
   - [x] Add dual ID generation and display
   - [x] Create visual mapping explanation component
   - [x] Include educational notes about `X-Tenant-Id` usage
   - [x] Add navigation to integrations

### Phase 3: Integration Management

8. ✅ **Build integrations hub page**
   - [x] Implement `app/integrations/page.tsx` with provider grid
   - [x] Create `ProviderTile` component for GitHub and Zoho Cliq
   - [x] Add connection status display and management
   - [x] Implement mock OAuth consent modal
   - [x] Include educational notes about real OAuth flows

9. ✅ **Implement GitHub connection flow**
   - [x] Add GitHub connection creation logic
   - [x] Create visual OAuth consent simulation
   - [x] Update connection state in context
   - [x] Add post-connection CTAs for signal scanning
   - [x] Include annotations about real Connectors endpoints

10. ✅ **Add Zoho Cliq integration**
    - [x] Extend `ProviderTile` for Zoho Cliq provider
    - [x] Implement connection flow similar to GitHub
    - [x] Add Zoho-specific signal types and content
    - [x] Enable multi-provider scenarios for grounding demo

### Phase 4: Signal Discovery

11. ✅ **Create signals list page**
    - [x] Implement `app/signals/page.tsx` with list/table view
    - [x] Add provider filtering and search functionality
    - [x] Create signal loading and empty states
    - [x] Implement navigation to signal detail pages
    - [x] Add educational notes about real `/signals` API

12. ✅ **Implement mock signal scanning**
    - [x] Add "Scan for signals" functionality
    - [x] Create loading states and progress indication
    - [x] Generate signals based on connected providers
    - [x] Update context with generated signals
    - [x] Include success/error handling patterns

13. ✅ **Build signal detail view**
    - [x] Implement `app/signals/[id]/page.tsx` with signal details
    - [x] Create comprehensive signal metadata display
    - [x] Add collapsible JSON-style raw data view
    - [x] Include navigation back to signals list
    - [x] Add educational annotations about signal structure

### Phase 5: Signal Grounding Demo

14. ✅ **Implement signal grounding functionality**
    - [x] Add "Ground this signal" action on detail page
    - [x] Create grounding generation with scoring algorithm
    - [x] Implement evidence aggregation across providers
    - [x] Display results with score visualization
    - [x] Include explanations of grounding concepts

15. ✅ **Create grounded signal display**
    - [x] Build `GroundedSignal` component for results
    - [x] Add dimensional score visualization (Relevance, Impact, etc.)
    - [x] Create evidence list grouped by source provider
    - [x] Include cross-connector relationship demonstration
    - [x] Add educational notes about production grounding

### Phase 6: Polish and Documentation

16. ✅ **Add comprehensive inline annotations**
    - [x] Place info hints with tooltips on key UI elements
    - [x] Add screen-level explanations about production equivalents
    - [x] Include code comments mapping mock to real API calls
    - [x] Create reference links to OpenSpec changes and API docs
    - [x] Ensure all major features have educational context

17. ✅ **Implement responsive design improvements**
    - [x] Test and optimize for mobile layouts
    - [x] Ensure all interactions work on touch devices
    - [x] Add accessibility improvements (ARIA labels, keyboard navigation)
    - [x] Verify color contrast and readability
    - [x] Test with screen readers where possible

18. ✅ **Add error handling and edge cases**
    - [x] Handle lost state on page refresh gracefully
    - [x] Add validation for form inputs
    - [x] Implement loading states for all async operations
    - [x] Add user-friendly error messages
    - [x] Include recovery options for failed operations

### Phase 7: Integration and Validation

19. ✅ **Update main project documentation**
    - [x] Add demo to main README with clear description
    - [x] Update integration guide to reference demo
    - [x] Add demo to developer onboarding checklist
    - [x] Include screenshots and setup instructions
    - [x] Cross-reference with related OpenSpec changes

20. ✅ **End-to-end testing and validation**
    - [x] Test complete flow from login to grounding
    - [x] Verify all mock data is deterministic and realistic
    - [x] Check all educational annotations are accurate
    - [x] Validate responsive design on multiple devices
    - [x] Ensure demo runs without external dependencies

21. ✅ **Performance optimization and cleanup**
    - [x] Optimize bundle size and loading performance
    - [x] Remove unused dependencies and code
    - [x] Add proper error boundaries
    - [x] Implement React performance optimizations where needed
    - [x] Clean up development artifacts

## Dependencies and Prerequisites

- Must have existing `examples/nextjs-demo` Next.js project
- Requires `plan/nextjs-demo/` documents to be complete and approved
- Related OpenSpec changes should be reviewed for consistency
- Design system (Tailwind + shadcn/ui) should be properly configured

## Validation Criteria

Each task should be considered complete when:
- Implementation matches requirements in corresponding spec
- Code follows project conventions and patterns
- Educational annotations are clear and accurate
- Component is tested manually for basic functionality
- No console errors or warnings in development mode

## Parallel Work Opportunities

- Tasks 2-4 (domain layer) can be worked in parallel
- Tasks 6-7 (UI pages) can be developed simultaneously
- Tasks 11-13 (signal pages) can be parallelized
- Documentation tasks (16, 19) can overlap with development

## Estimated Timeline

- **Phase 1**: 1-2 days (Foundation)
- **Phase 2**: 2-3 days (Core UI)
- **Phase 3**: 2-3 days (Integrations)
- **Phase 4**: 2-3 days (Signals)
- **Phase 5**: 2-3 days (Grounding)
- **Phase 6**: 1-2 days (Polish)
- **Phase 7**: 1-2 days (Validation)

**Total estimated effort**: 11-18 days depending on parallelization and experience level.