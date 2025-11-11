/**
 * Shared Backend Client for Next.js Demo
 *
 * This module provides a production-ready, shared backend client that implements
 * the DemoApiClient interface with proper error handling, authentication, rate limiting,
 * and resilience features for real API mode while maintaining backward compatibility.
 */

import {
  DemoProvider,
  DemoConnection,
  DemoSignal,
  DemoGroundedSignal,
  DemoSyncJob,
  DemoWebhook,
  DemoToken,
  DemoRateLimit,
  DemoUser,
  DemoApiResponse,
  DemoApiError,
} from "./types";
import { getDemoConfig } from "./demoConfig";

// ============================================================================
// TENANT RESPONSE TYPES
// ============================================================================

/**
 * Response data structure for tenant creation API calls.
 */
interface CreateTenantResponseData {
  id: string;
  name: string;
  created_at: string;
  updated_at?: string;
  metadata?: Record<string, unknown>;
}

/**
 * Response data structure for tenant retrieval API calls.
 */
interface TenantResponseData {
  id: string;
  name: string;
  created_at: string;
  updated_at?: string;
  metadata?: Record<string, unknown>;
}

// ============================================================================
// ERROR TYPES - Re-exported from types module
// ============================================================================

/**
 * Union error type used internally for strict error handling.
 */
export type SharedBackendError = AuthenticationError | NetworkError;

// ============================================================================
// CONFIGURATION AND INTERFACES
// ============================================================================

/**
 * Configuration options for the shared backend client.
 */
export interface RetryConfig {
  /** Maximum number of retry attempts */
  maxAttempts: number;
  /** Base delay in milliseconds for exponential backoff */
  baseDelay: number;
  /** Maximum delay in milliseconds */
  maxDelay: number;
  /** Multiplier for exponential backoff */
  multiplier: number;
  /** Whether to apply jitter (+/- 25%) */
  jitter: boolean;
}

export interface RateLimitConfig {
  /** Sustained requests per second */
  requestsPerSecond: number;
  /** Burst capacity for short spikes */
  burstCapacity: number;
}

export interface CircuitBreakerConfig {
  /** Number of failures in monitoring period to open circuit */
  failureThreshold: number;
  /** Time in milliseconds to keep circuit open before half-open */
  recoveryTimeout: number;
  /** Monitoring period window in milliseconds */
  monitoringPeriod: number;
}

/**
 * Configuration options for the shared backend client.
 */
export interface SharedBackendClientConfig {
  /** Base URL for the Connectors API */
  baseUrl?: string;
  /** Authentication token for API requests */
  authToken?: string;
  /** Tenant ID for multi-tenant requests */
  tenantId?: string;
  /** Request timeout in milliseconds */
  timeout?: number;
  /** Retry configuration */
  retry?: RetryConfig;
  /** Rate limiting configuration */
  rateLimit?: RateLimitConfig;
  /** Circuit breaker configuration */
  circuitBreaker?: CircuitBreakerConfig;
  /** Enable request/response logging */
  enableLogging?: boolean;
  /** Log level for debugging */
  logLevel?: "debug" | "info" | "warn" | "error";
  /** Enable educational annotations for demo mode */
  enableEducationalAnnotations?: boolean;
}

/**
 * Authentication error types.
 */
export interface AuthenticationError {
  code: "AUTH_001" | "AUTH_002" | "AUTH_003" | "AUTH_004" | "AUTH_005";
  message: string;
  details?: {
    tokenType?: "access" | "refresh";
    expiresAt?: string;
    permissions?: string[];
  };
}

/**
 * Network error types.
 */
export interface NetworkError {
  code: "NET_001" | "NET_002" | "NET_003" | "NET_004" | "NET_005";
  message: string;
  details?: {
    timeout?: number;
    retryAfter?: number;
    statusCode?: number;
    hostname?: string;
    endpoint?: string;
  };
}

/**
 * Circuit breaker states.
 */
type CircuitState = "CLOSED" | "OPEN" | "HALF_OPEN";

/**
 * Rate limiter state.
 */
interface RateLimiterState {
  /** Available tokens in the bucket */
  tokens: number;
  /** Last refill timestamp (ms) */
  lastRefill: number;
  /** Last request timestamp (ms) */
  lastRequestTime: number;
  /** Number of requests in current window */
  requestCount: number;
}

/**
 * Circuit breaker state.
 */
interface CircuitBreakerState {
  state: CircuitState;
  /** Number of failures in current monitoring window */
  failureCount: number;
  /** Timestamp of last failure (ms) */
  lastFailureTime: number;
  /** Number of consecutive successes */
  successCount: number;
}

/**
 * Request metrics for monitoring.
 */
export interface RequestMetrics {
  /** Request ID for tracking */
  requestId: string;
  /** HTTP method used */
  method: string;
  /** API endpoint called */
  endpoint: string;
  /** Request start timestamp */
  startTime: number;
  /** Request end timestamp */
  endTime: number;
  /** Total request duration in milliseconds */
  duration: number;
  /** Number of retry attempts */
  retryCount: number;
  /** Whether the request was successful */
  success: boolean;
  /** Error message (if any) */
  error?: string;
  /** HTTP status code */
  statusCode?: number;
}

// ============================================================================
// ERROR CODES AND MESSAGES
// ============================================================================

/** Authentication error codes and messages */
export const AUTH_ERRORS = {
  AUTH_001: {
    code: "AUTH_001" as const,
    message: "Authentication token is missing or invalid",
    recovery:
      "Provide a valid authentication token in the Authorization header",
  },
  AUTH_002: {
    code: "AUTH_002" as const,
    message: "Authentication token has expired",
    recovery: "Refresh the authentication token and retry the request",
  },
  AUTH_003: {
    code: "AUTH_003" as const,
    message: "Insufficient permissions for this operation",
    recovery: "Check that the token has the required scopes for this endpoint",
  },
  AUTH_004: {
    code: "AUTH_004" as const,
    message: "Tenant ID is required for this operation",
    recovery: "Provide a valid tenant ID in the X-Tenant-Id header",
  },
  AUTH_005: {
    code: "AUTH_005" as const,
    message: "Invalid tenant context for this request",
    recovery: "Ensure the tenant ID matches your authorized tenant context",
  },
} as const;

/** Network error codes and messages */
export const NETWORK_ERRORS = {
  NET_001: {
    code: "NET_001" as const,
    message: "Network timeout occurred",
    recovery: "Check your network connection and retry the request",
  },
  NET_002: {
    code: "NET_002" as const,
    message: "API server is unavailable",
    recovery: "The service is temporarily unavailable, please try again later",
  },
  NET_003: {
    code: "NET_003" as const,
    message: "DNS resolution failure",
    recovery: "Verify DNS configuration or try again later",
  },
  NET_004: {
    code: "NET_004" as const,
    message: "Rate limit exceeded",
    recovery:
      "Wait before making additional requests or respect Retry-After header",
  },
  NET_005: {
    code: "NET_005" as const,
    message: "Circuit breaker is open",
    recovery: "The service is experiencing issues, automatic retry will occur",
  },
} as const;

// ============================================================================
// SHARED BACKEND CLIENT CLASS
// ============================================================================

/**
 * Production-ready shared backend client with comprehensive error handling,
 * authentication, rate limiting, and resilience features.
 */
export class SharedBackendClient {
  private config: Required<SharedBackendClientConfig>;
  private rateLimiter: RateLimiterState;
  private circuitBreaker: CircuitBreakerState;
  private metrics: RequestMetrics[] = [];
  private isInitialized: boolean = false;

  constructor(config: SharedBackendClientConfig = {}) {
    // Set default configuration
    const defaultRetry: RetryConfig = {
      maxAttempts: 3,
      baseDelay: 1000,
      maxDelay: 10000,
      multiplier: 2,
      jitter: true,
    };

    const defaultRateLimit: RateLimitConfig = {
      requestsPerSecond: 10,
      burstCapacity: 20,
    };

    const defaultCircuitBreaker: CircuitBreakerConfig = {
      failureThreshold: 5,
      recoveryTimeout: 30000,
      monitoringPeriod: 60000,
    };

    this.config = {
      baseUrl: config.baseUrl || this.getDefaultBaseUrl(),
      authToken: config.authToken || this.getDefaultAuthToken(),
      tenantId: config.tenantId || "demo-tenant",
      timeout: config.timeout || 30000,
      retry: { ...defaultRetry, ...(config.retry ?? {}) },
      rateLimit: { ...defaultRateLimit, ...(config.rateLimit ?? {}) },
      circuitBreaker: {
        ...defaultCircuitBreaker,
        ...(config.circuitBreaker ?? {}),
      },
      enableLogging: config.enableLogging ?? true,
      logLevel: config.logLevel ?? "info",
      enableEducationalAnnotations: config.enableEducationalAnnotations ?? true,
    };

    // Initialize rate limiter and circuit breaker
    this.rateLimiter = {
      tokens: this.config.rateLimit.burstCapacity,
      lastRefill: Date.now(),
      lastRequestTime: 0,
      requestCount: 0,
    };

    this.circuitBreaker = {
      state: "CLOSED",
      failureCount: 0,
      lastFailureTime: 0,
      successCount: 0,
    };

    this.isInitialized = true;
    this.logInfo("SharedBackendClient initialized", {
      baseUrl: this.config.baseUrl,
      tenantId: this.config.tenantId,
      timeout: this.config.timeout,
      retry: this.config.retry,
      rateLimit: this.config.rateLimit,
      circuitBreaker: this.config.circuitBreaker,
      logLevel: this.config.logLevel,
    });
  }

  // ============================================================================
  // PRIVATE UTILITY METHODS
  // ============================================================================

  private getDefaultBaseUrl(): string {
    const config = getDemoConfig();
    if (config.mode === "real" && config.connectorsApiBaseUrl) {
      return config.connectorsApiBaseUrl;
    }
    // Return a default test URL for tests and fallback
    return "https://api.connectors.example.com";
  }

  private getDefaultAuthToken(): string {
    // Check environment variables first for real tokens
    if (process.env.CONNECTORS_API_TOKEN) {
      return process.env.CONNECTORS_API_TOKEN;
    }

    // Check demo config for token
    const config = getDemoConfig();
    if (config.connectorsApiToken) {
      return config.connectorsApiToken;
    }

    // For demo purposes, return a mock token with a clear identifier
    return "demo-auth-token-" + Date.now();
  }

  private generateRequestId(): string {
    return `req-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
  }

  private logInfo(message: string, data?: unknown): void {
    if (!this.config.enableLogging) return;

    const safeData =
      data && typeof data === "object"
        ? {
            ...(data as Record<string, unknown>),
            // Never log raw tokens
            authToken: undefined,
          }
        : data;

    this.logWithLevel("info", message, safeData);
  }

  private logError(message: string, error?: unknown): void {
    if (!this.config.enableLogging) return;
    this.logWithLevel("error", message, error);
  }

  private logEducational(message: string, details?: unknown): void {
    if (!this.config.enableEducationalAnnotations) return;
    this.logWithLevel("info", `📚 [EDUCATIONAL] ${message}`, details);
  }

  private refillTokens(now: number): void {
    const { requestsPerSecond, burstCapacity } = this.config.rateLimit;
    const elapsed = now - this.rateLimiter.lastRefill;
    if (elapsed <= 0) return;

    const tokensToAdd = (elapsed / 1000) * requestsPerSecond;
    this.rateLimiter.tokens = Math.min(
      burstCapacity,
      this.rateLimiter.tokens + tokensToAdd,
    );
    this.rateLimiter.lastRefill = now;
  }

  private checkRateLimit(endpoint: string): void {
    const now = Date.now();
    this.refillTokens(now);

    if (this.rateLimiter.tokens < 1) {
      const error: NetworkError = {
        code: "NET_004",
        message: NETWORK_ERRORS.NET_004.message,
        details: {
          retryAfter: 1,
          endpoint,
        },
      };
      this.logWarn("Rate limit exceeded", error);
      throw this.toError(error, this.generateRequestId());
    }

    this.rateLimiter.tokens -= 1;
    this.rateLimiter.lastRequestTime = now;
    this.rateLimiter.requestCount += 1;
  }

  private checkCircuitBreaker(): void {
    const now = Date.now();
    const { failureThreshold, recoveryTimeout, monitoringPeriod } =
      this.config.circuitBreaker;

    if (this.circuitBreaker.state === "OPEN") {
      if (now - this.circuitBreaker.lastFailureTime >= recoveryTimeout) {
        this.circuitBreaker.state = "HALF_OPEN";
        this.logInfo("Circuit breaker transitioning to HALF_OPEN");
      } else {
        const error: NetworkError = {
          code: "NET_005",
          message: NETWORK_ERRORS.NET_005.message,
          details: {
            endpoint: "circuit-breaker",
          },
        };
        throw this.toError(error, this.generateRequestId());
      }
    }

    // Decay failure count over monitoring period
    if (now - this.circuitBreaker.lastFailureTime > monitoringPeriod) {
      this.circuitBreaker.failureCount = 0;
    }

    if (this.circuitBreaker.failureCount >= failureThreshold) {
      this.circuitBreaker.state = "OPEN";
      this.logError("Circuit breaker forced OPEN due to failure threshold", {
        failureCount: this.circuitBreaker.failureCount,
      });
      const error: NetworkError = {
        code: "NET_005",
        message: NETWORK_ERRORS.NET_005.message,
        details: {
          endpoint: "circuit-breaker",
        },
      };
      throw this.toError(error, this.generateRequestId());
    }
  }

  private recordSuccess(): void {
    if (this.circuitBreaker.state !== "CLOSED") {
      this.logInfo("Successful request; closing circuit if half-open");
    }
    this.circuitBreaker.state = "CLOSED";
    this.circuitBreaker.failureCount = 0;
    this.circuitBreaker.successCount += 1;
  }

  private recordFailure(): void {
    this.circuitBreaker.failureCount++;
    this.circuitBreaker.lastFailureTime = Date.now();
    this.logWarn("Recorded failure for circuit breaker", {
      failureCount: this.circuitBreaker.failureCount,
    });
  }

  private async sleep(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }

  private async retryWithBackoff<T>(
    operation: () => Promise<T>,
    metricsTracker?: { retryCount: number; method?: string; endpoint?: string; requestId?: string; startTime?: number },
  ): Promise<T> {
    const { maxAttempts, baseDelay, maxDelay, multiplier, jitter } =
      this.config.retry;
    let lastError: Error | null = null;

    for (let attempt = 0; attempt < maxAttempts; attempt++) {
      try {
        const result = await operation();
        if (attempt > 0) {
          this.logInfo(`Operation succeeded after ${attempt} retries`);
          if (metricsTracker) metricsTracker.retryCount = attempt;
        }
        return result;
      } catch (error) {
        lastError = error as Error;

        // Record failure metrics for non-retryable errors
        if (!this.isRetryableError(lastError) || attempt === maxAttempts - 1) {
          this.recordFailure();
          if (metricsTracker) {
            metricsTracker.retryCount = attempt;
            // Record metrics if we have the required information
            if (metricsTracker.method && metricsTracker.endpoint && metricsTracker.requestId && metricsTracker.startTime) {
              const endTime = Date.now();
              const duration = endTime - metricsTracker.startTime;
              const failureMetrics: RequestMetrics = {
                requestId: metricsTracker.requestId,
                method: metricsTracker.method,
                endpoint: metricsTracker.endpoint,
                startTime: metricsTracker.startTime,
                endTime,
                duration,
                retryCount: attempt,
                success: false,
                error: lastError.message,
              };
              this.metrics.push(failureMetrics);
            }
          }
        }

        if (!this.isRetryableError(lastError)) {
          if (metricsTracker) metricsTracker.retryCount = attempt;
          throw lastError;
        }

        if (attempt === maxAttempts - 1) {
          this.logError(
            `Operation failed after ${maxAttempts} attempts`,
            lastError,
          );
          if (metricsTracker) metricsTracker.retryCount = attempt;
          throw lastError;
        }

        const base = Math.min(
          maxDelay,
          baseDelay * Math.pow(multiplier, attempt),
        );
        const jitterFactor = jitter ? 1 + (Math.random() * 0.5 - 0.25) : 1;
        const delay = Math.max(0, Math.min(maxDelay, base * jitterFactor));

        this.logWarn(
          `Retrying operation in ${Math.round(
            delay,
          )}ms (attempt ${attempt + 1}/${maxAttempts})`,
          { lastError: lastError.message },
        );
        if (metricsTracker) metricsTracker.retryCount = attempt;
        await this.sleep(delay);
      }
    }

    throw (
      lastError ?? new Error("RetryWithBackoff failed without error context")
    );
  }

  private isRetryableError(error: Error): boolean {
    const message = error.message.toLowerCase();

    // Authentication and validation errors are not retryable
    if (
      message.includes("auth") ||
      message.includes("unauthorized") ||
      message.includes("forbidden") ||
      message.includes("validation")
    ) {
      return false;
    }

    // Retry network/timeout/server/rate-limit issues
    return (
      message.includes("network") ||
      message.includes("timeout") ||
      message.includes("server") ||
      message.includes("rate limit") ||
      message.includes("temporarily unavailable")
    );
  }

  private async makeRequest<T>(
    method: string,
    endpoint: string,
    data?: unknown,
    options: RequestInit = {},
    requestId?: string,
    startTime?: number,
  ): Promise<DemoApiResponse<T>> {
    const id: string = requestId || this.generateRequestId();
    const start = startTime || Date.now();
    const metricsTracker = { retryCount: 0 };

    this.logDebug(`Making ${method} request to ${endpoint}`, { requestId: id });

    // Check circuit breaker and rate limiting before performing request
    this.checkCircuitBreaker();
    this.checkRateLimit(endpoint);

    
    const url = `${this.config.baseUrl}${endpoint}`;

    const defaultHeaders = {
      "Content-Type": "application/json",
      Authorization: `Bearer ${this.config.authToken}`,
      "X-Tenant-Id": this.config.tenantId,
      "X-Request-Id": id,
    };

    // Create timeout signal (AbortSignal.timeout is not available in older environments)
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), this.config.timeout);

    let response: Response;
    try {
      const fetchResult = await fetch(url, {
        method,
        headers: {
          ...defaultHeaders,
          ...options.headers,
        },
        body: data ? JSON.stringify(data) : undefined,
        signal: controller.signal,
      });

      // Handle case where mock fetch returns undefined
      if (!fetchResult) {
        throw new Error('Fetch returned undefined - possible mock configuration issue');
      }

      response = fetchResult;
    } catch (error) {
      clearTimeout(timeoutId);
      throw error;
    } finally {
      clearTimeout(timeoutId);
    }

    const endTime = Date.now();
    const duration = endTime - start;

    // Record metrics
    const metrics: RequestMetrics = {
      requestId: id,
      method,
      endpoint,
      startTime: start,
      endTime,
      duration,
      retryCount: metricsTracker.retryCount,
      success: response.ok,
      statusCode: response.status,
    };

    if (!response.ok) {
      const error = await this.handleError(response, id, endpoint);
      this.recordFailure();
      metrics.success = false;
      metrics.error = error.message;
      this.metrics.push(metrics);
      throw error;
    }

    this.recordSuccess();
    this.logEducational(`Request completed successfully`, {
      requestId: id,
      duration: `${duration}ms`,
      statusCode: response.status,
    });

    let responseData;
    try {
      // Handle 204 No Content responses
      if (response.status === 204) {
        responseData = null;
      } else {
        responseData = await response.json();
      }
    } catch {
      const error = new Error(
        "Invalid JSON response",
      ) as unknown as DemoApiError;
      error.code = "NET_004";
      error.message = "Invalid JSON response";
      error.requestId = id;
      error.timestamp = new Date().toISOString();
      this.recordFailure();
      metrics.error = error.message;
      this.metrics.push(metrics);
      throw error;
    }

    this.metrics.push(metrics);

    // Return appropriate response based on method and status
    if (method === "DELETE" && response.status === 204) {
      // For DELETE with 204, return empty response
      return {
        data: null as T,
        meta: {
          requestId: id,
          timestamp: new Date().toISOString(),
        },
      };
    }

    return {
      data: responseData,
      meta: {
        requestId: id,
        timestamp: new Date().toISOString(),
        ...(response.headers.get("X-Total-Count") && {
          pagination: {
            page: parseInt(response.headers.get("X-Page") ?? "1"),
            limit: parseInt(response.headers.get("X-Limit") ?? "50"),
            total: parseInt(response.headers.get("X-Total-Count") ?? "0"),
            totalPages: parseInt(response.headers.get("X-Total-Pages") ?? "1"),
          },
        }),
      },
    };
  }

  private async handleError(
    response: Response,
    requestId: string,
    endpoint: string,
  ): Promise<Error> {
    let errorData: Record<string, unknown>;

    try {
      errorData = await response.json();
    } catch {
      errorData = { message: "Unknown error" };
    }

    const errorMessage =
      (errorData.message as string) ||
      `HTTP ${response.status}: ${response.statusText}`;

    // Handle specific error types
    if (response.status === 401) {
      const authError: AuthenticationError = {
        code: (errorData.code as AuthenticationError["code"]) || "AUTH_001",
        message: errorMessage,
        details: {
          tokenType: "access",
        },
      };
      return this.toError(authError, requestId);
    }

    if (response.status === 403) {
      const authError: AuthenticationError = {
        code: (errorData.code as AuthenticationError["code"]) || "AUTH_003",
        message: errorMessage,
        details: {
          tokenType: "access",
        },
      };
      return this.toError(authError, requestId);
    }

    if (response.status >= 500) {
      const netError: NetworkError = {
        code: "NET_002",
        message: errorMessage,
        details: {
          statusCode: response.status,
          endpoint,
        },
      };
      return this.toError(netError, requestId);
    }

    const generic: NetworkError = {
      code: "NET_005",
      message: errorMessage,
      details: {
        statusCode: response.status,
        endpoint,
      },
    };
    return this.toError(generic, requestId);
  }

  // ============================================================================
  // API METHODS - DemoApiClient Interface Implementation
  // ============================================================================

  async getProviders(): Promise<DemoApiResponse<DemoProvider[]>> {
    const requestId = this.generateRequestId();
    const startTime = Date.now();
    return this.retryWithBackoff(async () => {
      return this.makeRequest<DemoProvider[]>("GET", "/api/v1/providers", undefined, {}, requestId, startTime);
    }, {
      retryCount: 0,
      method: "GET",
      endpoint: "/api/v1/providers",
      requestId,
      startTime,
    });
  }

  async getConnections(): Promise<DemoApiResponse<DemoConnection[]>> {
    return this.retryWithBackoff(async () => {
      return this.makeRequest<DemoConnection[]>("GET", "/api/v1/connections");
    });
  }

  async createConnection(
    connection: Omit<DemoConnection, "id" | "createdAt">,
  ): Promise<DemoApiResponse<DemoConnection>> {
    return this.retryWithBackoff(async () => {
      return this.makeRequest<DemoConnection>(
        "POST",
        "/api/v1/connections",
        connection,
      );
    });
  }

  async updateConnection(
    id: string,
    updates: Partial<DemoConnection>,
  ): Promise<DemoApiResponse<DemoConnection>> {
    return this.retryWithBackoff(async () => {
      return this.makeRequest<DemoConnection>(
        `PATCH`,
        `/api/v1/connections/${id}`,
        updates,
      );
    });
  }

  async deleteConnection(id: string): Promise<void> {
    return this.retryWithBackoff(async () => {
      await this.makeRequest("DELETE", `/api/v1/connections/${id}`);
      // Return undefined for DELETE operations
      return undefined;
    });
  }

  async getSignals(): Promise<DemoApiResponse<DemoSignal[]>> {
    return this.retryWithBackoff(async () => {
      return this.makeRequest<DemoSignal[]>("GET", "/api/v1/signals");
    });
  }

  async getGroundedSignals(): Promise<DemoApiResponse<DemoGroundedSignal[]>> {
    return this.retryWithBackoff(async () => {
      return this.makeRequest<DemoGroundedSignal[]>(
        "GET",
        "/api/v1/signals/grounded",
      );
    });
  }

  async getSyncJobs(): Promise<DemoApiResponse<DemoSyncJob[]>> {
    return this.retryWithBackoff(async () => {
      return this.makeRequest<DemoSyncJob[]>("GET", "/api/v1/sync-jobs");
    });
  }

  async getWebhooks(): Promise<DemoApiResponse<DemoWebhook[]>> {
    return this.retryWithBackoff(async () => {
      return this.makeRequest<DemoWebhook[]>("GET", "/api/v1/webhooks");
    });
  }

  async getTokens(): Promise<DemoApiResponse<DemoToken[]>> {
    return this.retryWithBackoff(async () => {
      return this.makeRequest<DemoToken[]>("GET", "/api/v1/tokens");
    });
  }

  async getRateLimits(): Promise<DemoApiResponse<DemoRateLimit[]>> {
    return this.retryWithBackoff(async () => {
      return this.makeRequest<DemoRateLimit[]>("GET", "/api/v1/rate-limits");
    });
  }

  /**
   * Creates a new tenant
   */
  async createTenant(tenant: {name: string, metadata?: object}): Promise<DemoApiResponse<CreateTenantResponseData>> {
    return this.retryWithBackoff(async () => {
      return this.makeRequest<CreateTenantResponseData>("POST", "/api/v1/tenants", tenant);
    });
  }

  /**
   * Gets a tenant by ID.
   */
  async getTenant(tenantId: string): Promise<DemoApiResponse<TenantResponseData>> {
    return this.retryWithBackoff(async () => {
      return this.makeRequest<TenantResponseData>("GET", `/api/v1/tenants/${tenantId}`);
    });
  }

  // ============================================================================
  // ENHANCED METHODS - Additional Features Beyond DemoApiClient
  // ============================================================================

  /**
   * Gets current user information.
   */
  async getCurrentUser(): Promise<DemoApiResponse<DemoUser>> {
    return this.retryWithBackoff(async () => {
      return this.makeRequest<DemoUser>("GET", "/api/v1/auth/me");
    });
  }

  /**
   * Refreshes the authentication token.
   */
  async refreshToken(): Promise<DemoApiResponse<DemoToken>> {
    return this.retryWithBackoff(async () => {
      return this.makeRequest<DemoToken>("POST", "/api/v1/auth/refresh");
    });
  }

  /**
   * Creates a new sync job.
   */
  async createSyncJob(
    syncJob: Omit<DemoSyncJob, "id" | "createdAt">,
  ): Promise<DemoApiResponse<DemoSyncJob>> {
    return this.retryWithBackoff(async () => {
      return this.makeRequest<DemoSyncJob>(
        "POST",
        "/api/v1/sync-jobs",
        syncJob,
      );
    });
  }

  /**
   * Creates a new webhook.
   */
  async createWebhook(
    webhook: Omit<DemoWebhook, "id" | "createdAt">,
  ): Promise<DemoApiResponse<DemoWebhook>> {
    return this.retryWithBackoff(async () => {
      return this.makeRequest<DemoWebhook>("POST", "/api/v1/webhooks", webhook);
    });
  }

  /**
   * Deletes a webhook.
   */
  async deleteWebhook(id: string): Promise<void> {
    return this.retryWithBackoff(async () => {
      await this.makeRequest("DELETE", `/api/v1/webhooks/${id}`);
      // Return undefined for DELETE operations
      return undefined;
    });
  }

  // ============================================================================
  // MONITORING AND UTILITIES
  // ============================================================================

  /**
   * Gets request metrics for monitoring.
   */
  getMetrics(): RequestMetrics[] {
    return [...this.metrics];
  }

  /**
   * Gets current circuit breaker state.
   */
  getCircuitBreakerState(): CircuitBreakerState {
    return { ...this.circuitBreaker };
  }

  /**
   * Gets current rate limiter state.
   */
  getRateLimiterState(): RateLimiterState {
    return { ...this.rateLimiter };
  }

  /**
   * Clears all metrics.
   */
  clearMetrics(): void {
    this.metrics = [];
  }

  /**
   * Resets the circuit breaker to closed state.
   */
  resetCircuitBreaker(): void {
    this.circuitBreaker = {
      state: "CLOSED",
      failureCount: 0,
      lastFailureTime: 0,
      successCount: 0,
    };
    this.logInfo("Circuit breaker manually reset to CLOSED state");
  }

  /**
   * Updates the client configuration.
   */
  updateConfig(newConfig: Partial<SharedBackendClientConfig>): void {
    const merged: SharedBackendClientConfig = {
      ...this.config,
      ...newConfig,
    };

    // Re-apply strong typing for nested configs
    this.config = {
      ...this.config,
      baseUrl: merged.baseUrl ?? this.config.baseUrl,
      authToken: merged.authToken ?? this.config.authToken,
      tenantId: merged.tenantId ?? this.config.tenantId,
      timeout: merged.timeout ?? this.config.timeout,
      retry: { ...this.config.retry, ...(merged.retry ?? {}) },
      rateLimit: { ...this.config.rateLimit, ...(merged.rateLimit ?? {}) },
      circuitBreaker: {
        ...this.config.circuitBreaker,
        ...(merged.circuitBreaker ?? {}),
      },
      enableLogging: merged.enableLogging ?? this.config.enableLogging,
      logLevel: merged.logLevel ?? this.config.logLevel,
      enableEducationalAnnotations:
        merged.enableEducationalAnnotations ??
        this.config.enableEducationalAnnotations,
    };

    this.logInfo("Client configuration updated", {
      timeout: this.config.timeout,
      retry: this.config.retry,
      rateLimit: this.config.rateLimit,
      circuitBreaker: this.config.circuitBreaker,
      logLevel: this.config.logLevel,
    });
  }

  // ========================================================================
  // INTERNAL LOGGING + ERROR HELPERS
  // ========================================================================

  private logWithLevel(
    level: "debug" | "info" | "warn" | "error",
    message: string,
    data?: unknown,
  ): void {
    const allowedLevels: Array<"debug" | "info" | "warn" | "error"> = [
      "debug",
      "info",
      "warn",
      "error",
    ];
    const currentIndex = allowedLevels.indexOf(this.config.logLevel);
    const levelIndex = allowedLevels.indexOf(level);

    if (levelIndex < currentIndex) return;

    const logger =
      level === "error"
        ? console.error
        : level === "warn"
          ? console.warn
          : console.log;

    if (data !== undefined) {
      logger(`[SharedBackendClient] ${message}`, data);
    } else {
      logger(`[SharedBackendClient] ${message}`);
    }
  }

  private logDebug(message: string, data?: unknown): void {
    this.logWithLevel("debug", message, data);
  }

  private logWarn(message: string, data?: unknown): void {
    this.logWithLevel("warn", message, data);
  }

  private toError(
    err: AuthenticationError | NetworkError,
    requestId?: string,
  ): Error {
    const error = new Error(err.message);
    const apiError = error as Error & DemoApiError;

    apiError.code = err.code;
    if (requestId) {
      apiError.requestId = requestId;
    } else {
      apiError.requestId = "";
    }
    apiError.timestamp = new Date().toISOString();

    if ("details" in err && err.details) {
      apiError.details = err.details;
    }

    return apiError;
  }
}

// ============================================================================
// FACTORY FUNCTIONS
// ============================================================================

let sharedClientInstance: SharedBackendClient | null = null;

/**
 * App/demo-facing factory: returns a shared singleton client configured from
 * the default demo settings.
 */
export function getSharedBackendClient(): SharedBackendClient {
  if (!sharedClientInstance) {
    sharedClientInstance = new SharedBackendClient();
  }
  return sharedClientInstance;
}

/**
 * Explicit/test-facing factory: always returns a new isolated client instance.
 *
 * This avoids hidden shared state between tests and ensures expectations like
 * `toBeInstanceOf(SharedBackendClient)` and per-test cleanup via
 * `clearMetrics` / `resetCircuitBreaker` remain accurate.
 */
export function createSharedBackendClient(
  config?: Partial<SharedBackendClientConfig>,
): SharedBackendClient {
  return new SharedBackendClient(config ?? {});
}

// ============================================================================
// EDUCATIONAL EXPORTS
// ============================================================================

/**
 * Educational: Default configuration values for learning purposes.
 */
export const DEFAULT_CONFIG = {
  timeout: 30000,
  maxRetries: 3,
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
} as const;

/**
 * Educational: Example configuration for different environments.
 */
export const EXAMPLE_CONFIGS = {
  development: {
    timeout: 10000,
    maxRetries: 1,
    enableLogging: true,
    enableEducationalAnnotations: true,
  },
  production: {
    timeout: 30000,
    maxRetries: 5,
    enableLogging: false,
    enableEducationalAnnotations: false,
  },
  testing: {
    timeout: 5000,
    maxRetries: 0,
    enableLogging: true,
    enableEducationalAnnotations: false,
  },
} as const;
