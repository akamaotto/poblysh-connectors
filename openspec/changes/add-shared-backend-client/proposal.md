# Add Shared Backend Client to Next.js Demo

## Summary

This change introduces a production-ready, shared backend client for the Next.js demo that provides a unified interface for communicating with the Poblysh Connectors API. The client will enhance the existing mock/real mode routing system with proper error handling, authentication, rate limiting, and connection pooling.

## Motivation

The current Next.js demo has a sophisticated API routing system (`apiRouter.ts`) that supports both mock and real modes, but the real API client implementation is incomplete. This change will:

1. Complete the real API client implementation with proper error handling
2. Add authentication support for Connectors API endpoints
3. Implement retry logic and rate limiting for production resilience
4. Provide a shared client that can be used consistently across all demo components
5. Enhance the educational value by showing production-ready patterns

## Design Decisions

### Client Architecture
- **Singleton Pattern**: Single client instance shared across the application
- **Configuration-driven**: Client behavior controlled by demo mode configuration
- **Type Safety**: Full TypeScript support with proper API response typing
- **Error Boundaries**: Graceful degradation when API calls fail

### Authentication Strategy
- **Token-based**: Support for Bearer token authentication
- **Tenant Scoping**: Automatic inclusion of `X-Tenant-Id` headers
- **Development Mode**: Mock authentication tokens for demo purposes

### Resilience Features
- **Exponential Backoff**: Retry logic for failed requests
- **Rate Limiting**: Client-side rate limiting to prevent API abuse
- **Circuit Breaking**: Fail-fast behavior when API is consistently unavailable
- **Graceful Fallbacks**: Automatic fallback to mock data when real API fails

## Implementation Plan

1. **Core Client Implementation**
   - Create `SharedBackendClient` class with full API coverage
   - Implement proper authentication and request handling
   - Add error handling and retry logic

2. **Integration with Existing System**
   - Enhance `apiRouter.ts` to use the new shared client via wrapper pattern
   - The shared client will implement the existing `DemoApiClient` interface internally
   - Update configuration system to support client options
   - Maintain backward compatibility with existing mock system

**Integration Strategy:**
The shared client will use a **wrapper pattern** around the existing `DemoApiClient` interface:

```typescript
// Existing interface (unchanged)
export interface DemoApiClient {
  getProviders(): Promise<DemoApiResponse<DemoProvider[]>>;
  getConnections(): Promise<DemoApiResponse<DemoConnection[]>>;
  createConnection(connection: Omit<DemoConnection, 'id' | 'createdAt'>): Promise<DemoApiResponse<DemoConnection>>;
  updateConnection(id: string, updates: Partial<DemoConnection>): Promise<DemoApiResponse<DemoConnection>>;
  deleteConnection(id: string): Promise<void>;
  getSignals(): Promise<DemoApiResponse<DemoSignal[]>>;
  getGroundedSignals(): Promise<DemoApiResponse<DemoGroundedSignal[]>>;
  getSyncJobs(): Promise<DemoApiResponse<DemoSyncJob[]>>;
  getWebhooks(): Promise<DemoApiResponse<DemoWebhook[]>>;
  getTokens(): Promise<DemoApiResponse<DemoToken[]>>;
  getRateLimits(): Promise<DemoApiResponse<DemoRateLimit[]>>;
}

// New wrapper implementation that implements DemoApiClient interface
export class SharedBackendClientWrapper implements DemoApiClient {
  private sharedClient: SharedBackendClient;

  constructor(config?: SharedBackendClientConfig) {
    this.sharedClient = createSharedBackendClient(config);
  }

  // DemoApiClient methods - delegate to shared client
  async getProviders(): Promise<DemoApiResponse<DemoProvider[]>> {
    return this.sharedClient.getProviders();
  }

  async getConnections(): Promise<DemoApiResponse<DemoConnection[]>> {
    return this.sharedClient.getConnections();
  }

  async createConnection(connection: Omit<DemoConnection, 'id' | 'createdAt'>): Promise<DemoApiResponse<DemoConnection>> {
    return this.sharedClient.createConnection(connection);
  }

  async updateConnection(id: string, updates: Partial<DemoConnection>): Promise<DemoApiResponse<DemoConnection>> {
    return this.sharedClient.updateConnection(id, updates);
  }

  async deleteConnection(id: string): Promise<void> {
    return this.sharedClient.deleteConnection(id);
  }

  async getSignals(): Promise<DemoApiResponse<DemoSignal[]>> {
    return this.sharedClient.getSignals();
  }

  async getGroundedSignals(): Promise<DemoApiResponse<DemoGroundedSignal[]>> {
    return this.sharedClient.getGroundedSignals();
  }

  async getSyncJobs(): Promise<DemoApiResponse<DemoSyncJob[]>> {
    return this.sharedClient.getSyncJobs();
  }

  async getWebhooks(): Promise<DemoApiResponse<DemoWebhook[]>> {
    return this.sharedClient.getWebhooks();
  }

  async getTokens(): Promise<DemoApiResponse<DemoToken[]>> {
    return this.sharedClient.getTokens();
  }

  async getRateLimits(): Promise<DemoApiResponse<DemoRateLimit[]>> {
    return this.sharedClient.getRateLimits();
  }

  // Enhanced methods available only when accessing SharedBackendClient directly
  async getCurrentUser(): Promise<DemoApiResponse<DemoUser>> {
    return this.sharedClient.getCurrentUser();
  }

  async refreshToken(): Promise<DemoApiResponse<DemoToken>> {
    return this.sharedClient.refreshToken();
  }

  async createSyncJob(syncJob: Omit<DemoSyncJob, 'id' | 'createdAt'>): Promise<DemoApiResponse<DemoSyncJob>> {
    return this.sharedClient.createSyncJob(syncJob);
  }

  async createWebhook(webhook: Omit<DemoWebhook, 'id' | 'createdAt'>): Promise<DemoApiResponse<DemoWebhook>> {
    return this.sharedClient.createWebhook(webhook);
  }

  async deleteWebhook(id: string): Promise<void> {
    return this.sharedClient.deleteWebhook(id);
  }
}

// apiRouter.ts updates
let apiClient: DemoApiClient;
let sharedClient: SharedBackendClient | null = null;

export function initializeApiClient() {
  const config = getDemoConfig();

  if (config.mode === 'real') {
    const sharedBackendClient = createSharedBackendClient({
      baseUrl: config.apiBaseUrl,
      authToken: config.authToken,
      tenantId: config.tenantId,
      // ... other config
    });

    apiClient = new SharedBackendClientWrapper(sharedBackendClient);
    sharedClient = sharedBackendClient; // Store for direct access to enhanced features
  } else {
    apiClient = new MockApiClient(); // existing implementation
    sharedClient = null;
  }
}

export function getApiClient(): DemoApiClient {
  return apiClient;
}

// New function to access enhanced features when available
export function getSharedBackendClient(): SharedBackendClient | null {
  return sharedClient;
}
```

**Backward Compatibility:**
- All existing component code continues to work without changes
- `getApiClient()` returns the same `DemoApiClient` interface
- Components that call `apiClient.getProviders()` work identically
- Only the internal implementation changes, not the public interface

3. **Production Features**
   - Add request/response interceptors for logging
   - Implement connection pooling and caching
   - Add performance monitoring and metrics

4. **Educational Enhancements**
   - Add inline documentation explaining real-world patterns
   - Include configuration examples for different deployment scenarios
   - Provide troubleshooting guides for common issues

## Backward Compatibility

This change maintains full backward compatibility with the existing Next.js demo:

- Mock mode continues to work exactly as before
- All existing component interfaces remain unchanged
- Configuration system is enhanced but doesn't break existing configs
- Educational annotations are preserved and enhanced

## Success Criteria

### Functional Requirements
- [ ] Real API mode successfully connects to actual Connectors API endpoints with 99% uptime in normal conditions
- [ ] Mock mode continues to work without external dependencies and maintains <100ms response times
- [ ] Client handles all API endpoints (providers, connections, signals, sync jobs, webhooks) with consistent interfaces
- [ ] Authentication works with tenant-scoped requests using Bearer tokens and X-Tenant-Id headers

### Performance Requirements
- [ ] API response times: <500ms for 95th percentile in real mode, <100ms for 95th percentile in mock mode
- [ ] Retry logic successfully recovers from transient failures in 90% of cases within 10 seconds
- [ ] Rate limiting prevents API abuse while allowing legitimate usage patterns (10 req/s sustained, 20 req/s burst)
- [ ] Circuit breaker prevents cascading failures and recovers within 30 seconds after API restoration

### Error Handling Requirements
- [ ] Proper error handling with meaningful error messages and specific error codes (AUTH_001-AUTH_005 for authentication, NET_001-NET_005 for network issues)
- [ ] Graceful fallback to mock data when real API is unavailable with user notification
- [ ] Token refresh workflow automatically handles expired tokens without user intervention
- [ ] All error scenarios are covered with specific TypeScript interfaces (AuthenticationError, NetworkError) and documentation
- [ ] Network errors are properly categorized with retry logic and appropriate timeout handling

### Testing Requirements
- [ ] All existing tests continue to pass with 100% success rate
- [ ] New tests achieve 95% code coverage for shared client functionality
- [ ] Load testing demonstrates 100 concurrent requests handled without degradation
- [ ] Failover testing confirms graceful degradation from real to mock mode

### Educational Requirements
- [ ] Educational value is enhanced with production patterns and inline documentation
- [ ] All client methods include JSDoc documentation with usage examples
- [ ] Configuration examples provided for development, staging, and production environments
- [ ] Troubleshooting guide covers common integration issues and solutions

### Integration Requirements
- [ ] Backward compatibility maintained: existing components work without code changes
- [ ] Integration with existing apiRouter.ts follows wrapper pattern without breaking current interfaces
- [ ] Configuration system extends existing demo config without breaking existing configurations
- [ ] Bundle size increase limited to <50KB gzipped for the shared client implementation

## Future Considerations

- **WebSocket Support**: Real-time signal updates via WebSocket connections
- **Caching Strategy**: Implement intelligent caching for frequently accessed data
- **Multi-tenant Support**: Enhanced support for switching between different tenant contexts
- **Offline Mode**: Better offline capabilities with queue-based request persistence