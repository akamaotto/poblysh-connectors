/**
 * Shared Backend Client - Refactored with Type Safety
 *
 * This is a refactored version of the SharedBackendClient that eliminates `any` and `undefined`
 * usage by using Result and Option types for robust error handling and null safety.
 */

import {
  Option,
  AppResult,
  AppError,
  Ok,
  Err,
  Some,
  None,
  isOk,
  isErr,
  map,
  match,
  matchOption,
  fromNullable,
} from '../types/functional';

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
} from '../types';

// Helper type for API response mapping
type ApiResponseData<T> = DemoApiResponse<T>;

// ============================================================================
// TYPE-SAFE CONFIGURATION
// ============================================================================

interface SafeClientConfig {
  baseUrl: Option<string>;
  authToken: Option<string>;
  tenantId: string;
  timeout: number;
  enableLogging: boolean;
  logLevel: 'debug' | 'info' | 'warn' | 'error';
}

interface SafeRetryConfig {
  maxAttempts: number;
  baseDelay: number;
  maxDelay: number;
  multiplier: number;
  jitter: boolean;
}

interface SafeRateLimitConfig {
  requestsPerSecond: number;
  burstCapacity: number;
}

interface SafeCircuitBreakerConfig {
  failureThreshold: number;
  recoveryTimeout: number;
  monitoringPeriod: number;
}

// ============================================================================
// TYPE-SAFE API CLIENT
// ============================================================================

export class SafeSharedBackendClient {
  private config: SafeClientConfig;
  private retryConfig: SafeRetryConfig;
  private rateLimitConfig: SafeRateLimitConfig;
  private circuitBreakerConfig: SafeCircuitBreakerConfig;
  private circuitState: 'closed' | 'open' | 'half-open' = 'closed';
  private failureCount = 0;
  private lastFailureTime = 0;
  private tokens = 0;
  private lastRefill = 0;

  constructor(config: Partial<SafeClientConfig> = {}) {
    // Set safe defaults
    const defaultConfig: SafeClientConfig = {
      baseUrl: None,
      authToken: None,
      tenantId: 'demo-tenant',
      timeout: 30000,
      enableLogging: true,
      logLevel: 'info',
    };

    this.config = { ...defaultConfig, ...config };
    this.retryConfig = {
      maxAttempts: 3,
      baseDelay: 1000,
      maxDelay: 10000,
      multiplier: 2,
      jitter: true,
    };
    this.rateLimitConfig = {
      requestsPerSecond: 10,
      burstCapacity: 20,
    };
    this.circuitBreakerConfig = {
      failureThreshold: 5,
      recoveryTimeout: 30000,
      monitoringPeriod: 60000,
    };

    this.tokens = this.rateLimitConfig.burstCapacity;
    this.lastRefill = Date.now();

    this.logInfo('SafeSharedBackendClient initialized');
  }

  // ============================================================================
  // PRIVATE HELPER METHODS
  // ============================================================================

  private logInfo(message: string, data?: unknown): void {
    if (!this.config.enableLogging) return;
    console.log(`[SafeSharedBackendClient] ${message}`, data);
  }

  private logError(message: string, error?: unknown): void {
    if (!this.config.enableLogging) return;
    console.error(`[SafeSharedBackendClient] ${message}`, error);
  }

  private logWarn(message: string, data?: unknown): void {
    if (!this.config.enableLogging) return;
    console.warn(`[SafeSharedBackendClient] ${message}`, data || '');
  }

  private getBaseUrl(): string {
    return matchOption<string, string>({
      Some: (url) => url,
      None: () => {
        this.logWarn('No base URL configured, using default', {
          fallback: 'https://api.connectors.example.com'
        });
        return 'https://api.connectors.example.com';
      },
    })(this.config.baseUrl);
  }

  private getAuthToken(): Option<string> {
    return this.config.authToken;
  }

  private refillTokens(): void {
    const now = Date.now();
    const elapsed = now - this.lastRefill;
    if (elapsed <= 0) return;

    const tokensToAdd = (elapsed / 1000) * this.rateLimitConfig.requestsPerSecond;
    this.tokens = Math.min(
      this.rateLimitConfig.burstCapacity,
      this.tokens + tokensToAdd
    );
    this.lastRefill = now;
  }

  private checkRateLimit(): AppResult<void> {
    this.refillTokens();

    if (this.tokens < 1) {
      return Err({
        _tag: 'NetworkError',
        message: 'Rate limit exceeded, retry after 1 second',
        statusCode: 429,
      } as AppError);
    }

    this.tokens -= 1;
    return Ok(undefined);
  }

  private checkCircuitBreaker(): AppResult<void> {
    const now = Date.now();

    if (this.circuitState === 'open') {
      if (now - this.lastFailureTime >= this.circuitBreakerConfig.recoveryTimeout) {
        this.circuitState = 'half-open';
        this.logInfo('Circuit breaker transitioning to half-open');
      } else {
        return Err({
          _tag: 'NetworkError',
          message: 'Circuit breaker is open, service temporarily unavailable',
          statusCode: 503,
        } as AppError);
      }
    }

    // Reset failure count if monitoring period has passed
    if (now - this.lastFailureTime > this.circuitBreakerConfig.monitoringPeriod) {
      this.failureCount = 0;
    }

    if (this.failureCount >= this.circuitBreakerConfig.failureThreshold) {
      this.circuitState = 'open';
      this.lastFailureTime = now;
      this.logError('Circuit breaker opened due to failure threshold', {
        failureCount: this.failureCount,
      });

      return Err({
        _tag: 'NetworkError',
        message: 'Circuit breaker opened due to failure threshold',
        statusCode: 503,
      } as AppError);
    }

    return Ok(undefined);
  }

  private recordSuccess(): void {
    if (this.circuitState !== 'closed') {
      this.logInfo('Request succeeded, closing circuit breaker');
    }
    this.circuitState = 'closed';
    this.failureCount = 0;
  }

  private recordFailure(): void {
    this.failureCount++;
    this.lastFailureTime = Date.now();
    this.logWarn('Recorded failure for circuit breaker', {
      failureCount: this.failureCount,
    });
  }

  private sleep(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }

  private async retryWithBackoff<T>(
    operation: () => Promise<AppResult<T>>
  ): Promise<AppResult<T>> {
    let lastError: AppError | null = null;

    for (let attempt = 0; attempt < this.retryConfig.maxAttempts; attempt++) {
      try {
        const result = await operation();
        if (isOk(result)) {
          if (attempt > 0) {
            this.logInfo(`Operation succeeded after ${attempt} retries`);
          }
          return result;
        }

        lastError = result.error;

        // Don't retry on certain error types
        if (result.error._tag === 'ValidationError' ||
            result.error._tag === 'AuthenticationError') {
          return result;
        }

        if (attempt === this.retryConfig.maxAttempts - 1) {
          this.logError(`Operation failed after ${this.retryConfig.maxAttempts} attempts`, lastError);
          return result;
        }

        const base = Math.min(
          this.retryConfig.maxDelay,
          this.retryConfig.baseDelay * Math.pow(this.retryConfig.multiplier, attempt)
        );

        const jitterFactor = this.retryConfig.jitter
          ? 1 + (Math.random() * 0.5 - 0.25)
          : 1;
        const delay = Math.max(0, base * jitterFactor);

        this.logWarn(`Retrying operation in ${Math.round(delay)}ms (attempt ${attempt + 1}/${this.retryConfig.maxAttempts})`);

        await this.sleep(delay);
      } catch (error) {
        lastError = {
          _tag: 'NetworkError',
          message: error instanceof Error ? error.message : 'Unknown error during retry',
        };

        if (attempt === this.retryConfig.maxAttempts - 1) {
          return Err(lastError);
        }

        await this.sleep(1000);
      }
    }

    return lastError ? Err(lastError) : Err({
      _tag: 'NetworkError',
      message: 'Operation failed without specific error',
    });
  }

  private async makeRequest<T>(
    method: string,
    endpoint: string,
    data?: unknown,
    options: RequestInit = {}
  ): Promise<AppResult<DemoApiResponse<T>>> {
    // Pre-request checks
    const rateLimitCheck = this.checkRateLimit();
    if (isErr(rateLimitCheck)) {
      this.recordFailure();
      return rateLimitCheck;
    }

    const circuitCheck = this.checkCircuitBreaker();
    if (isErr(circuitCheck)) {
      this.recordFailure();
      return circuitCheck;
    }

    const url = `${this.getBaseUrl()}${endpoint}`;
    const authToken = this.getAuthToken();

    // Build headers safely
    const headers = new Headers();
    headers.set('Content-Type', 'application/json');
    headers.set('X-Tenant-Id', this.config.tenantId);
    headers.set('X-Request-Id', `req-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`);

    matchOption({
      Some: (token) => headers.set('Authorization', `Bearer ${token}`),
      None: () => {
        this.logWarn('No auth token provided, request may fail');
      },
    })(authToken);

    // Add custom headers
    if (options.headers) {
      const customHeaders = options.headers as Record<string, string>;
      Object.entries(customHeaders).forEach(([key, value]) => {
        headers.set(key, value);
      });
    }

    // Create timeout signal
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), this.config.timeout);

    try {
      this.logInfo(`Making ${method} request to ${endpoint}`);

      const response = await fetch(url, {
        method,
        headers,
        body: data ? JSON.stringify(data) : undefined,
        signal: controller.signal,
        ...options,
      });

      clearTimeout(timeoutId);

      if (!response.ok) {
        this.recordFailure();
        return Err({
          _tag: 'NetworkError',
          message: `HTTP ${response.status}: ${response.statusText}`,
          statusCode: response.status,
        });
      }

      this.recordSuccess();

      // Handle response data
      let responseData: T;
      if (response.status === 204) {
        responseData = null as T;
      } else {
        responseData = await response.json() as T;
      }

      // Extract pagination headers
      const pagination = response.headers.get('X-Total-Count') ? {
        page: parseInt(response.headers.get('X-Page') ?? '1'),
        limit: parseInt(response.headers.get('X-Limit') ?? '50'),
        total: parseInt(response.headers.get('X-Total-Count') ?? '0'),
        totalPages: parseInt(response.headers.get('X-Total-Pages') ?? '1'),
      } : undefined;

      return Ok({
        data: responseData,
        meta: {
          requestId: headers.get('X-Request-Id') ?? '',
          timestamp: new Date().toISOString(),
          pagination,
        },
      });

    } catch (error) {
      clearTimeout(timeoutId);
      this.recordFailure();

      if (error instanceof Error) {
        if (error.name === 'AbortError') {
          return Err({
            _tag: 'NetworkError',
            message: `Request timeout after ${this.config.timeout}ms`,
            statusCode: 408,
          } as AppError);
        }

        return Err({
          _tag: 'NetworkError',
          message: error.message,
        } as AppError);
      }

      return Err({
        _tag: 'NetworkError',
        message: 'Unknown network error occurred',
      } as AppError);
    }
  }

  // ============================================================================
  // PUBLIC API METHODS
  // ============================================================================

  async getProviders(): Promise<AppResult<DemoProvider[]>> {
    return this.retryWithBackoff(async () => {
      const result = await this.makeRequest<DemoProvider[]>('GET', '/api/v1/providers');
      const mappedResult = map((response: ApiResponseData<DemoProvider[]>) => response.data)(result);
      return mappedResult as AppResult<DemoProvider[]>;
    });
  }

  async getConnections(): Promise<AppResult<DemoConnection[]>> {
    return this.retryWithBackoff(async () => {
      const result = await this.makeRequest<DemoConnection[]>('GET', '/api/v1/connections');
      const mappedResult = map((response: ApiResponseData<DemoConnection[]>) => response.data)(result);
      return mappedResult as AppResult<DemoConnection[]>;
    });
  }

  async createConnection(
    connection: Omit<DemoConnection, 'id' | 'createdAt'>
  ): Promise<AppResult<DemoConnection>> {
    return this.retryWithBackoff(async () => {
      const result = await this.makeRequest<DemoConnection>(
        'POST',
        '/api/v1/connections',
        connection
      );
      const mappedResult = map((response: ApiResponseData<DemoConnection>) => response.data)(result);
      return mappedResult as AppResult<DemoConnection>;
    });
  }

  async updateConnection(
    id: string,
    updates: Partial<DemoConnection>
  ): Promise<AppResult<DemoConnection>> {
    return this.retryWithBackoff(async () => {
      const result = await this.makeRequest<DemoConnection>(
        'PATCH',
        `/api/v1/connections/${id}`,
        updates
      );
      const mappedResult = map((response: ApiResponseData<DemoConnection>) => response.data)(result);
      return mappedResult as AppResult<DemoConnection>;
    });
  }

  async deleteConnection(id: string): Promise<AppResult<void>> {
    return this.retryWithBackoff(async () => {
      const result = await this.makeRequest<null>(
        'DELETE',
        `/api/v1/connections/${id}`
      );
      const mappedResult = map(() => undefined)(result);
      return mappedResult as AppResult<void>;
    });
  }

  async getSignals(): Promise<AppResult<DemoSignal[]>> {
    return this.retryWithBackoff(async () => {
      const result = await this.makeRequest<DemoSignal[]>('GET', '/api/v1/signals');
      const mappedResult = map((response: ApiResponseData<DemoSignal[]>) => response.data)(result);
      return mappedResult as AppResult<DemoSignal[]>;
    });
  }

  async getGroundedSignals(): Promise<AppResult<DemoGroundedSignal[]>> {
    return this.retryWithBackoff(async () => {
      const result = await this.makeRequest<DemoGroundedSignal[]>(
        'GET',
        '/api/v1/signals/grounded'
      );
      const mappedResult = map((response: ApiResponseData<DemoGroundedSignal[]>) => response.data)(result);
      return mappedResult as AppResult<DemoGroundedSignal[]>;
    });
  }

  async getSyncJobs(): Promise<AppResult<DemoSyncJob[]>> {
    return this.retryWithBackoff(async () => {
      const result = await this.makeRequest<DemoSyncJob[]>('GET', '/api/v1/sync-jobs');
      const mappedResult = map((response: ApiResponseData<DemoSyncJob[]>) => response.data)(result);
      return mappedResult as AppResult<DemoSyncJob[]>;
    });
  }

  async getWebhooks(): Promise<AppResult<DemoWebhook[]>> {
    return this.retryWithBackoff(async () => {
      const result = await this.makeRequest<DemoWebhook[]>('GET', '/api/v1/webhooks');
      const mappedResult = map((response: ApiResponseData<DemoWebhook[]>) => response.data)(result);
      return mappedResult as AppResult<DemoWebhook[]>;
    });
  }

  async getTokens(): Promise<AppResult<DemoToken[]>> {
    return this.retryWithBackoff(async () => {
      const result = await this.makeRequest<DemoToken[]>('GET', '/api/v1/tokens');
      const mappedResult = map((response: ApiResponseData<DemoToken[]>) => response.data)(result);
      return mappedResult as AppResult<DemoToken[]>;
    });
  }

  async getRateLimits(): Promise<AppResult<DemoRateLimit[]>> {
    return this.retryWithBackoff(async () => {
      const result = await this.makeRequest<DemoRateLimit[]>('GET', '/api/v1/rate-limits');
      const mappedResult = map((response: ApiResponseData<DemoRateLimit[]>) => response.data)(result);
      return mappedResult as AppResult<DemoRateLimit[]>;
    });
  }

  async getCurrentUser(): Promise<AppResult<DemoUser>> {
    return this.retryWithBackoff(async () => {
      const result = await this.makeRequest<DemoUser>('GET', '/api/v1/auth/me');
      const mappedResult = map((response: ApiResponseData<DemoUser>) => response.data)(result);
      return mappedResult as AppResult<DemoUser>;
    });
  }

  async refreshToken(): Promise<AppResult<DemoToken>> {
    return this.retryWithBackoff(async () => {
      const result = await this.makeRequest<DemoToken>('POST', '/api/v1/auth/refresh');
      const mappedResult = map((response: ApiResponseData<DemoToken>) => response.data)(result);
      return mappedResult as AppResult<DemoToken>;
    });
  }

  // ============================================================================
  // CONFIGURATION AND MONITORING
  // ============================================================================

  updateConfig(newConfig: Partial<SafeClientConfig>): void {
    this.config = { ...this.config, ...newConfig };
    this.logInfo('Configuration updated');
  }

  getHealthStatus(): {
    circuitBreaker: { state: string; failureCount: number };
    rateLimit: { availableTokens: number; maxTokens: number };
  } {
    return {
      circuitBreaker: {
        state: this.circuitState,
        failureCount: this.failureCount,
      },
      rateLimit: {
        availableTokens: this.tokens,
        maxTokens: this.rateLimitConfig.burstCapacity,
      },
    };
  }

  resetCircuitBreaker(): void {
    this.circuitState = 'closed';
    this.failureCount = 0;
    this.lastFailureTime = 0;
    this.logInfo('Circuit breaker reset to closed state');
  }
}

// ============================================================================
// FACTORY FUNCTIONS
// ============================================================================

export const createSafeSharedBackendClient = (
  config?: Partial<SafeClientConfig>
): SafeSharedBackendClient => {
  return new SafeSharedBackendClient(config);
};

export const getDefaultSafeClient = (): SafeSharedBackendClient => {
  return new SafeSharedBackendClient({
    baseUrl: fromNullable(process.env.CONNECTORS_API_BASE_URL),
    authToken: fromNullable(process.env.CONNECTORS_API_TOKEN),
    tenantId: process.env.CONNECTORS_TENANT_ID || 'demo-tenant',
    timeout: parseInt(process.env.CONNECTORS_API_TIMEOUT || '30000', 10),
    enableLogging: process.env.NODE_ENV !== 'test',
    logLevel: (process.env.LOG_LEVEL as 'error' | 'warn' | 'info' | 'debug') || 'info',
  });
};

// ============================================================================
// USAGE EXAMPLES
// ============================================================================

/**
 * Example usage in a service or component
 */
export const exampleUsage = async () => {
  const client = createSafeSharedBackendClient({
    baseUrl: Some('https://api.example.com'),
    authToken: Some('your-token-here'),
    tenantId: 'your-tenant-id',
  });

  // Safe API calls with error handling
  const providersResult = await client.getProviders();
  const providers = match({
    Ok: (data: DemoProvider[]) => data,
    Err: (error: AppError) => {
      const message = error._tag === 'NotFoundError'
        ? `Resource not found: ${error.resource} with id ${error.id}`
        : 'message' in error ? error.message
        : 'Unknown error occurred';
      console.error('Failed to fetch providers:', message);
      return [];
    },
  })(providersResult);

  console.log(`Found ${providers.length} providers`);

  // Safe connection creation
  const connectionResult = await client.createConnection({
    tenantId: 'demo-tenant',
    providerSlug: 'github',
    displayName: 'GitHub Connection',
    status: 'disconnected',
  });

  match({
    Ok: (connection: DemoConnection) => {
      console.log('Connection created:', connection.id);
    },
    Err: (error: AppError) => {
      const message = error._tag === 'NotFoundError'
        ? `Resource not found: ${error.resource} with id ${error.id}`
        : 'message' in error ? error.message
        : 'Unknown error occurred';
      console.error('Failed to create connection:', message);
      if (error._tag === 'ValidationError') {
        console.error('Validation failed for field:', error.field);
      }
    },
  })(connectionResult);
};