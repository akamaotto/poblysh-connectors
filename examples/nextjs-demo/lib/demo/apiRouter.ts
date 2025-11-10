/**
 * API routing abstraction layer for demo modes.
 * 
 * This module provides a unified interface for data operations that can work
 * with both mock data (generated locally) and real API calls to the Connectors service.
 * The appropriate implementation is selected based on the current demo mode.
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
  DemoApiResponse,
  DemoApiError,
  DemoMode
} from './types';
import { getDemoConfig } from './demoConfig';

// Import mock implementations
import { generateMockProviders } from './mockData';
import { MOCK_DELAYS } from './constants';

// Import real API implementations (these will be implemented separately)
// import * as realApiClient from './realApiClient';

/**
 * Base API client interface that both mock and real implementations must follow.
 */
export interface DemoApiClient {
  // Provider operations
  getProviders(): Promise<DemoApiResponse<DemoProvider[]>>;
  
  // Connection operations
  getConnections(): Promise<DemoApiResponse<DemoConnection[]>>;
  createConnection(connection: Omit<DemoConnection, 'id' | 'createdAt'>): Promise<DemoApiResponse<DemoConnection>>;
  updateConnection(id: string, updates: Partial<DemoConnection>): Promise<DemoApiResponse<DemoConnection>>;
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
}

/**
 * Mock API client implementation.
 * Uses locally generated mock data with realistic delays.
 */
class MockApiClient implements DemoApiClient {
  private async mockDelay(ms?: number): Promise<void> {
    const delay = ms ?? MOCK_DELAYS.NORMAL;
    await new Promise(resolve => setTimeout(resolve, delay));
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

  async createConnection(connection: Omit<DemoConnection, 'id' | 'createdAt'>): Promise<DemoApiResponse<DemoConnection>> {
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

  async updateConnection(id: string, updates: Partial<DemoConnection>): Promise<DemoApiResponse<DemoConnection>> {
    await this.mockDelay();
    
    // In a real implementation, this would update the mock data
    const updatedConnection: DemoConnection = {
      id,
      tenantId: updates.tenantId ?? 'demo-tenant',
      providerSlug: updates.providerSlug ?? 'github',
      displayName: updates.displayName ?? 'Demo Connection',
      status: updates.status ?? 'connected',
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
}

/**
 * Real API client implementation.
 * Makes actual HTTP requests to the Connectors API service.
 */
class RealApiClient implements DemoApiClient {
  private baseUrl: string;
  private tenantId: string;

  constructor(baseUrl: string, tenantId: string = 'demo-tenant') {
    this.baseUrl = baseUrl.replace(/\/$/, ''); // Remove trailing slash
    this.tenantId = tenantId;
  }

  private async makeRequest<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<DemoApiResponse<T>> {
    const url = `${this.baseUrl}${endpoint}`;
    
    const defaultHeaders = {
      'Content-Type': 'application/json',
      'X-Tenant-Id': this.tenantId,
    };

    const response = await fetch(url, {
      ...options,
      headers: {
        ...defaultHeaders,
        ...options.headers,
      },
    });

    if (!response.ok) {
      throw new Error(`API request failed: ${response.status} ${response.statusText}`);
    }

    const data = await response.json();
    
    return {
      data,
      meta: {
        requestId: response.headers.get('X-Request-Id') ?? `real-${Date.now()}`,
        timestamp: new Date().toISOString(),
        // Include pagination info if available in response headers
        ...(response.headers.get('X-Total-Count') && {
          pagination: {
            page: parseInt(response.headers.get('X-Page') ?? '1'),
            limit: parseInt(response.headers.get('X-Limit') ?? '50'),
            total: parseInt(response.headers.get('X-Total-Count') ?? '0'),
            totalPages: parseInt(response.headers.get('X-Total-Pages') ?? '1'),
          },
        }),
      },
    };
  }

  async getProviders(): Promise<DemoApiResponse<DemoProvider[]>> {
    return this.makeRequest<DemoProvider[]>('/api/v1/providers');
  }

  async getConnections(): Promise<DemoApiResponse<DemoConnection[]>> {
    return this.makeRequest<DemoConnection[]>('/api/v1/connections');
  }

  async createConnection(connection: Omit<DemoConnection, 'id' | 'createdAt'>): Promise<DemoApiResponse<DemoConnection>> {
    return this.makeRequest<DemoConnection>('/api/v1/connections', {
      method: 'POST',
      body: JSON.stringify(connection),
    });
  }

  async updateConnection(id: string, updates: Partial<DemoConnection>): Promise<DemoApiResponse<DemoConnection>> {
    return this.makeRequest<DemoConnection>(`/api/v1/connections/${id}`, {
      method: 'PATCH',
      body: JSON.stringify(updates),
    });
  }

  async deleteConnection(id: string): Promise<void> {
    await fetch(`${this.baseUrl}/api/v1/connections/${id}`, {
      method: 'DELETE',
      headers: {
        'X-Tenant-Id': this.tenantId,
      },
    });
  }

  async getSignals(): Promise<DemoApiResponse<DemoSignal[]>> {
    return this.makeRequest<DemoSignal[]>('/api/v1/signals');
  }

  async getGroundedSignals(): Promise<DemoApiResponse<DemoGroundedSignal[]>> {
    return this.makeRequest<DemoGroundedSignal[]>('/api/v1/signals/grounded');
  }

  async getSyncJobs(): Promise<DemoApiResponse<DemoSyncJob[]>> {
    return this.makeRequest<DemoSyncJob[]>('/api/v1/sync-jobs');
  }

  async getWebhooks(): Promise<DemoApiResponse<DemoWebhook[]>> {
    return this.makeRequest<DemoWebhook[]>('/api/v1/webhooks');
  }

  async getTokens(): Promise<DemoApiResponse<DemoToken[]>> {
    return this.makeRequest<DemoToken[]>('/api/v1/tokens');
  }

  async getRateLimits(): Promise<DemoApiResponse<DemoRateLimit[]>> {
    return this.makeRequest<DemoRateLimit[]>('/api/v1/rate-limits');
  }
}

/**
 * API router factory function.
 * Creates the appropriate API client based on the current demo mode.
 * 
 * @returns API client instance for the current mode
 * @throws Error if configuration is invalid for the selected mode
 */
export function createApiClient(): DemoApiClient {
  const config = getDemoConfig();
  
  if (!config.isValid) {
    throw new Error(`Invalid configuration: ${config.errors.join(', ')}`);
  }
  
  switch (config.mode) {
    case 'mock':
      return new MockApiClient();
      
    case 'real':
      if (!config.connectorsApiBaseUrl) {
        throw new Error('Connectors API base URL is required for real mode');
      }
      return new RealApiClient(config.connectorsApiBaseUrl);
      
    default:
      throw new Error(`Unsupported demo mode: ${config.mode}`);
  }
}

/**
 * Convenience function to get the current API client instance.
 * This is the preferred way to access the API client in most cases.
 * 
 * @returns API client instance for the current mode
 */
export function getApiClient(): DemoApiClient {
  return createApiClient();
}

/**
 * Checks if the current mode uses mock data.
 * 
 * @returns true if in mock mode, false otherwise
 */
export function isMockMode(): boolean {
  const config = getDemoConfig();
  return config.mode === 'mock';
}

/**
 * Checks if the current mode uses real API calls.
 * 
 * @returns true if in real mode, false otherwise
 */
export function isRealMode(): boolean {
  const config = getDemoConfig();
  return config.mode === 'real';
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
    isMock: config.mode === 'mock',
    isReal: config.mode === 'real',
    apiBaseUrl: config.connectorsApiBaseUrl,
    isValid: config.isValid,
    errors: config.errors,
    warnings: config.warnings,
  };
}