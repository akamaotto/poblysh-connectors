/**
 * Tests for SharedBackendClientWrapper
 *
 * These tests verify that the wrapper correctly implements the DemoApiClient
 * interface and delegates to the SharedBackendClient appropriately.
 */

import { describe, it, beforeEach, afterEach, expect, vi } from "bun:test";
import {
  SharedBackendClientWrapper,
  createSharedBackendClientWrapper,
} from "../sharedBackendClientWrapper";
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
} from "../types";
import {
  createSharedBackendClient,
  getSharedBackendClient,
} from "../sharedBackendClient";

const mockSharedClientMethods = {
  getProviders: vi.fn(),
  getConnections: vi.fn(),
  createConnection: vi.fn(),
  updateConnection: vi.fn(),
  deleteConnection: vi.fn(),
  getSignals: vi.fn(),
  getGroundedSignals: vi.fn(),
  getSyncJobs: vi.fn(),
  getWebhooks: vi.fn(),
  getTokens: vi.fn(),
  getRateLimits: vi.fn(),
  getCurrentUser: vi.fn(),
  refreshToken: vi.fn(),
  createSyncJob: vi.fn(),
  createWebhook: vi.fn(),
  deleteWebhook: vi.fn(),
  getMetrics: vi.fn(),
  getCircuitBreakerState: vi.fn(),
  resetCircuitBreaker: vi.fn(),
  updateConfig: vi.fn(),
};

vi.mock("../sharedBackendClient", () => ({
  SharedBackendClient: vi
    .fn()
    .mockImplementation(() => mockSharedClientMethods),
  createSharedBackendClient: vi.fn().mockReturnValue(mockSharedClientMethods),
  getSharedBackendClient: vi.fn().mockReturnValue(mockSharedClientMethods),
}));

describe("SharedBackendClientWrapper", () => {
  let wrapper: SharedBackendClientWrapper;
  let mockSharedClient = mockSharedClientMethods;

  beforeEach(() => {
    vi.clearAllMocks();
    mockSharedClient = mockSharedClientMethods;
    wrapper = SharedBackendClientWrapper.fromClientForTests(mockSharedClient);
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  describe("DemoApiClient Interface Implementation", () => {
    const mockProviders: DemoProvider[] = [
      {
        slug: "github",
        name: "GitHub",
        description: "GitHub integration",
        iconUrl: "/icons/github.svg",
        supportedSignalKinds: [],
        authType: "oauth2",
        webhookEvents: [],
        defaultScopes: [],
        features: {
          realtimeWebhooks: true,
          historicalSync: true,
          incrementalSync: true,
          crossProviderCorrelation: true,
        },
      },
    ];

    const mockConnections: DemoConnection[] = [
      {
        id: "conn-1",
        tenantId: "tenant-1",
        providerSlug: "github",
        displayName: "GitHub Connection",
        status: "connected",
        createdAt: "2024-01-01T00:00:00.000Z",
      },
    ];

    const mockResponse: DemoApiResponse<DemoProvider[]> = {
      data: mockProviders,
      meta: {
        requestId: "req-123",
        timestamp: "2024-01-01T00:00:00.000Z",
      },
    };

    describe("getProviders", () => {
      it("delegates to SharedBackendClient", async () => {
        mockSharedClient.getProviders.mockResolvedValue(mockResponse);

        const result = await wrapper.getProviders();

        expect(mockSharedClient.getProviders).toHaveBeenCalledTimes(1);
        expect(result).toEqual(mockResponse);
      });
    });

    describe("getConnections", () => {
      it("delegates to SharedBackendClient", async () => {
        const mockConnectionsResponse: DemoApiResponse<DemoConnection[]> = {
          data: mockConnections,
          meta: {
            requestId: "req-124",
            timestamp: "2024-01-01T00:00:00.000Z",
          },
        };
        mockSharedClient.getConnections.mockResolvedValue(
          mockConnectionsResponse,
        );

        const result = await wrapper.getConnections();

        expect(mockSharedClient.getConnections).toHaveBeenCalledTimes(1);
        expect(result).toEqual(mockConnectionsResponse);
      });
    });

    describe("createConnection", () => {
      it("delegates to SharedBackendClient", async () => {
        const newConnection: Omit<DemoConnection, "id" | "createdAt"> = {
          tenantId: "tenant-1",
          providerSlug: "github",
          displayName: "New Connection",
          status: "connected",
        };

        const mockCreateResponse: DemoApiResponse<DemoConnection> = {
          data: {
            id: "conn-2",
            createdAt: "2024-01-01T00:00:00.000Z",
            ...newConnection,
          },
          meta: {
            requestId: "req-125",
            timestamp: "2024-01-01T00:00:00.000Z",
          },
        };
        mockSharedClient.createConnection.mockResolvedValue(mockCreateResponse);

        const result = await wrapper.createConnection(newConnection);

        expect(mockSharedClient.createConnection).toHaveBeenCalledTimes(1);
        expect(mockSharedClient.createConnection).toHaveBeenCalledWith(
          newConnection,
        );
        expect(result).toEqual(mockCreateResponse);
      });
    });

    describe("updateConnection", () => {
      it("delegates to SharedBackendClient", async () => {
        const updates = { displayName: "Updated Connection" };
        const mockUpdateResponse: DemoApiResponse<DemoConnection> = {
          data: {
            ...mockConnections[0],
            ...updates,
          },
          meta: {
            requestId: "req-126",
            timestamp: "2024-01-01T00:00:00.000Z",
          },
        };
        mockSharedClient.updateConnection.mockResolvedValue(mockUpdateResponse);

        const result = await wrapper.updateConnection("conn-1", updates);

        expect(mockSharedClient.updateConnection).toHaveBeenCalledTimes(1);
        expect(mockSharedClient.updateConnection).toHaveBeenCalledWith(
          "conn-1",
          updates,
        );
        expect(result).toEqual(mockUpdateResponse);
      });
    });

    describe("deleteConnection", () => {
      it("delegates to SharedBackendClient", async () => {
        mockSharedClient.deleteConnection.mockResolvedValue(undefined);

        await wrapper.deleteConnection("conn-1");

        expect(mockSharedClient.deleteConnection).toHaveBeenCalledTimes(1);
        expect(mockSharedClient.deleteConnection).toHaveBeenCalledWith(
          "conn-1",
        );
      });
    });

    describe("getSignals", () => {
      it("delegates to SharedBackendClient", async () => {
        const mockSignalsResponse: DemoApiResponse<DemoSignal[]> = {
          data: [],
          meta: {
            requestId: "req-127",
            timestamp: "2024-01-01T00:00:00.000Z",
          },
        };
        mockSharedClient.getSignals.mockResolvedValue(mockSignalsResponse);

        const result = await wrapper.getSignals();

        expect(mockSharedClient.getSignals).toHaveBeenCalledTimes(1);
        expect(result).toEqual(mockSignalsResponse);
      });
    });

    describe("getGroundedSignals", () => {
      it("delegates to SharedBackendClient", async () => {
        const mockGroundedSignalsResponse: DemoApiResponse<
          DemoGroundedSignal[]
        > = {
          data: [],
          meta: {
            requestId: "req-128",
            timestamp: "2024-01-01T00:00:00.000Z",
          },
        };
        mockSharedClient.getGroundedSignals.mockResolvedValue(
          mockGroundedSignalsResponse,
        );

        const result = await wrapper.getGroundedSignals();

        expect(mockSharedClient.getGroundedSignals).toHaveBeenCalledTimes(1);
        expect(result).toEqual(mockGroundedSignalsResponse);
      });
    });

    describe("getSyncJobs", () => {
      it("delegates to SharedBackendClient", async () => {
        const mockSyncJobsResponse: DemoApiResponse<DemoSyncJob[]> = {
          data: [],
          meta: {
            requestId: "req-129",
            timestamp: "2024-01-01T00:00:00.000Z",
          },
        };
        mockSharedClient.getSyncJobs.mockResolvedValue(mockSyncJobsResponse);

        const result = await wrapper.getSyncJobs();

        expect(mockSharedClient.getSyncJobs).toHaveBeenCalledTimes(1);
        expect(result).toEqual(mockSyncJobsResponse);
      });
    });

    describe("getWebhooks", () => {
      it("delegates to SharedBackendClient", async () => {
        const mockWebhooksResponse: DemoApiResponse<DemoWebhook[]> = {
          data: [],
          meta: {
            requestId: "req-130",
            timestamp: "2024-01-01T00:00:00.000Z",
          },
        };
        mockSharedClient.getWebhooks.mockResolvedValue(mockWebhooksResponse);

        const result = await wrapper.getWebhooks();

        expect(mockSharedClient.getWebhooks).toHaveBeenCalledTimes(1);
        expect(result).toEqual(mockWebhooksResponse);
      });
    });

    describe("getTokens", () => {
      it("delegates to SharedBackendClient", async () => {
        const mockTokensResponse: DemoApiResponse<DemoToken[]> = {
          data: [],
          meta: {
            requestId: "req-131",
            timestamp: "2024-01-01T00:00:00.000Z",
          },
        };
        mockSharedClient.getTokens.mockResolvedValue(mockTokensResponse);

        const result = await wrapper.getTokens();

        expect(mockSharedClient.getTokens).toHaveBeenCalledTimes(1);
        expect(result).toEqual(mockTokensResponse);
      });
    });

    describe("getRateLimits", () => {
      it("delegates to SharedBackendClient", async () => {
        const mockRateLimitsResponse: DemoApiResponse<DemoRateLimit[]> = {
          data: [],
          meta: {
            requestId: "req-132",
            timestamp: "2024-01-01T00:00:00.000Z",
          },
        };
        mockSharedClient.getRateLimits.mockResolvedValue(
          mockRateLimitsResponse,
        );

        const result = await wrapper.getRateLimits();

        expect(mockSharedClient.getRateLimits).toHaveBeenCalledTimes(1);
        expect(result).toEqual(mockRateLimitsResponse);
      });
    });
  });

  describe("Enhanced Features", () => {
    describe("getSharedClient", () => {
      it("returns the underlying SharedBackendClient instance", () => {
        const sharedClient = wrapper.getSharedClient();

        expect(sharedClient).toBe(mockSharedClient);
      });
    });

    describe("updateConfig", () => {
      it("updates the configuration of the underlying shared client", () => {
        const newConfig = { timeout: 15000 };

        wrapper.updateConfig(newConfig);

        expect(mockSharedClient.updateConfig).toHaveBeenCalledTimes(1);
        expect(mockSharedClient.updateConfig).toHaveBeenCalledWith(newConfig);
      });
    });

    describe("getMetrics", () => {
      it("gets metrics from the underlying shared client", () => {
        const mockMetrics = [
          {
            requestId: "req-1",
            method: "GET",
            endpoint: "/test",
            success: true,
            duration: 100,
          },
        ];
        mockSharedClient.getMetrics.mockReturnValue(mockMetrics);

        const metrics = wrapper.getMetrics();

        expect(mockSharedClient.getMetrics).toHaveBeenCalledTimes(1);
        expect(metrics).toEqual(mockMetrics);
      });
    });

    describe("getCircuitBreakerState", () => {
      it("gets circuit breaker state from the underlying shared client", () => {
        const mockState = { state: "CLOSED", failureCount: 0 };
        mockSharedClient.getCircuitBreakerState.mockReturnValue(mockState);

        const state = wrapper.getCircuitBreakerState();

        expect(mockSharedClient.getCircuitBreakerState).toHaveBeenCalledTimes(
          1,
        );
        expect(state).toEqual(mockState);
      });
    });

    describe("resetCircuitBreaker", () => {
      it("resets circuit breaker in the underlying shared client", () => {
        wrapper.resetCircuitBreaker();

        expect(mockSharedClient.resetCircuitBreaker).toHaveBeenCalledTimes(1);
      });
    });
  });

  describe("Factory Function", () => {
    it("creates wrapper with configuration", () => {
      const config = {
        baseUrl: "https://custom-api.example.com",
        authToken: "custom-token",
        tenantId: "custom-tenant",
      };

      const customWrapper = createSharedBackendClientWrapper(config);

      expect(customWrapper).toBeInstanceOf(SharedBackendClientWrapper);
      expect(createSharedBackendClient).toHaveBeenCalledWith(config);
    });

    it("creates wrapper with default configuration", () => {
      const defaultWrapper = createSharedBackendClientWrapper();

      expect(defaultWrapper).toBeInstanceOf(SharedBackendClientWrapper);
      expect(getSharedBackendClient).toHaveBeenCalled();
    });
  });

  describe("Error Propagation", () => {
    it("propagates errors from SharedBackendClient", async () => {
      const testError = new Error("Test error");
      mockSharedClient.getProviders.mockRejectedValue(testError);

      await expect(wrapper.getProviders()).rejects.toThrow("Test error");
    });

    it("propagates authentication errors", async () => {
      const authError = new Error("Authentication failed");
      mockSharedClient.getConnections.mockRejectedValue(authError);

      await expect(wrapper.getConnections()).rejects.toThrow(
        "Authentication failed",
      );
    });

    it("propagates network errors", async () => {
      const networkError = new Error("Network timeout");
      mockSharedClient.createConnection.mockRejectedValue(networkError);

      await expect(
        wrapper.createConnection(
          {} as Omit<DemoConnection, "id" | "createdAt">,
        ),
      ).rejects.toThrow("Network timeout");
    });
  });

  describe("Interface Compliance", () => {
    it("implements all required DemoApiClient methods", () => {
      const requiredMethods = [
        "getProviders",
        "getConnections",
        "createConnection",
        "updateConnection",
        "deleteConnection",
        "getSignals",
        "getGroundedSignals",
        "getSyncJobs",
        "getWebhooks",
        "getTokens",
        "getRateLimits",
      ];

      requiredMethods.forEach((method) => {
        expect(wrapper).toHaveProperty(method);
        expect(typeof wrapper[method as keyof SharedBackendClientWrapper]).toBe(
          "function",
        );
      });
    });

    it("provides access to enhanced features", () => {
      const enhancedMethods = [
        "getSharedClient",
        "updateConfig",
        "getMetrics",
        "getCircuitBreakerState",
        "resetCircuitBreaker",
      ];

      enhancedMethods.forEach((method) => {
        expect(wrapper).toHaveProperty(method);
        expect(typeof wrapper[method as keyof SharedBackendClientWrapper]).toBe(
          "function",
        );
      });
    });
  });
});
