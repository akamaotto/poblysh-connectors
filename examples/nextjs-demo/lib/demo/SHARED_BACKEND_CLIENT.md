# Shared Backend Client

A production-ready, shared backend client for the Next.js demo that provides a unified interface for communicating with the Poblysh Connectors API. The client enhances the existing mock/real mode routing system with proper error handling, authentication, rate limiting, and connection pooling.

## Features

### 🚀 Production-Ready Features
- **Authentication**: Bearer token support with automatic `X-Tenant-Id` header injection
- **Error Handling**: Comprehensive error handling with specific error codes (AUTH_001-AUTH_005, NET_001-NET_005)
- **Retry Logic**: Exponential backoff retry for transient failures
- **Rate Limiting**: Client-side rate limiting to prevent API abuse
- **Circuit Breaking**: Fail-fast behavior when API is consistently unavailable
- **Graceful Fallbacks**: Automatic fallback to mock data when real API fails
- **Request Monitoring**: Comprehensive metrics collection and logging

### 🎓 Educational Features
- **Inline Documentation**: JSDoc comments with usage examples
- **Performance Annotations**: Educational logging about request performance
- **Configuration Examples**: Sample configurations for different environments
- **Troubleshooting Guides**: Common issues and solutions

## Quick Start

### Basic Usage

The shared backend client integrates seamlessly with the existing `apiRouter.ts`. Existing components continue to work without any changes:

```typescript
import { getApiClient } from '@/lib/demo/apiRouter';

// Existing code continues to work unchanged
const apiClient = getApiClient();
const providers = await apiClient.getProviders();
```

### Enhanced Features

For advanced features, access the shared client directly:

```typescript
import { getApiClient, getSharedBackendClient } from '@/lib/demo/apiRouter';

// Standard usage (backward compatible)
const apiClient = getApiClient();
const connections = await apiClient.getConnections();

// Enhanced features (only available in real mode)
const sharedClient = getSharedBackendClient();
if (sharedClient) {
  // Get current user information
  const user = await sharedClient.getCurrentUser();

  // Refresh authentication token
  const token = await sharedClient.refreshToken();

  // Get performance metrics
  const metrics = sharedClient.getMetrics();

  // Monitor circuit breaker state
  const circuitState = sharedClient.getCircuitBreakerState();
}
```

## Configuration

### Environment Variables

Configure the client using environment variables:

```bash
# Set demo mode
NEXT_PUBLIC_DEMO_MODE=real

# Configure API base URL for real mode
CONNECTORS_API_BASE_URL=https://your-connectors-api.example.com
```

### Advanced Configuration

For advanced use cases, create the client with custom configuration:

```typescript
import { createSharedBackendClient } from '@/lib/demo/sharedBackendClient';

const client = createSharedBackendClient({
  baseUrl: 'https://api.example.com',
  authToken: 'your-auth-token',
  tenantId: 'your-tenant-id',
  timeout: 30000,
  maxRetries: 5,
  retryDelay: 1000,
  rateLimit: {
    requestsPerSecond: 10,
    burstCapacity: 20,
  },
  circuitBreaker: {
    failureThreshold: 5,
    timeout: 30000,
    successThreshold: 3,
  },
  enableLogging: true,
  enableEducationalAnnotations: true,
});
```

## Error Handling

The client provides detailed error handling with specific error codes:

### Authentication Errors (AUTH_001-AUTH_005)

```typescript
import { isAuthenticationError, AUTH_ERRORS } from '@/lib/demo/sharedBackendClient';

try {
  await apiClient.getProviders();
} catch (error) {
  if (isAuthenticationError(error)) {
    console.error('Authentication failed:', error.message);
    console.error('Recovery:', AUTH_ERRORS[error.code].recovery);
  }
}
```

### Network Errors (NET_001-NET_005)

```typescript
import { isNetworkError, NETWORK_ERRORS } from '@/lib/demo/sharedBackendClient';

try {
  await apiClient.getProviders();
} catch (error) {
  if (isNetworkError(error)) {
    console.error('Network error:', error.message);
    console.error('Recovery:', NETWORK_ERRORS[error.code].recovery);
  }
}
```

## API Methods

### Standard DemoApiClient Methods

All standard methods from `DemoApiClient` are supported:

```typescript
// Providers
const providers = await apiClient.getProviders();

// Connections
const connections = await apiClient.getConnections();
const newConnection = await apiClient.createConnection(connectionData);
const updatedConnection = await apiClient.updateConnection('id', updates);
await apiClient.deleteConnection('id');

// Signals
const signals = await apiClient.getSignals();
const groundedSignals = await apiClient.getGroundedSignals();

// Sync Jobs
const syncJobs = await apiClient.getSyncJobs();

// Webhooks
const webhooks = await apiClient.getWebhooks();

// Tokens
const tokens = await apiClient.getTokens();

// Rate Limits
const rateLimits = await apiClient.getRateLimits();
```

### Enhanced Methods (SharedBackendClient only)

Additional methods available when accessing the shared client directly:

```typescript
const sharedClient = getSharedBackendClient();

if (sharedClient) {
  // User management
  const currentUser = await sharedClient.getCurrentUser();
  const newToken = await sharedClient.refreshToken();

  // Sync job management
  const newSyncJob = await sharedClient.createSyncJob(syncJobData);

  // Webhook management
  const newWebhook = await sharedClient.createWebhook(webhookData);
  await sharedClient.deleteWebhook('webhook-id');

  // Monitoring
  const metrics = sharedClient.getMetrics();
  const circuitState = sharedClient.getCircuitBreakerState();
  const rateLimitState = sharedClient.getRateLimiterState();

  // Circuit breaker management
  sharedClient.resetCircuitBreaker();

  // Configuration updates
  sharedClient.updateConfig({ timeout: 15000 });
}
```

## Monitoring and Metrics

### Request Metrics

Track request performance and success rates:

```typescript
const sharedClient = getSharedBackendClient();
if (sharedClient) {
  const metrics = sharedClient.getMetrics();

  metrics.forEach(metric => {
    console.log(`Request ${metric.requestId}:`);
    console.log(`  Method: ${metric.method}`);
    console.log(`  Endpoint: ${metric.endpoint}`);
    console.log(`  Duration: ${metric.duration}ms`);
    console.log(`  Success: ${metric.success}`);
    console.log(`  Retries: ${metric.retryCount}`);

    if (metric.error) {
      console.log(`  Error: ${metric.error}`);
    }
  });

  // Clear metrics when needed
  sharedClient.clearMetrics();
}
```

### Circuit Breaker Monitoring

Monitor circuit breaker state for API health:

```typescript
const sharedClient = getSharedBackendClient();
if (sharedClient) {
  const circuitState = sharedClient.getCircuitBreakerState();

  console.log(`Circuit State: ${circuitState.state}`);
  console.log(`Failure Count: ${circuitState.failureCount}`);
  console.log(`Success Count: ${circuitState.successCount}`);

  if (circuitState.state === 'OPEN') {
    console.warn('Circuit breaker is OPEN - API calls will fail fast');
  }
}
```

## Environment Configurations

### Development

```typescript
const devConfig = {
  timeout: 10000,
  maxRetries: 1,
  enableLogging: true,
  enableEducationalAnnotations: true,
  logging: {
    requests: true,
    responses: true,
    errors: true,
    logBodies: true,
  },
};
```

### Production

```typescript
const prodConfig = {
  timeout: 30000,
  maxRetries: 5,
  enableLogging: false,
  enableEducationalAnnotations: false,
  logging: {
    requests: false,
    responses: false,
    errors: true,
    logBodies: false,
  },
};
```

### Testing

```typescript
const testConfig = {
  timeout: 5000,
  maxRetries: 0,
  enableLogging: true,
  enableEducationalAnnotations: false,
};
```

## Migration Guide

### Phase 1: No Changes Required

Existing components continue to work without any modifications:

```typescript
// This continues to work exactly as before
import { getApiClient } from '@/lib/demo/apiRouter';

const apiClient = getApiClient();
const data = await apiClient.getProviders();
```

### Phase 2: Optional Enhanced Features

Components can optionally use enhanced features:

```typescript
import { getApiClient, getSharedBackendClient, hasEnhancedFeatures } from '@/lib/demo/apiRouter';

export default function DataComponent() {
  const [usingEnhancedFeatures, setUsingEnhancedFeatures] = useState(false);

  useEffect(() => {
    async function loadData() {
      try {
        const apiClient = getApiClient();
        const response = await apiClient.getProviders();

        // Optionally use enhanced features
        if (hasEnhancedFeatures()) {
          const sharedClient = getSharedBackendClient();
          if (sharedClient) {
            const metrics = sharedClient.getMetrics();
            console.log('Request metrics:', metrics);
            setUsingEnhancedFeatures(true);
          }
        }

        // Process data...
      } catch (error) {
        console.error('Failed to load data:', error);
      }
    }

    loadData();
  }, []);

  return (
    <div>
      {usingEnhancedFeatures && (
        <div className="enhanced-features-indicator">
          🚀 Using enhanced API features
        </div>
      )}
      {/* Component content */}
    </div>
  );
}
```

### Phase 3: Full Enhanced Integration

Full integration with all enhanced features:

```typescript
import { getSharedBackendClient } from '@/lib/demo/apiRouter';

export default function AdvancedComponent() {
  const [metrics, setMetrics] = useState(null);
  const [circuitState, setCircuitState] = useState(null);

  useEffect(() => {
    const sharedClient = getSharedBackendClient();
    if (sharedClient) {
      // Get real-time metrics
      const updateMetrics = () => {
        setMetrics(sharedClient.getMetrics());
        setCircuitState(sharedClient.getCircuitBreakerState());
      };

      // Update metrics periodically
      const interval = setInterval(updateMetrics, 5000);
      updateMetrics();

      return () => clearInterval(interval);
    }
  }, []);

  return (
    <div>
      {circuitState && (
        <div className={`circuit-status circuit-${circuitState.state.toLowerCase()}`}>
          Circuit Breaker: {circuitState.state}
        </div>
      )}

      {metrics && (
        <div className="metrics-dashboard">
          <h4>API Performance</h4>
          <p>Total Requests: {metrics.length}</p>
          <p>Success Rate: {metrics.filter(m => m.success).length / metrics.length * 100}%</p>
          <p>Average Duration: {metrics.reduce((sum, m) => sum + m.duration, 0) / metrics.length}ms</p>
        </div>
      )}
    </div>
  );
}
```

## Troubleshooting

### Common Issues

1. **"No base URL configured for real API mode"**
   - **Solution**: Set `CONNECTORS_API_BASE_URL` environment variable or provide `baseUrl` in configuration

2. **"Circuit breaker is open"**
   - **Solution**: Wait for automatic recovery or manually reset with `sharedClient.resetCircuitBreaker()`

3. **"Rate limit exceeded"**
   - **Solution**: Implement client-side throttling or reduce request frequency

4. **Authentication errors**
   - **Solution**: Verify auth token is valid and not expired, check tenant ID

### Debug Mode

Enable detailed logging for troubleshooting:

```typescript
const client = createSharedBackendClient({
  enableLogging: true,
  enableEducationalAnnotations: true,
  logging: {
    requests: true,
    responses: true,
    errors: true,
    logBodies: false, // Be careful with sensitive data
  },
});
```

## Educational Features

The shared backend client includes educational annotations to help understand API patterns:

### Performance Insights

```
📚 [EDUCATIONAL] Request completed successfully {
  requestId: 'req-1234567890-abc123',
  duration: '150ms',
  statusCode: 200
}
```

### Error Explanations

```
❌ [ERROR] Authentication failed: AUTH_001
💡 [RECOVERY] Check authentication token and permissions
📖 [LEARN] See documentation: https://docs.example.com/auth-errors
```

## Type Safety

The client provides comprehensive TypeScript support:

```typescript
import {
  SharedBackendClient,
  SharedBackendClientConfig,
  AuthenticationError,
  NetworkError,
  isAuthenticationError,
  isNetworkError
} from '@/lib/demo/sharedBackendClient';

// Type-safe configuration
const config: SharedBackendClientConfig = {
  baseUrl: 'https://api.example.com',
  timeout: 30000,
};

// Type-safe error handling
try {
  await client.getProviders();
} catch (error) {
  if (isAuthenticationError(error)) {
    // TypeScript knows this is AuthenticationError
    console.log(error.code); // AUTH_001, AUTH_002, etc.
  }
}
```

## Contributing

When extending the shared backend client:

1. **Add comprehensive tests** for new features
2. **Update TypeScript types** for new methods
3. **Add JSDoc documentation** with examples
4. **Include educational annotations** for demo mode
5. **Test both mock and real modes**
6. **Update this README** with new capabilities

## License

This implementation follows the same license as the Poblysh Connectors project.