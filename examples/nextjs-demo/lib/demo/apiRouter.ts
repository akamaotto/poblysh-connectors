/**
 * API routing abstraction layer for demo modes.
 *
 * This module provides a unified interface for data operations that can work
 * with both mock data (generated locally) and real API calls to the Connectors service.
 * A singleton DemoApiClient is used to ensure consistent behavior and to align
 * with the shared backend client OpenSpec.
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
  DemoTenant,
  DemoApiResponse,
} from "./types";
import { getDemoConfig } from "./demoConfig";

// Import mock implementations
import { generateMockProviders } from "./mockData";
import { MOCK_DELAYS } from "./constants";

// Import shared backend client factories and wrapper
import {
  getSharedBackendClient,
  createSharedBackendClient,
} from "./sharedBackendClient";
import { createSharedBackendClientWrapper } from "./sharedBackendClientWrapper";

/**
 * Base API client interface that both mock and real implementations must follow.
 */
export interface DemoApiClient {
  // Provider operations
  getProviders(): Promise<DemoApiResponse<DemoProvider[]>>;

  // Connection operations
  getConnections(): Promise<DemoApiResponse<DemoConnection[]>>;
  createConnection(
    connection: Omit<DemoConnection, "id" | "createdAt">,
  ): Promise<DemoApiResponse<DemoConnection>>;
  updateConnection(
    id: string,
    updates: Partial<DemoConnection>,
  ): Promise<DemoApiResponse<DemoConnection>>;
  deleteConnection(id: string): Promise<void>;

  // Signal operations
  getSignals(): Promise<DemoApiResponse<DemoSignal[]>>;
  getGroundedSignals(): Promise<DemoApiResponse<DemoGroundedSignal[]>>;

  // Sync job operations
  getSyncJobs(): Promise<DemoApiResponse<DemoSyncJob[]>>;

  // Webhook operations
  getWebhooks(): Promise<DemoApiResponse<DemoWebhook[]>>;

  // Token operations
  getTokens(): Promise<DemoApiResponse<DemoToken[]>>;

  // Rate limit operations
  getRateLimits(): Promise<DemoApiResponse<DemoRateLimit[]>>;

  // Tenant operations
  createTenant(
    tenant: { name: string; metadata?: Record<string, unknown> }
  ): Promise<DemoApiResponse<DemoTenant>>;
  getTenant(tenantId: string): Promise<DemoApiResponse<DemoTenant>>;
}

/**
 * Mock API client implementation.
 * Uses locally generated mock data with realistic delays.
 */
class MockApiClient implements DemoApiClient {
  private async mockDelay(ms?: number): Promise<void> {
    const delay = ms ?? MOCK_DELAYS.NORMAL;
    await new Promise((resolve) => setTimeout(resolve, delay));
  }

  async getProviders(): Promise<DemoApiResponse<DemoProvider[]>> {
    await this.mockDelay(MOCK_DELAYS.FAST);

    const providers = generateMockProviders();

    return {
      data: providers,
      meta: {
        requestId: `mock-${Date.now()}`,
        timestamp: new Date().toISOString(),
      },
    };
  }

  async getConnections(): Promise<DemoApiResponse<DemoConnection[]>> {
    await this.mockDelay();

    // In a real implementation, this would use mock data generators
    const connections: DemoConnection[] = [];

    return {
      data: connections,
      meta: {
        requestId: `mock-${Date.now()}`,
        timestamp: new Date().toISOString(),
      },
    };
  }

  async createConnection(
    connection: Omit<DemoConnection, "id" | "createdAt">,
  ): Promise<DemoApiResponse<DemoConnection>> {
    await this.mockDelay(MOCK_DELAYS.SLOW);

    const newConnection: DemoConnection = {
      ...connection,
      id: `conn-${Date.now()}`,
      createdAt: new Date().toISOString(),
    };

    return {
      data: newConnection,
      meta: {
        requestId: `mock-${Date.now()}`,
        timestamp: new Date().toISOString(),
      },
    };
  }

  async updateConnection(
    id: string,
    updates: Partial<DemoConnection>,
  ): Promise<DemoApiResponse<DemoConnection>> {
    await this.mockDelay();

    // In a real implementation, this would update the mock data
    const updatedConnection: DemoConnection = {
      id,
      tenantId: updates.tenantId ?? "demo-tenant",
      providerSlug: updates.providerSlug ?? "github",
      displayName: updates.displayName ?? "Demo Connection",
      status: updates.status ?? "connected",
      createdAt: new Date().toISOString(),
      ...updates,
    };

    return {
      data: updatedConnection,
      meta: {
        requestId: `mock-${Date.now()}`,
        timestamp: new Date().toISOString(),
      },
    };
  }

  async deleteConnection(id: string): Promise<void> {
    await this.mockDelay();
    // Mock implementation - no actual deletion needed
    // In real implementation, id would be used to identify the connection to delete
    void id; // Suppress unused variable warning
  }

  async getSignals(): Promise<DemoApiResponse<DemoSignal[]>> {
    await this.mockDelay(MOCK_DELAYS.SCAN);

    // In a real implementation, this would generate mock signals
    const signals: DemoSignal[] = [];

    return {
      data: signals,
      meta: {
        requestId: `mock-${Date.now()}`,
        timestamp: new Date().toISOString(),
      },
    };
  }

  async getGroundedSignals(): Promise<DemoApiResponse<DemoGroundedSignal[]>> {
    await this.mockDelay(MOCK_DELAYS.GROUND);

    // In a real implementation, this would generate mock grounded signals
    const groundedSignals: DemoGroundedSignal[] = [];

    return {
      data: groundedSignals,
      meta: {
        requestId: `mock-${Date.now()}`,
        timestamp: new Date().toISOString(),
      },
    };
  }

  async getSyncJobs(): Promise<DemoApiResponse<DemoSyncJob[]>> {
    await this.mockDelay();

    // In a real implementation, this would generate mock sync jobs
    const syncJobs: DemoSyncJob[] = [];

    return {
      data: syncJobs,
      meta: {
        requestId: `mock-${Date.now()}`,
        timestamp: new Date().toISOString(),
      },
    };
  }

  async getWebhooks(): Promise<DemoApiResponse<DemoWebhook[]>> {
    await this.mockDelay();

    // In a real implementation, this would generate mock webhooks
    const webhooks: DemoWebhook[] = [];

    return {
      data: webhooks,
      meta: {
        requestId: `mock-${Date.now()}`,
        timestamp: new Date().toISOString(),
      },
    };
  }

  async getTokens(): Promise<DemoApiResponse<DemoToken[]>> {
    await this.mockDelay();

    // In a real implementation, this would generate mock tokens
    const tokens: DemoToken[] = [];

    return {
      data: tokens,
      meta: {
        requestId: `mock-${Date.now()}`,
        timestamp: new Date().toISOString(),
      },
    };
  }

  async getRateLimits(): Promise<DemoApiResponse<DemoRateLimit[]>> {
    await this.mockDelay();

    // In a real implementation, this would generate mock rate limits
    const rateLimits: DemoRateLimit[] = [];

    return {
      data: rateLimits,
      meta: {
        requestId: `mock-${Date.now()}`,
        timestamp: new Date().toISOString(),
      },
    };
  }

  async createTenant(
    tenant: { name: string; metadata?: Record<string, unknown> }
  ): Promise<DemoApiResponse<DemoTenant>> {
    await this.mockDelay(MOCK_DELAYS.SLOW);

    const newTenant: DemoTenant = {
      id: `tenant-${Date.now()}`,
      name: tenant.name,
      slug: tenant.name.toLowerCase().replace(/[^a-z0-9]/g, '-'),
      connectorsTenantId: `conn-${Date.now()}`,
      createdAt: new Date().toISOString(),
      plan: 'free',
    };

    return {
      data: newTenant,
      meta: {
        requestId: `mock-${Date.now()}`,
        timestamp: new Date().toISOString(),
      },
    };
  }

  async getTenant(tenantId: string): Promise<DemoApiResponse<DemoTenant>> {
    await this.mockDelay(MOCK_DELAYS.FAST);

    // Mock implementation - in real would lookup tenant by ID
    const tenant: DemoTenant = {
      id: tenantId,
      name: "Mock Tenant",
      slug: "mock-tenant",
      connectorsTenantId: `conn-${tenantId}`,
      createdAt: new Date().toISOString(),
      plan: 'free',
    };

    return {
      data: tenant,
      meta: {
        requestId: `mock-${Date.now()}`,
        timestamp: new Date().toISOString(),
      },
    };
  }
}

/**
 * NOTE:
 * The previous RealApiClient has been removed in favor of using the
 * spec-compliant SharedBackendClient + SharedBackendClientWrapper.
 * All real-mode behavior should go through those abstractions.
 */

// Singleton instances
let demoApiClientSingleton: DemoApiClient | null = null;

/**
 * Internal factory for the mock client (stateless).
 */
function createMockApiClient(): DemoApiClient {
  return new MockApiClient();
}

/**
 * Internal factory for the real client using the SharedBackendClient wrapper.
 * This always goes through the shared backend client singleton to align with OpenSpec.
 */
function createRealApiClientFromSharedClient(): DemoApiClient {
  const config = getDemoConfig();

  if (!config.connectorsApiBaseUrl) {
    throw new Error("Connectors API base URL is required for real mode");
  }

  const clientConfig = {
    baseUrl: config.connectorsApiBaseUrl,
    tenantId: config.tenantId || "demo-tenant",
    authToken: config.connectorsApiToken,
    timeout: config.apiTimeout || 30000,
    enableLogging: config.enableApiLogging ?? true,
    enableEducationalAnnotations: config.enableEducationalAnnotations ?? true,
  };

  // Ensure shared backend client is configured as a singleton
  const sharedClient = createSharedBackendClient(clientConfig);
  return createSharedBackendClientWrapper({
    baseUrl: sharedClient["config"]?.baseUrl ?? clientConfig.baseUrl,
    tenantId: sharedClient["config"]?.tenantId ?? clientConfig.tenantId,
    authToken: sharedClient["config"]?.authToken ?? clientConfig.authToken,
    timeout: sharedClient["config"]?.timeout ?? clientConfig.timeout,
    enableLogging: clientConfig.enableLogging,
    enableEducationalAnnotations: clientConfig.enableEducationalAnnotations,
  });
}

/**
 * Returns the singleton DemoApiClient for the current configuration.
 * This enforces a single shared client per runtime and aligns with the shared
 * backend client singleton semantics from OpenSpec.
 */
export function getApiClient(): DemoApiClient {
  const config = getDemoConfig();

  if (!config.isValid) {
    throw new Error(`Invalid configuration: ${config.errors.join(", ")}`);
  }

  // If we already have a client and the mode has not changed, reuse it.
  if (!demoApiClientSingleton) {
    if (config.mode === "mock") {
      demoApiClientSingleton = createMockApiClient();
    } else if (config.mode === "real") {
      demoApiClientSingleton = createRealApiClientFromSharedClient();
    } else {
      throw new Error(`Unsupported demo mode: ${config.mode}`);
    }
  }

  return demoApiClientSingleton;
}

/**
 * Gets the shared backend client singleton for enhanced features.
 *
 * This function is spec-compliant:
 * - It always returns a SharedBackendClient instance.
 * - The client internally respects the current demo configuration and mode.
 * - Components can rely on this being a singleton across the app runtime.
 */
export { getSharedBackendClient };

/**
 * Checks if the enhanced shared client features are available.
 *
 * @returns true if in real mode with shared client, false otherwise
 */
export function hasEnhancedFeatures(): boolean {
  const config = getDemoConfig();
  return config.mode === "real";
}

/**
 * Checks if the current mode uses mock data.
 *
 * @returns true if in mock mode, false otherwise
 */
export function isMockMode(): boolean {
  const config = getDemoConfig();
  return config.mode === "mock";
}

/**
 * Checks if the current mode uses real API calls.
 *
 * @returns true if in real mode, false otherwise
 */
export function isRealMode(): boolean {
  const config = getDemoConfig();
  return config.mode === "real";
}

/**
 * Gets information about the current API mode for UI display purposes.
 *
 * @returns Object with mode information
 */
export function getApiModeInfo() {
  const config = getDemoConfig();

  return {
    mode: config.mode,
    isMock: config.mode === "mock",
    isReal: config.mode === "real",
    apiBaseUrl: config.connectorsApiBaseUrl,
    isValid: config.isValid,
    errors: config.errors,
    warnings: config.warnings,
  };
}
