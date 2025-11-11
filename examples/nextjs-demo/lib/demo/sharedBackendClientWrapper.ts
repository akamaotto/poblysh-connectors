/**
 * Shared Backend Client Wrapper
 *
 * This module provides a wrapper that implements the DemoApiClient interface
 * using the SharedBackendClient singleton, ensuring backward compatibility while
 * providing enhanced features for real API mode without leaking internal state.
 */

import { DemoApiClient } from "./apiRouter";
import {
  DemoProvider,
  DemoConnection,
  DemoSignal,
  DemoGroundedSignal,
  DemoSyncJob,
  DemoWebhook,
  DemoToken,
  DemoRateLimit,
  DemoApiResponse,
} from "./types";
import {
  SharedBackendClient,
  SharedBackendClientConfig,
  getSharedBackendClient,
  createSharedBackendClient,
} from "./sharedBackendClient";

/**
 * Wrapper class that implements DemoApiClient interface using SharedBackendClient.
 *
 * This class acts as an adapter between the existing DemoApiClient interface
 * and the new SharedBackendClient implementation, ensuring backward compatibility
 * while providing enhanced functionality.
 */
export class SharedBackendClientWrapper implements DemoApiClient {
  private sharedClient: SharedBackendClient;

  /**
   * Creates a wrapper bound to the shared backend client singleton.
   *
   * - When a config is provided, it updates/initializes the shared client via
   *   createSharedBackendClient(config) and wraps that singleton.
   * - When omitted, it wraps the existing shared client singleton, creating it
   *   from default demo configuration if needed.
   */
  constructor(config?: SharedBackendClientConfig) {
    if (config) {
      this.sharedClient = createSharedBackendClient(config);
    } else {
      this.sharedClient = getSharedBackendClient();
    }
  }

  /**
   * Private constructor overload for tests to inject a concrete SharedBackendClient.
   */
  static fromClientForTests(
    sharedClient: SharedBackendClient,
  ): SharedBackendClientWrapper {
    const wrapper = Object.create(
      SharedBackendClientWrapper.prototype,
    ) as SharedBackendClientWrapper;
    wrapper.sharedClient = sharedClient;
    return wrapper;
  }

  // ============================================================================
  // DemoApiClient Interface Implementation
  // ============================================================================

  /**
   * Gets available providers from the API.
   */
  async getProviders(): Promise<DemoApiResponse<DemoProvider[]>> {
    return this.sharedClient.getProviders();
  }

  /**
   * Gets connections for the current tenant.
   */
  async getConnections(): Promise<DemoApiResponse<DemoConnection[]>> {
    return this.sharedClient.getConnections();
  }

  /**
   * Creates a new connection.
   */
  async createConnection(
    connection: Omit<DemoConnection, "id" | "createdAt">,
  ): Promise<DemoApiResponse<DemoConnection>> {
    return this.sharedClient.createConnection(connection);
  }

  /**
   * Updates an existing connection.
   */
  async updateConnection(
    id: string,
    updates: Partial<DemoConnection>,
  ): Promise<DemoApiResponse<DemoConnection>> {
    return this.sharedClient.updateConnection(id, updates);
  }

  /**
   * Deletes a connection.
   */
  async deleteConnection(id: string): Promise<void> {
    return this.sharedClient.deleteConnection(id);
  }

  /**
   * Gets signals from the API.
   */
  async getSignals(): Promise<DemoApiResponse<DemoSignal[]>> {
    return this.sharedClient.getSignals();
  }

  /**
   * Gets grounded signals from the API.
   */
  async getGroundedSignals(): Promise<DemoApiResponse<DemoGroundedSignal[]>> {
    return this.sharedClient.getGroundedSignals();
  }

  /**
   * Gets sync jobs from the API.
   */
  async getSyncJobs(): Promise<DemoApiResponse<DemoSyncJob[]>> {
    return this.sharedClient.getSyncJobs();
  }

  /**
   * Gets webhooks from the API.
   */
  async getWebhooks(): Promise<DemoApiResponse<DemoWebhook[]>> {
    return this.sharedClient.getWebhooks();
  }

  /**
   * Gets tokens from the API.
   */
  async getTokens(): Promise<DemoApiResponse<DemoToken[]>> {
    return this.sharedClient.getTokens();
  }

  /**
   * Gets rate limits from the API.
   */
  async getRateLimits(): Promise<DemoApiResponse<DemoRateLimit[]>> {
    return this.sharedClient.getRateLimits();
  }

  // ============================================================================
  // ENHANCED METHODS - Access to Additional SharedBackendClient Features
  // ============================================================================

  /**
   * Gets the underlying SharedBackendClient instance for advanced features.
   *
   * This provides access to enhanced methods like getCurrentUser(), refreshToken(),
   * createSyncJob(), createWebhook(), deleteWebhook(), and monitoring utilities.
   */
  getSharedClient(): SharedBackendClient {
    return this.sharedClient;
  }

  /**
   * Updates the configuration of the underlying shared client.
   */
  updateConfig(config: Partial<SharedBackendClientConfig>): void {
    this.sharedClient.updateConfig(config);
  }

  /**
   * Gets metrics from the underlying shared client.
   */
  getMetrics() {
    return this.sharedClient.getMetrics();
  }

  /**
   * Gets circuit breaker state from the underlying shared client.
   */
  getCircuitBreakerState() {
    return this.sharedClient.getCircuitBreakerState();
  }

  /**
   * Resets the circuit breaker in the underlying shared client.
   */
  resetCircuitBreaker(): void {
    this.sharedClient.resetCircuitBreaker();
  }
}

/**
 * Creates a SharedBackendClientWrapper with the provided configuration.
 *
 * @param config - Configuration options for the shared client
 * @returns A configured SharedBackendClientWrapper instance
 */
export function createSharedBackendClientWrapper(
  config?: SharedBackendClientConfig,
): SharedBackendClientWrapper {
  return new SharedBackendClientWrapper(config);
}
