# Design: Enhanced Next.js Demo Mock Domain Model

## Architectural Overview

This enhancement extends the existing Next.js demo mock domain model while maintaining backward compatibility and clean separation of concerns. The design follows the same patterns established in the current implementation but adds additional layers of realism and educational value.

## Current State Analysis

The existing demo provides:
- ✅ Solid foundation with User, Tenant, Connection, Signal entities
- ✅ Sophisticated seeded random generation for deterministic data
- ✅ React Context-based state management
- ✅ GitHub and Zoho Cliq provider implementations
- ✅ Basic signal grounding and evidence concepts

## Enhancement Design

### 1. Extended Domain Entity Model

#### New Entities to Add:

*(Note: The following type definitions are illustrative. Implementation will be in `examples/nextjs-demo/lib/demo/types.ts`)*

**DemoSyncJob**: Represents scheduled synchronization tasks
```typescript
interface DemoSyncJob {
  id: string;
  tenantId: string;
  connectionId: string;
  providerSlug: string;
  status: 'pending' | 'running' | 'completed' | 'failed' | 'retrying';
  kind: 'full' | 'incremental' | 'webhook_triggered';
  cursor?: string;
  errorCount: number;
  lastRunAt?: string;
  nextRunAt?: string;
  createdAt: string;
  errorMessage?: string;
}
```

**DemoWebhook**: Represents incoming webhook events
```typescript
interface DemoWebhook {
  id: string;
  tenantId: string;
  connectionId: string;
  providerSlug: string;
  eventType: string;
  payload: Record<string, unknown>;
  signature?: string;
  verified: boolean;
  processedAt?: string;
  createdAt: string;
}
```

**DemoToken**: Represents OAuth tokens and credentials
```typescript
interface DemoToken {
  id: string;
  connectionId: string;
  tokenType: 'oauth' | 'api_key' | 'service_account';
  scopes: string[];
  expiresAt?: string;
  lastRefreshed?: string;
  status: 'active' | 'expired' | 'revoked' | 'refreshing';
}
```

**DemoRateLimit**: Represents API rate limiting state
```typescript
interface DemoRateLimit {
  id: string;
  connectionId: string;
  providerSlug: string;
  endpoint: string;
  currentLimit: number;
  remaining: number;
  resetAt: string;
  retryAfter?: number;
}
```

**DemoProviderConfig**: Extended provider configuration
```typescript
interface DemoProviderConfig {
  slug: string;
  name: string;
  description: string;
  iconUrl: string;
  supportedSignalKinds: string[];
  authType: 'oauth2' | 'api_key' | 'webhook' | 'hybrid';
  rateLimit?: {
    requestsPerHour: number;
    requestsPerMinute: number;
  };
  webhookEvents: string[];
  defaultScopes: string[];
  features: {
    realtimeWebhooks: boolean;
    historicalSync: boolean;
    incrementalSync: boolean;
    crossProviderCorrelation: boolean;
  };
}
```

### 2. Enhanced Signal Model

Extend the existing `DemoSignal` with additional fields:

```typescript
interface DemoSignal {
  // Existing fields remain unchanged...

  // Extended metadata (new additions)
  rawPayload: Record<string, unknown>;
  processingDetails: {
    fetchTime: number;
    processingTime: number;
    retryCount: number;
    lastRetryAt?: string;
  };

  // Relationship data
  relatedSignals: string[]; // IDs of related signals
  parentSignalId?: string;  // For grouped/threaded activities
  childSignalIds: string[]; // For activities that spawn other activities

  // Enhanced classification
  categories: string[];     // e.g., ['security', 'deployment', 'collaboration']
  sentiment?: 'positive' | 'negative' | 'neutral';
  urgency: 'low' | 'medium' | 'high' | 'critical';

  // Contextual information
  environment: 'production' | 'staging' | 'development';
  impact: {
    scope: 'team' | 'project' | 'organization' | 'public';
    affectedUsers?: number;
    estimatedCost?: number;
  };
}
```

### 3. Additional Provider Implementations

#### Slack Provider
- Signal kinds: `message_sent`, `message_received`, `mention`, `reaction_added`, `file_shared`, `channel_created`, `user_added`
- Webhook events: Real-time message events, reactions, file uploads
- Rate limiting: Tier-based (free vs. paid workspaces)

#### Google Workspace Provider
- Signal kinds: `email_sent`, `email_received`, `document_created`, `document_shared`, `calendar_event_created`, `drive_file_modified`
- Auth: OAuth2 with service account support
- Features: Cross-product integration (Docs mentioning Drive files, etc.)

#### Jira Provider
- Signal kinds: `issue_created`, `issue_updated`, `issue_assigned`, `sprint_started`, `workflow_transition`, `comment_added`
- Rate limiting: Complex tier based on Jira Cloud plan
- Webhook support: Real-time issue and project events

### 4. Advanced Scenario Simulation

#### Error Handling Scenarios
- Network timeouts and retry logic
- API rate limiting and backoff strategies
- Authentication token expiration and refresh
- Webhook signature validation failures
- Partial sync failures and recovery

#### Performance Scenarios
- Large dataset handling with pagination
- Concurrent sync job management
- Memory-efficient signal processing
- Progressive loading for UI components

#### Configuration Scenarios
- Adjustable signal volume and frequency
- Configurable mock data complexity
- Toggle between fast demo and realistic timing
- Custom signal filtering rules

### 5. Enhanced State Management

Extend the existing state structure:

```typescript
interface DemoState {
  // Existing fields remain unchanged...

  // New entity collections
  syncJobs: DemoSyncJob[];
  webhooks: DemoWebhook[];
  tokens: DemoToken[];
  rateLimits: DemoRateLimit[];

  // Enhanced loading states
  loading: DemoState['loading'] & {
    syncJobs: boolean;
    webhooks: boolean;
    tokens: boolean;
    rateLimits: boolean;
  };

  // Enhanced error states
  errors: DemoState['errors'] & {
    syncJobs?: string;
    webhooks?: string;
    tokens?: string;
    rateLimits?: string;
  };

  // Demo configuration
  config: {
    signalFrequency: 'low' | 'medium' | 'high';
    errorRate: '0%' | '10%' | '20%';  // Error rate for demo scenarios
    timingMode: 'fast' | 'realistic';
    providerComplexity: 'simple' | 'detailed';
  };
}
```

### 6. UI/UX Enhancements

#### New Components
- **SyncJobMonitor**: Real-time sync job status dashboard
- **WebhookViewer**: Incoming webhook event log
- **TokenManager**: OAuth token status and management
- **RateLimitStatus**: API rate limiting visualization
- **ConfigurationPanel**: Demo settings and controls

#### Enhanced Existing Components
- **SignalList**: Add signal grouping, threading, and advanced filtering
- **ProviderTile**: Show detailed provider capabilities and status
- **ConnectionDetail**: Display sync jobs, tokens, and rate limits per connection

## Implementation Strategy

### Phase 1: Core Extensions
1. Extend TypeScript interfaces without breaking existing code
2. Implement new mock data generators for additional entities
3. Add basic state management for new entities
4. Create simple UI components for visualization

### Phase 2: Provider Implementations
1. Implement Slack, Google Workspace, and Jira mock generators
2. Add provider-specific signal types and metadata
3. Implement realistic timing and rate limiting per provider
4. Add cross-provider correlation logic

### Phase 3: Advanced Scenarios
1. Implement error simulation and retry logic
2. Add webhook processing simulation
3. Implement rate limiting and backoff scenarios
4. Add configuration controls for demo behavior

### Phase 4: UI Polish
1. Create comprehensive dashboard views
2. Add detailed signal relationship visualization
3. Implement interactive configuration controls
4. Add educational annotations and help content

### Phase 5: Documentation and Testing
1. Update inline documentation and comments
2. Create usage examples and tutorials
3. Add TypeScript strict mode compliance
4. Performance optimization and cleanup

## Data Flow Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   UI Actions    │───▶│  State Store    │───▶│ Mock Generators │
│                 │    │                 │    │                 │
│ - Connect       │    │ - Users         │    │ - Deterministic │
│ - Scan Signals  │    │ - Tenants       │    │   Seeded RNG    │
│ - Configure     │    │ - Connections   │    │ - Realistic     │
│                 │    │ - Signals       │    │   Timing        │
└─────────────────┘    │ - Sync Jobs     │    │ - Error Cases   │
                       │ - Webhooks      │    └─────────────────┘
                       │ - Tokens        │             │
                       │ - Rate Limits   │             ▼
                       └─────────────────┘    ┌─────────────────┐
                                              │  React Context  │
                                              │                 │
                                              │ - Efficient     │
                                              │   Subscriptions │
                                              │ - Derived State │
                                              │ - Actions       │
                                              └─────────────────┘
```

## Performance Considerations

### Memory Management
- Use lazy loading for large signal datasets
- Implement virtual scrolling for signal lists
- Cache computed values and derived state
- Implement signal cleanup for old data

### React Rendering
- Use useMemo and useCallback for expensive operations
- Implement proper key props for lists
- Avoid unnecessary re-renders with stable references
- Use React.memo for components with expensive renders

### Data Generation
- Generate data on-demand rather than all at once
- Use seeded random for consistency but cache results
- Implement background generation for complex scenarios
- Use Web Workers for computationally expensive operations

## Testing Strategy

### Unit Tests
- Mock data generator determinism tests
- Type safety validation tests
- State management reducer tests
- Utility function tests

### Integration Tests
- Component rendering tests
- User interaction flow tests
- Configuration change tests
- Cross-entity relationship tests

### Performance Tests
- Large dataset rendering tests
- Memory usage monitoring
- Rendering performance benchmarks
- Data generation performance tests

## Migration Strategy

Since this enhancement extends the existing domain model without breaking changes, migration can be incremental:

1. **Backward Compatibility**: All existing types and interfaces remain functional
2. **Feature Flags**: New features can be enabled/disabled via configuration
3. **Progressive Enhancement**: UI components can opt-in to new features
4. **Graceful Degradation**: Advanced features fall back to basic functionality

## Risks and Mitigations

### Complexity Management
- **Risk**: The enhanced model becomes too complex for new users
- **Mitigation**: Progressive disclosure with simple/advanced modes and clear documentation

### Performance Impact
- **Risk**: Additional features slow down the demo experience
- **Mitigation**: Lazy loading, caching, and configuration controls for feature complexity

### Maintenance Burden
- **Risk**: More code and types become difficult to maintain
- **Mitigation**: Clear separation of concerns, comprehensive documentation, and automated testing

### Educational Value Dilution
- **Risk**: Complex features obscure the core Connectors concepts
- **Mitigation**: Clear annotations, guided tours, and focused use case demonstrations