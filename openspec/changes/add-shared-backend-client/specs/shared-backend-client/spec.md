# shared-backend-client Specification

## Purpose
Provides a production-ready, shared backend client for the Next.js demo that enables real API communication with the Poblysh Connectors service while maintaining the existing mock functionality.

## ADDED Requirements

### Requirement: Unified Backend Client Interface
The system SHALL provide a shared backend client that offers a consistent interface for both mock and real API modes while maintaining backward compatibility.

#### Scenario: Client instantiation and configuration
Given a developer wants to use the backend client in their Next.js demo component
When they call `getSharedBackendClient()` or `createSharedBackendClient(config)`
Then the client should be properly configured based on the current demo mode
And should return a client that implements the `SharedBackendClient` interface
And should handle both mock and real API configurations seamlessly

**TypeScript Interface Definition:**
```typescript
interface SharedBackendClientConfig {
  /** Base URL for the Connectors API */
  baseUrl?: string;
  /** Authentication token for API requests */
  authToken?: string;
  /** Tenant ID for multi-tenant requests */
  tenantId?: string;
  /** Request timeout in milliseconds */
  timeout?: number;
  /** Retry configuration */
  retry?: {
    maxAttempts: number;
    baseDelay: number;
    maxDelay: number;
    multiplier: number;
    jitter: boolean;
  };
  /** Rate limiting configuration */
  rateLimit?: {
    requestsPerSecond: number;
    burstCapacity: number;
  };
  /** Circuit breaker configuration */
  circuitBreaker?: {
    failureThreshold: number;
    recoveryTimeout: number;
    monitoringPeriod: number;
  };
  /** Enable request/response logging */
  enableLogging?: boolean;
  /** Log level for debugging */
  logLevel?: 'debug' | 'info' | 'warn' | 'error';
}

interface SharedBackendClient {
  // Provider operations (matches existing DemoApiClient)
  getProviders(): Promise<DemoApiResponse<DemoProvider[]>>;

  // Connection operations (matches existing DemoApiClient)
  getConnections(): Promise<DemoApiResponse<DemoConnection[]>>;
  createConnection(connection: Omit<DemoConnection, 'id' | 'createdAt'>): Promise<DemoApiResponse<DemoConnection>>;
  updateConnection(id: string, updates: Partial<DemoConnection>): Promise<DemoApiResponse<DemoConnection>>;
  deleteConnection(id: string): Promise<void>;

  // Signal operations (matches existing DemoApiClient)
  getSignals(): Promise<DemoApiResponse<DemoSignal[]>>;
  getGroundedSignals(): Promise<DemoApiResponse<DemoGroundedSignal[]>>;

  // Sync job operations (matches existing DemoApiClient - read-only)
  getSyncJobs(): Promise<DemoApiResponse<DemoSyncJob[]>>;

  // Webhook operations (matches existing DemoApiClient - read-only)
  getWebhooks(): Promise<DemoApiResponse<DemoWebhook[]>>;

  // Token operations (matches existing DemoApiClient - read-only)
  getTokens(): Promise<DemoApiResponse<DemoToken[]>>;

  // Rate limit operations (matches existing DemoApiClient - array format)
  getRateLimits(): Promise<DemoApiResponse<DemoRateLimit[]>>;

  // Enhanced authentication operations (additional methods beyond DemoApiClient)
  getCurrentUser(): Promise<DemoApiResponse<DemoUser>>;  // DemoUser type from existing types.ts
  refreshToken(): Promise<DemoApiResponse<DemoToken>>;

  // Enhanced creation operations (additional methods beyond DemoApiClient)
  createSyncJob(syncJob: Omit<DemoSyncJob, 'id' | 'createdAt'>): Promise<DemoApiResponse<DemoSyncJob>>;
  createWebhook(webhook: Omit<DemoWebhook, 'id' | 'createdAt'>): Promise<DemoApiResponse<DemoWebhook>>;
  deleteWebhook(id: string): Promise<void>;

// Note: Creation methods return complete entities with server-generated ID and timestamps
// Response format: { data: { id: string, createdAt: string, ...inputFields }, meta: {...} }
}

// Factory functions
function getSharedBackendClient(): SharedBackendClient;
function createSharedBackendClient(config?: Partial<SharedBackendClientConfig>): SharedBackendClient;
```

**Usage Examples:**
```typescript
// Default client (uses demo configuration)
const client = getSharedBackendClient();

// Custom client with specific configuration
const customClient = createSharedBackendClient({
  timeout: 10000,
  retry: {
    maxAttempts: 3,
    baseDelay: 1000,
    maxDelay: 10000,
    multiplier: 2,
    jitter: true
  },
  enableLogging: true,
  logLevel: 'debug'
});

// Making API calls
const providers = await client.getProviders();
const connections = await client.getConnections();
```

#### Scenario: Client usage across components
Given multiple components need to make API calls to the Connectors service
When they each call `getSharedBackendClient()`
Then they should receive the same singleton client instance
And the client should maintain consistent configuration and state across the application
And should handle concurrent requests properly without race conditions

#### Scenario: Mode-aware client behavior
Given the demo is configured in either mock or real mode
When components make API calls through the shared client
Then the client should automatically route requests to the appropriate implementation
And should not require components to be aware of the underlying mode
And should provide consistent error handling and response formats across modes

### Requirement: Authentication and Authorization
The system SHALL provide proper authentication support for real API calls while maintaining mock authentication for demo mode.

#### Scenario: Tenant-scoped API requests
Given a user is authenticated with a specific tenant context
When the shared client makes API calls to the Connectors service
Then it should automatically include the `X-Tenant-Id` header in all requests
And should use the appropriate authentication token for the tenant
And should handle token refresh when necessary

#### Scenario: Bearer token authentication
Given the demo is configured for real API mode with authentication tokens
When the client makes requests to protected endpoints
Then it should include the `Authorization: Bearer <token>` header in the format: `Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...`
And should handle token expiration and refresh scenarios with automatic retry
And should provide clear error messages when authentication fails with specific error codes

**Authentication Header Format:**
```http
Authorization: Bearer <JWT_TOKEN>
X-Tenant-Id: <TENANT_ID>
```

**Token Refresh Workflow:**
1. Request fails with 401 Unauthorized or 403 Forbidden
2. Client attempts token refresh using refresh token endpoint
3. If refresh succeeds, retry original request with new token
4. If refresh fails, return authentication error to caller

**Token Refresh Endpoint Specification:**
- **Path**: `/api/v1/auth/refresh`
- **Method**: `POST`
- **Headers**:
  - `Content-Type: application/json`
  - `X-Tenant-Id: <tenant_id>`
- **Request Body**:
  ```json
  {
    "refreshToken": "string"
  }
  ```
- **Success Response** (200 OK):
  ```json
  {
    "accessToken": "string",
    "tokenType": "Bearer",
    "expiresIn": 3600,
    "refreshToken": "string" // optional - may be rotated
  }
  ```
- **Error Response** (401 Unauthorized):
  ```json
  {
    "code": "AUTH_005",
    "message": "Token refresh failed",
    "details": {
      "tokenType": "refresh"
    }
  }
  ```

**Authentication Error Codes:**
- `AUTH_001`: Missing or invalid Authorization header
- `AUTH_002`: Expired token (automatic refresh attempted)
- `AUTH_003`: Invalid token signature
- `AUTH_004`: Insufficient permissions for requested resource
- `AUTH_005`: Token refresh failed (requires re-authentication)

**Network Error Codes:**
- `NET_001`: Network timeout - request exceeded configured timeout duration
- `NET_002`: Connection refused - API service is unreachable or not accepting connections
- `NET_003`: DNS resolution failure - unable to resolve API service hostname
- `NET_004`: Rate limit exceeded - too many requests, retry after specified delay
- `NET_005`: Server error - API service returned 5xx error, request may be retried

**Error Response Format:**
```typescript
interface AuthenticationError {
  code: 'AUTH_001' | 'AUTH_002' | 'AUTH_003' | 'AUTH_004' | 'AUTH_005';
  message: string;
  details?: {
    tokenType: 'access' | 'refresh';
    expiresAt?: string;
    permissions?: string[];
  };
}

interface NetworkError {
  code: 'NET_001' | 'NET_002' | 'NET_003' | 'NET_004' | 'NET_005';
  message: string;
  details?: {
    timeout?: number;          // For NET_001
    retryAfter?: number;       // For NET_004
    statusCode?: number;       // For NET_005
    hostname?: string;         // For NET_003
    endpoint?: string;         // Request endpoint that failed
  };
}

type SharedBackendError = AuthenticationError | NetworkError;
```

#### Scenario: Mock authentication for demo mode
Given the demo is running in mock mode
When authentication-related operations are performed
Then the client should simulate authentication flows without real tokens
And should generate realistic mock authentication responses
And should provide educational annotations about real authentication processes

### Requirement: Error Handling and Resilience
The system SHALL provide comprehensive error handling with retry logic, rate limiting, and graceful fallbacks.

#### Scenario: API request failures with retry logic
Given an API request fails due to transient network issues or server errors
When the shared client encounters the failure
Then it should implement exponential backoff retry logic with the following default configuration:
And should retry up to a configurable maximum number of attempts (default: 3)
And should use a base delay of 1000ms with exponential multiplier of 2
And should cap the maximum delay at 10000ms to prevent excessive wait times
And should add jitter (random ±25% variation) to prevent thundering herd
And should provide clear error messages after exhausting retries

**Default Retry Configuration:**
```typescript
const defaultRetryConfig = {
  maxAttempts: 3,
  baseDelay: 1000,      // 1 second
  maxDelay: 10000,      // 10 seconds
  multiplier: 2,        // Exponential backoff
  jitter: true          // Add random variation
};
```

**Retryable Error Conditions:**
- Network timeouts (HTTP 408)
- Server errors (HTTP 5xx)
- Rate limit exceeded (HTTP 429) with Retry-After header
- Connection refused/timeout errors
- Temporary DNS resolution failures

#### Scenario: Rate limiting and circuit breaking
Given the Connectors API returns rate limit errors or becomes unavailable
When the shared client detects these conditions
Then it should implement client-side rate limiting to prevent API abuse with the following defaults:
And should allow 10 requests per second with burst capacity of 20 requests
And should use circuit breaking patterns to fail fast when the API is consistently unavailable
And should trigger circuit breaker after 5 consecutive failures within a 60-second monitoring period
And should keep circuit open for 30 seconds before attempting recovery
And should provide fallback to mock data when real API is unavailable

**Default Rate Limiting Configuration:**
```typescript
const defaultRateLimitConfig = {
  requestsPerSecond: 10,    // Sustained rate
  burstCapacity: 20         // Short burst capacity
};
```

**Default Circuit Breaker Configuration:**
```typescript
const defaultCircuitBreakerConfig = {
  failureThreshold: 5,      // Open circuit after 5 failures
  recoveryTimeout: 30000,   // Wait 30 seconds before retry
  monitoringPeriod: 60000   // Monitor failures in 60-second windows
};
```

**Circuit Breaker States:**
- **CLOSED**: Normal operation, all requests pass through
- **OPEN**: All requests fail fast, no API calls made
- **HALF-OPEN**: Limited requests allowed to test recovery

#### Scenario: Graceful fallback to mock data
Given the demo is configured for real mode but the API is unavailable
When API calls fail after retry attempts
Then the client should automatically fall back to mock data generation
And should notify users that they're seeing simulated data due to API unavailability
And should maintain application functionality despite API failures

**Fallback Triggers:**
- Network errors NET_001, NET_002, NET_003 after exhausting retry attempts
- Circuit breaker is OPEN (preventing cascading failures)
- Authentication errors AUTH_005 (token refresh failure)
- Server errors NET_005 when configured for fallback mode
- Configurable timeout duration for API unavailability

**Fallback Behavior:**
- Automatic switch to mock data generation without application restart
- Preserve user notification of fallback mode with clear messaging
- Maintain consistent response formats between real and mock modes
- Optionally cache failed real responses to retry when API becomes available
- Log fallback events for debugging and monitoring

**User Notification:**
- Display clear indicator when showing fallback data: "🟡 Using simulated data due to API unavailability"
- Provide retry mechanism to attempt reconnection to real API
- Include timestamp of last successful API call
- Offer configuration option to disable automatic fallback

### Requirement: Request/Response Interceptors and Monitoring
The system SHALL provide request/response interceptors for logging, monitoring, and debugging capabilities.

#### Scenario: Request logging and debugging
Given a developer is debugging API interactions in the demo
When the shared client makes requests to the Connectors API
Then it should log request details including URL, headers, and payload
And should include request timing and duration information
And should provide configurable log levels for development vs production

#### Scenario: Response monitoring and metrics
Given the application needs to monitor API performance and health
When the shared client receives responses from the Connectors API
Then it should collect metrics including response times, success rates, and error patterns
And should expose these metrics through a monitoring interface
And should provide performance alerts for degraded API behavior

#### Scenario: Educational annotations and insights
Given a user is learning about the Connectors API through the demo
When API calls are made and responses are received
Then the client should provide educational annotations about what's happening
And should explain the real-world implications of each API interaction
And should map mock behavior to actual API concepts for better understanding

### Requirement: Configuration and Customization
The system SHALL provide flexible configuration options for the shared backend client while maintaining sensible defaults.

#### Scenario: Client configuration overrides
Given a developer needs to customize the client behavior for specific use cases
When they create or configure the shared client
Then they should be able to override default settings like timeout values, retry limits, and rate limits
And should be able to provide custom headers and authentication mechanisms
And should have access to configuration validation and error handling

#### Scenario: Environment-specific configurations
Given the demo runs in different environments (development, staging, production)
When the shared client is initialized
Then it should automatically apply environment-appropriate configurations
And should validate that required environment variables are present
And should provide helpful error messages for missing or invalid configurations

#### Scenario: Runtime configuration updates
Given the demo configuration changes during runtime (e.g., switching between mock and real modes)
When the shared client detects configuration changes
Then it should gracefully update its behavior without requiring application restart
And should maintain in-flight requests with their original configuration
And should provide clear feedback about configuration changes

### Requirement: Type Safety and Developer Experience
The system SHALL provide excellent TypeScript support and developer experience features.

#### Scenario: Full TypeScript coverage
Given a developer is using the shared client in their TypeScript code
When they access client methods and responses
Then they should have complete type safety with proper interface definitions
And should receive helpful autocompletion and compile-time error checking
And should have access to comprehensive JSDoc documentation

#### Scenario: Intuitive API design
Given a developer is learning to use the shared client
When they explore the client interface and methods
Then they should find an intuitive and consistent API design
And should have clear method names that map to Connectors API concepts
And should receive helpful error messages for common usage mistakes

#### Scenario: Development tools and debugging support
Given a developer is building with the shared client
When they need to debug or optimize their API interactions
Then they should have access to development tools and debugging utilities
And should be able to inspect request/response cycles and client state
And should receive performance recommendations and optimization suggestions