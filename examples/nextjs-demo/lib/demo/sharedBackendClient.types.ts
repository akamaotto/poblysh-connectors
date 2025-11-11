/**
 * Type definitions for the Shared Backend Client
 *
 * This file contains comprehensive TypeScript types, interfaces, and utilities
 * for the shared backend client implementation.
 */

import {
  DemoUser,
  DemoApiResponse
} from './types';

// ============================================================================
// CONFIGURATION TYPES
// ============================================================================

/**
 * Rate limiting configuration for the shared backend client.
 */
export interface RateLimitConfig {
  /** Maximum requests per second allowed */
  requestsPerSecond?: number;
  /** Maximum burst capacity (requests allowed in a short burst) */
  burstCapacity?: number;
}

/**
 * Circuit breaker configuration for resilience.
 */
export interface CircuitBreakerConfig {
  /** Number of failures before opening the circuit */
  failureThreshold?: number;
  /** Time in milliseconds to wait before attempting to close circuit */
  timeout?: number;
  /** Number of successful requests required to close circuit */
  successThreshold?: number;
}

/**
 * Authentication configuration options.
 */
export interface AuthenticationConfig {
  /** Authentication token for API requests */
  token?: string;
  /** Token refresh endpoint */
  refreshEndpoint?: string;
  /** Whether to automatically refresh expired tokens */
  autoRefresh?: boolean;
  /** Time before expiry to attempt refresh (milliseconds) */
  refreshThreshold?: number;
}

/**
 * Comprehensive configuration for the shared backend client.
 */
export interface SharedBackendClientConfig {
  /** Base URL for the Connectors API */
  baseUrl?: string;
  /** Authentication configuration */
  auth?: AuthenticationConfig;
  /** Tenant ID for multi-tenant requests */
  tenantId?: string;
  /** Request timeout in milliseconds */
  timeout?: number;
  /** Maximum number of retry attempts for failed requests */
  maxRetries?: number;
  /** Initial retry delay in milliseconds (exponential backoff) */
  retryDelay?: number;
  /** Rate limiting configuration */
  rateLimit?: RateLimitConfig;
  /** Circuit breaker configuration */
  circuitBreaker?: CircuitBreakerConfig;
  /** Request/response logging configuration */
  logging?: {
    /** Enable request logging */
    requests?: boolean;
    /** Enable response logging */
    responses?: boolean;
    /** Enable error logging */
    errors?: boolean;
    /** Log request/response bodies (potential security concern) */
    logBodies?: boolean;
  };
  /** Educational features for demo mode */
  educational?: {
    /** Enable educational annotations */
    enableAnnotations?: boolean;
    /** Show performance metrics */
    showMetrics?: boolean;
    /** Include learning explanations */
    includeExplanations?: boolean;
  };
}

// ============================================================================
// ERROR TYPES
// ============================================================================

/**
 * Base error class for shared backend client errors.
 */
export abstract class SharedBackendClientError extends Error {
  /** Error code following the pattern CATEGORY_XXX */
  abstract readonly code: string;
  /** Human-readable error message */
  message: string;
  /** Request ID for tracing */
  requestId?: string;
  /** Error timestamp */
  timestamp: string;
  /** Additional error details */
  details?: Record<string, unknown>;

  constructor(message: string, requestId?: string, details?: Record<string, unknown>) {
    super(message);
    this.name = this.constructor.name;
    this.message = message;
    this.requestId = requestId;
    this.timestamp = new Date().toISOString();
    this.details = details;
  }
}

/**
 * Authentication-related errors.
 */
export class AuthenticationError extends SharedBackendClientError {
  readonly code: 'AUTH_001' | 'AUTH_002' | 'AUTH_003' | 'AUTH_004' | 'AUTH_005';

  constructor(
    code: AuthenticationError['code'],
    message: string,
    requestId?: string,
    details?: Record<string, unknown>
  ) {
    super(message, requestId, details);
    this.code = code;
  }
}

/**
 * Network-related errors.
 */
export class NetworkError extends SharedBackendClientError {
  readonly code: 'NET_001' | 'NET_002' | 'NET_003' | 'NET_004' | 'NET_005';

  constructor(
    code: NetworkError['code'],
    message: string,
    requestId?: string,
    details?: Record<string, unknown>
  ) {
    super(message, requestId, details);
    this.code = code;
  }
}

/**
 * Validation errors for request parameters.
 */
export class ValidationError extends SharedBackendClientError {
  readonly code = 'VAL_001' as const;

  constructor(
    message: string,
    requestId?: string,
    details?: Record<string, unknown>
  ) {
    super(message, requestId, details);
  }
}

/**
 * Configuration errors.
 */
export class ConfigurationError extends SharedBackendClientError {
  readonly code = 'CFG_001' as const;

  constructor(
    message: string,
    requestId?: string,
    details?: Record<string, unknown>
  ) {
    super(message, requestId, details);
  }
}

// ============================================================================
// MONITORING AND METRICS TYPES
// ============================================================================

/**
 * Request-level metrics for monitoring and debugging.
 */
export interface RequestMetrics {
  /** Unique request identifier */
  requestId: string;
  /** HTTP method used */
  method: string;
  /** API endpoint path */
  endpoint: string;
  /** Request start timestamp (Unix epoch) */
  startTime: number;
  /** Request end timestamp (Unix epoch) */
  endTime: number;
  /** Total request duration in milliseconds */
  duration: number;
  /** Number of retry attempts */
  retryCount: number;
  /** Whether the request was ultimately successful */
  success: boolean;
  /** HTTP status code returned */
  statusCode?: number;
  /** Error message (if request failed) */
  error?: string;
  /** Request payload size in bytes */
  requestSize?: number;
  /** Response payload size in bytes */
  responseSize?: number;
  /** Headers sent with request */
  requestHeaders?: Record<string, string>;
  /** Headers received in response */
  responseHeaders?: Record<string, string>;
}

/**
 * Circuit breaker state information.
 */
export interface CircuitBreakerState {
  /** Current circuit state */
  state: 'CLOSED' | 'OPEN' | 'HALF_OPEN';
  /** Number of consecutive failures */
  failureCount: number;
  /** Timestamp of last failure */
  lastFailureTime: number;
  /** Number of consecutive successes */
  successCount: number;
  /** Timestamp of last state change */
  lastStateChange: number;
}

/**
 * Rate limiter state information.
 */
export interface RateLimiterState {
  /** Timestamp of last request */
  lastRequestTime: number;
  /** Number of requests in current time window */
  requestCount: number;
  /** Current time window start */
  windowStart: number;
  /** Remaining requests in current window */
  remainingRequests: number;
  /** Time until next request is allowed (milliseconds) */
  nextRequestAllowedAt: number;
}

/**
 * Aggregated performance metrics.
 */
export interface PerformanceMetrics {
  /** Total number of requests made */
  totalRequests: number;
  /** Number of successful requests */
  successfulRequests: number;
  /** Number of failed requests */
  failedRequests: number;
  /** Success rate as percentage */
  successRate: number;
  /** Average request duration in milliseconds */
  averageDuration: number;
  /** Minimum request duration in milliseconds */
  minDuration: number;
  /** Maximum request duration in milliseconds */
  maxDuration: number;
  /** Requests per second */
  requestsPerSecond: number;
  /** 95th percentile response time */
  p95Duration: number;
  /** 99th percentile response time */
  p99Duration: number;
}

// ============================================================================
// ENHANCED API RESPONSE TYPES
// ============================================================================

/**
 * Enhanced API response with additional metadata.
 */
export interface EnhancedApiResponse<T> extends DemoApiResponse<T> {
  /** Enhanced response metadata */
  meta: DemoApiResponse<T>['meta'] & {
    /** Request processing time in milliseconds */
    processingTime?: number;
    /** Number of retry attempts */
    retryCount?: number;
    /** Whether response was served from cache */
    fromCache?: boolean;
    /** API version used */
    apiVersion?: string;
    /** Rate limit information */
    rateLimit?: {
      /** Requests remaining in current window */
      remaining: number;
      /** Time until rate limit resets */
      resetAt: string;
      /** Maximum requests allowed */
      limit: number;
    };
  };
}

// ============================================================================
// CLIENT STATE TYPES
// ============================================================================

/**
 * Current state of the shared backend client.
 */
export interface ClientState {
  /** Whether the client is initialized and ready */
  isInitialized: boolean;
  /** Current authentication state */
  authentication: {
    /** Whether client is authenticated */
    isAuthenticated: boolean;
    /** Current user information (if authenticated) */
    user?: DemoUser;
    /** Token expiration time */
    tokenExpiresAt?: string;
    /** Whether token refresh is in progress */
    refreshing: boolean;
  };
  /** Current circuit breaker state */
  circuitBreaker: CircuitBreakerState;
  /** Current rate limiter state */
  rateLimiter: RateLimiterState;
  /** Performance metrics */
  performance: PerformanceMetrics;
  /** Recent request metrics */
  recentRequests: RequestMetrics[];
}

// ============================================================================
// UTILITY TYPES
// ============================================================================

/**
 * Type for API endpoint paths.
 */
export type ApiEndpoint =
  | '/api/v1/providers'
  | '/api/v1/connections'
  | '/api/v1/connections/:id'
  | '/api/v1/signals'
  | '/api/v1/signals/grounded'
  | '/api/v1/sync-jobs'
  | '/api/v1/webhooks'
  | '/api/v1/webhooks/:id'
  | '/api/v1/tokens'
  | '/api/v1/rate-limits'
  | '/api/v1/auth/me'
  | '/api/v1/auth/refresh';

/**
 * HTTP methods supported by the client.
 */
export type HttpMethod = 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE';

/**
 * Request options for API calls.
 */
export interface RequestOptions {
  /** Additional headers to include */
  headers?: Record<string, string>;
  /** Request timeout override */
  timeout?: number;
  /** Whether to retry on failure */
  retry?: boolean;
  /** Maximum retry attempts for this request */
  maxRetries?: number;
  /** Whether to bypass rate limiting */
  bypassRateLimit?: boolean;
  /** Whether to bypass circuit breaker */
  bypassCircuitBreaker?: boolean;
}

/**
 * Event types emitted by the shared backend client.
 */
export type ClientEvent =
  | { type: 'request_start'; data: { requestId: string; method: string; endpoint: string } }
  | { type: 'request_success'; data: { requestId: string; duration: number; statusCode: number } }
  | { type: 'request_error'; data: { requestId: string; error: Error; retryCount: number } }
  | { type: 'circuit_breaker_opened'; data: { failureCount: number } }
  | { type: 'circuit_breaker_closed'; data: { successCount: number } }
  | { type: 'rate_limit_hit'; data: { waitTime: number } }
  | { type: 'token_refreshed'; data: { newToken: string } }
  | { type: 'authentication_error'; data: { error: AuthenticationError } };

/**
 * Event listener function type.
 */
export type EventListener<T extends ClientEvent['type']> = (event: Extract<ClientEvent, { type: T }>) => void;

// ============================================================================
// TYPE GUARDS
// ============================================================================

/**
 * Type guard for authentication errors.
 */
export function isAuthenticationError(error: unknown): error is AuthenticationError {
  return error instanceof AuthenticationError;
}

/**
 * Type guard for network errors.
 */
export function isNetworkError(error: unknown): error is NetworkError {
  return error instanceof NetworkError;
}

/**
 * Type guard for validation errors.
 */
export function isValidationError(error: unknown): error is ValidationError {
  return error instanceof ValidationError;
}

/**
 * Type guard for configuration errors.
 */
export function isConfigurationError(error: unknown): error is ConfigurationError {
  return error instanceof ConfigurationError;
}

/**
 * Type guard for shared backend client errors.
 */
export function isSharedBackendClientError(error: unknown): error is SharedBackendClientError {
  return error instanceof SharedBackendClientError;
}

// ============================================================================
// EDUCATIONAL EXPORTS
// ============================================================================

/**
 * Educational: Default error messages and recovery suggestions.
 */
export const ERROR_CATALOG = {
  AUTH_001: {
    message: 'Authentication token is missing or invalid',
    explanation: 'The API requires a valid authentication token to authorize requests.',
    recovery: 'Ensure you provide a valid Bearer token in the Authorization header.',
    example: 'Authorization: Bearer your-auth-token-here'
  },
  AUTH_002: {
    message: 'Authentication token has expired',
    explanation: 'Authentication tokens have a limited lifetime for security reasons.',
    recovery: 'Use the refresh token to get a new access token, or re-authenticate.',
    example: 'POST /api/v1/auth/refresh with refresh token'
  },
  NET_001: {
    message: 'Network timeout occurred',
    explanation: 'The request took too long to complete, likely due to network issues.',
    recovery: 'Check your internet connection and try again. The client will retry automatically.',
    example: 'Wait for automatic retry or check network connectivity'
  },
  NET_005: {
    message: 'Circuit breaker is open',
    explanation: 'Too many failures have occurred, so requests are temporarily blocked.',
    recovery: 'Wait for the circuit breaker to close automatically, or reset it manually.',
    example: 'Circuit will close after 30 seconds of no failures'
  }
} as const;

/**
 * Educational: Configuration examples for different environments.
 */
export const CONFIGURATION_EXAMPLES = {
  development: {
    baseUrl: 'http://localhost:8080',
    timeout: 10000,
    maxRetries: 1,
    logging: {
      requests: true,
      responses: true,
      errors: true,
      logBodies: true
    },
    educational: {
      enableAnnotations: true,
      showMetrics: true,
      includeExplanations: true
    }
  },
  production: {
    timeout: 30000,
    maxRetries: 5,
    logging: {
      requests: false,
      responses: false,
      errors: true,
      logBodies: false
    },
    educational: {
      enableAnnotations: false,
      showMetrics: false,
      includeExplanations: false
    }
  },
  testing: {
    timeout: 5000,
    maxRetries: 0,
    logging: {
      requests: true,
      responses: true,
      errors: true,
      logBodies: false
    },
    educational: {
      enableAnnotations: false,
      showMetrics: false,
      includeExplanations: false
    }
  }
} as const;