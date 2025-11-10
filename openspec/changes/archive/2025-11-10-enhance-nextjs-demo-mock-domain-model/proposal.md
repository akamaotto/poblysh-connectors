# Enhance Next.js Demo Mock Domain Model

## Change Summary

This change enhances the existing Next.js demo mock domain model to provide a more comprehensive and realistic simulation of the Poblysh Connectors integration story. While the current implementation already provides a solid foundation, this enhancement adds missing domain entities, expands signal types, introduces realistic data relationships, and improves the educational value of the demo.

## Why

The current Next.js demo mock domain model provides a good foundation but lacks several key aspects that would make it a truly comprehensive educational tool:

1. **Incomplete Domain Coverage**: Several important Connectors service entities are missing, including SyncJob, Webhook, Token, and RateLimit entities, which limits the demo's ability to show the complete integration picture.

2. **Limited Provider Variety**: Only GitHub and Zoho Cliq are currently implemented, while the actual Connectors service supports Slack, Google Workspace, Jira, and other providers. This limits users' understanding of multi-provider scenarios.

3. **Simplified Scenarios**: The demo lacks realistic complexity such as error handling, rate limiting, webhook processing, and performance considerations that users would encounter in production.

4. **Missing Configuration Options**: Users cannot adjust demo behavior to focus on specific aspects they want to learn about, making the demo less flexible for different educational needs.

5. **Educational Value Gap**: The current implementation doesn't fully demonstrate advanced concepts like cross-provider signal correlation, evidence chaining, and real-time webhook processing.

Enhancing the mock domain model will significantly improve the demo's educational value by providing a more realistic, comprehensive, and configurable simulation of the complete Connectors integration story.

## Problem Statement

The current Next.js demo mock domain model is functional but could benefit from:

1. **Missing domain entities**: Some important concepts from the Connectors service are not represented (SyncJobs, Webhooks, Tokens, etc.)
2. **Limited signal variety**: Only GitHub and Zoho Cliq are implemented; other providers like Google Workspace, Slack, and Jira are missing
3. **Simplified relationships**: The relationships between entities could be more realistic and interconnected
4. **Missing advanced scenarios**: Complex scenarios like rate limiting, partial syncs, error handling, and webhook processing are not simulated
5. **Limited configuration options**: The demo lacks configurable parameters for different use cases

## Proposed Solution

Enhance the mock domain model by adding:

1. **Additional domain entities**: SyncJob, Webhook, Token, ProviderConfig, RateLimit
2. **Extended provider support**: Add mock implementations for Slack, Google Workspace, Jira, and other planned providers
3. **Realistic signal relationships**: Enhanced cross-provider correlations and evidence chains
4. **Advanced scenario simulation**: Error states, rate limiting, webhook failures, partial syncs
5. **Configurable mock parameters**: Allow demo users to adjust signal volumes, timing, and complexity
6. **Enhanced metadata**: More detailed signal payloads, provider-specific configurations, and realistic data structures

## Scope

### In Scope
- Extend existing TypeScript interfaces and types
- Add new mock data generators for additional providers
- Enhance the state management to handle new entities
- Update UI components to display new entity types
- Add configuration controls for demo parameters
- Implement advanced scenario simulations

### Out of Scope
- Real API integration (remains mock-only)
- Changes to the actual Connectors service
- Database schema changes
- Authentication/authorization changes

## Success Criteria

1. The Next.js demo includes representations of all major Connectors service domain entities
2. Users can interact with 5+ different provider types (vs. current 2)
3. Advanced scenarios (rate limiting, errors, partial syncs) are demonstrable
4. Demo configuration is adjustable through UI controls
5. All new entities are properly typed and documented
6. Existing functionality remains unchanged and backward compatible

## Technical Approach

1. **Phase 1**: Extend type definitions and add new domain entities
2. **Phase 2**: Implement mock data generators for new providers and entities
3. **Phase 3**: Enhance state management and UI components
4. **Phase 4**: Add configuration controls and advanced scenarios
5. **Phase 5**: Documentation and testing

## Dependencies

- Existing Next.js demo implementation
- Current OpenSpec nextjs-demo specification
- No external dependencies required

## Timeline Estimate

- **Phase 1**: 1-2 days (type definitions)
- **Phase 2**: 2-3 days (mock data generators)
- **Phase 3**: 2-3 days (state management and UI)
- **Phase 4**: 1-2 days (configuration and scenarios)
- **Phase 5**: 1 day (documentation)

**Total**: 7-11 days

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Complexity becomes overwhelming | Medium | Implement incrementally with clear separation of concerns |
| Performance degradation in browser | Low | Use efficient data structures and lazy loading |
| UI becomes cluttered with new features | Medium | Design clean interfaces with progressive disclosure |