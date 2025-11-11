/**
 * SharedBackendClient tests (Bun + vi-native)
 *
 * These tests:
 * - Use Bun's built-in test runner and vi-style mocks directly.
 * - Do not depend on any Jest shim or global jest.
 * - Isolate all mocks within this file to avoid cross-suite interference.
 *
 * Conventions:
 * - Use vi.fn / vi.mock / vi.spyOn for mocking.
 * - Use Bun's `describe` / `it` / `beforeEach` / `afterEach` / `expect`.
 * - Keep assertions aligned with the actual SharedBackendClient implementation.
 */

import { describe, it, beforeEach, afterEach, expect, vi } from "bun:test";
import {
  SharedBackendClient,
  createSharedBackendClient,
  NETWORK_ERRORS,
} from "../sharedBackendClient";
import { DemoConnection, DemoProvider, DemoApiResponse } from "../types";

/**
 * Typed fetch mock helper: we override globalThis.fetch in each test.
 */
type FetchMock = ReturnType<typeof vi.fn<(input: RequestInfo | URL, init?: RequestInit) => Promise<Response>>>;

describe("SharedBackendClient", () => {
  let client: SharedBackendClient;
  let fetchMock: FetchMock;

  /**
   * Install a fresh vi-based fetch mock and client instance before each test.
   */
  beforeEach(() => {
    fetchMock = vi.fn<(input: RequestInfo | URL, init?: RequestInit) => Promise<Response>>();
    // Override global fetch for this test file.
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    (globalThis as any).fetch = fetchMock;

    client = createSharedBackendClient({
      baseUrl: "https://test-api.example.com",
      authToken: "test-token",
      tenantId: "test-tenant",
      timeout: 5_000,
      retry: {
        maxAttempts: 3,
        baseDelay: 5,
        maxDelay: 20,
        multiplier: 1,
        jitter: false,
      },
      rateLimit: {
        requestsPerSecond: 100,
        burstCapacity: 100,
      },
      circuitBreaker: {
        failureThreshold: 5,
        recoveryTimeout: 500,
        monitoringPeriod: 5_000,
      },
      enableLogging: false,
      enableEducationalAnnotations: false,
    });
  });

  /**
   * Reset all vi mocks and client state after each test.
   */
  afterEach(() => {
    vi.restoreAllMocks();
    if (typeof client.clearMetrics === "function") {
      client.clearMetrics();
    }
    if (typeof client.resetCircuitBreaker === "function") {
      client.resetCircuitBreaker();
    }
  });

  // ---------------------------------------------------------------------------
  // Configuration
  // ---------------------------------------------------------------------------

  describe("Configuration", () => {
    it("initializes with default configuration", () => {
      const defaultClient = createSharedBackendClient();

      expect(defaultClient).toBeTruthy();
      expect(
        typeof (defaultClient as unknown as { getProviders: () => void })
          .getProviders,
      ).toBe("function");
    });

    it("accepts custom configuration", () => {
      const customClient = createSharedBackendClient({
        baseUrl: "https://custom-api.example.com",
        timeout: 10_000,
        retry: {
          maxAttempts: 5,
          baseDelay: 50,
          maxDelay: 200,
          multiplier: 2,
          jitter: true,
        },
      });

      expect(customClient).toBeTruthy();
      expect(
        typeof (customClient as unknown as { getProviders: () => void })
          .getProviders,
      ).toBe("function");
    });

    it("updates configuration at runtime without throwing", () => {
      client.updateConfig({
        timeout: 15_000,
        rateLimit: {
          requestsPerSecond: 50,
          burstCapacity: 50,
        },
      });

      // If we reached here, updateConfig worked; we don't assert internals.
      expect(true).toBe(true);
    });
  });

  // ---------------------------------------------------------------------------
  // Request headers & simple success paths
  // ---------------------------------------------------------------------------

  describe("Request Headers", () => {
    it("includes required headers in GET requests", async () => {
      const providers: DemoProvider[] = [
        {
          slug: "github",
          name: "GitHub",
          description: "GitHub",
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

      const payload: DemoApiResponse<DemoProvider[]> = {
        data: providers,
        meta: {
          requestId: "req-123",
          timestamp: new Date().toISOString(),
        },
      };

      fetchMock.mockResolvedValueOnce(
        new Response(JSON.stringify(payload), {
          status: 200,
          headers: { "Content-Type": "application/json" },
        }) as unknown as Response,
      );

      await client.getProviders();

      expect(fetchMock).toHaveBeenCalledTimes(1);
      const [url, options] = fetchMock.mock.calls[0] as [string, RequestInit];

      expect(url).toBe("https://test-api.example.com/api/v1/providers");
      expect(options.method).toBe("GET");
      const headers = options.headers as Record<string, string>;
      expect(headers["Content-Type"]).toBe("application/json");
      expect(headers["Authorization"]).toBe("Bearer test-token");
      expect(headers["X-Tenant-Id"]).toBe("test-tenant");
      expect(typeof headers["X-Request-Id"]).toBe("string");
    });
  });

  describe("Successful Requests", () => {
    it("handles successful GET", async () => {
      const providersData = [
          { slug: "github", name: "GitHub" } as DemoProvider,
          { slug: "slack", name: "Slack" } as DemoProvider,
        ];

      fetchMock.mockResolvedValueOnce({
        ok: true,
        status: 200,
        headers: new Headers({ "Content-Type": "application/json" }),
        json: async () => providersData,
      } as Response);

      const result = await client.getProviders();

      expect(fetchMock).toHaveBeenCalledTimes(1);
      expect(result.data.length).toBe(2);
      expect(result.data[0].slug).toBe("github");
    });

    it("handles successful POST", async () => {
      const newConnection: Omit<DemoConnection, "id" | "createdAt"> = {
        tenantId: "test-tenant",
        providerSlug: "github",
        displayName: "Test Connection",
        status: "connected",
      };

      const connectionData = {
          id: "conn-123",
          createdAt: "2024-01-01T00:00:00.000Z",
          ...newConnection,
        } as DemoConnection;

      fetchMock.mockResolvedValueOnce({
        ok: true,
        status: 201,
        headers: new Headers({ "Content-Type": "application/json" }),
        json: async () => connectionData,
      } as Response);

      const result = await client.createConnection(newConnection);

      expect(fetchMock).toHaveBeenCalledTimes(1);
      const [url, options] = fetchMock.mock.calls[0] as [string, RequestInit];

      expect(url).toBe("https://test-api.example.com/api/v1/connections");
      expect(options.method).toBe("POST");
      expect(options.body).toBe(JSON.stringify(newConnection));
      expect(result.data.id).toBe("conn-123");
    });

    it("handles successful DELETE", async () => {
      fetchMock.mockResolvedValueOnce(
        new Response(null, { status: 204 }) as unknown as Response,
      );

      const result = await client.deleteConnection("conn-123");

      expect(result).toBeUndefined();
      expect(fetchMock).toHaveBeenCalledTimes(1);
      const [url, options] = fetchMock.mock.calls[0] as [string, RequestInit];
      expect(url).toBe(
        "https://test-api.example.com/api/v1/connections/conn-123",
      );
      expect(options.method).toBe("DELETE");
    });
  });

  // ---------------------------------------------------------------------------
  // Error handling
  // ---------------------------------------------------------------------------

  describe("Error Handling", () => {
    it("handles network rejection and records metrics", async () => {
      fetchMock.mockRejectedValueOnce(new Error("Network error"));

      await expect(client.getProviders()).rejects.toThrow();

      const metrics = client.getMetrics();
      expect(metrics.length).toBeGreaterThan(0);
      const last = metrics[metrics.length - 1];
      expect(last.success).toBe(false);
      expect(typeof last.error).toBe("string");
    });

    it("handles HTTP 401 unauthorized errors", async () => {
      fetchMock.mockResolvedValueOnce(
        new Response(
          JSON.stringify({ message: "Unauthorized", code: "AUTH_001" }),
          { status: 401 },
        ) as unknown as Response,
      );

      await expect(client.getProviders()).rejects.toThrow("Unauthorized");

      const [metric] = client.getMetrics();
      expect(metric.statusCode).toBe(401);
      expect(metric.success).toBe(false);
    });

    it("handles HTTP 403 forbidden errors", async () => {
      fetchMock.mockResolvedValueOnce(
        new Response(
          JSON.stringify({
            message: "Insufficient permissions",
            code: "AUTH_003",
          }),
          { status: 403 },
        ) as unknown as Response,
      );

      await expect(client.getProviders()).rejects.toThrow(
        "Insufficient permissions",
      );

      const [metric] = client.getMetrics();
      expect(metric.statusCode).toBe(403);
      expect(metric.success).toBe(false);
    });

    it("handles HTTP 500 server errors without retries when configured", async () => {
      // Create isolated client with no retries.
      const noRetryClient = createSharedBackendClient({
        baseUrl: "https://test-api.example.com",
        authToken: "test-token",
        tenantId: "test-tenant",
        retry: {
          maxAttempts: 1,
          baseDelay: 1,
          maxDelay: 1,
          multiplier: 1,
          jitter: false,
        },
        enableLogging: false,
        enableEducationalAnnotations: false,
      });

      fetchMock.mockResolvedValueOnce(
        new Response(JSON.stringify({ message: "Internal server error" }), {
          status: 500,
        }) as unknown as Response,
      );

      await expect(noRetryClient.getProviders()).rejects.toThrow(
        "Internal server error",
      );

      const [metric] = noRetryClient.getMetrics();
      expect(metric.statusCode).toBe(500);
      expect(metric.retryCount).toBe(0);
    });

    it("handles malformed JSON responses", async () => {
      const badResponse = {
        ok: true,
        status: 200,
        headers: new Headers(),
        json: vi.fn().mockRejectedValue(new Error("Unexpected token")),
      } as unknown as Response;

      fetchMock.mockResolvedValueOnce(badResponse);

      await expect(client.getProviders()).rejects.toThrow("Invalid JSON response");
    });
  });

  // ---------------------------------------------------------------------------
  // Retry logic
  // ---------------------------------------------------------------------------

  describe("Retry Logic", () => {
    beforeEach(() => {
      // Fresh mock + client specifically tuned for retry tests.
      fetchMock = vi.fn<(input: RequestInfo | URL, init?: RequestInit) => Promise<Response>>();
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      (globalThis as any).fetch = fetchMock;

      client = createSharedBackendClient({
        baseUrl: "https://test-api.example.com",
        authToken: "test-token",
        tenantId: "test-tenant",
        retry: {
          maxAttempts: 3,
          baseDelay: 1,
          maxDelay: 2,
          multiplier: 1,
          jitter: false,
        },
        enableLogging: false,
        enableEducationalAnnotations: false,
      });
    });

    it("retries transient failures then succeeds", async () => {
      const okProviderData = [{ slug: "github", name: "GitHub" } as DemoProvider];

      fetchMock
        .mockRejectedValueOnce(new Error("Network error"))
        .mockResolvedValueOnce({
          ok: true,
          status: 200,
          headers: new Headers({ "Content-Type": "application/json" }),
          json: async () => okProviderData,
        } as Response);

      const result = await client.getProviders();

      expect(result.data.length).toBe(1);
      expect(fetchMock.mock.calls.length).toBeGreaterThanOrEqual(2);
    });

    it("does not retry authentication errors", async () => {
      fetchMock.mockResolvedValueOnce(
        new Response(
          JSON.stringify({
            message: "Unauthorized",
            code: "AUTH_001",
          }),
          { status: 401 },
        ) as unknown as Response,
      );

      await expect(client.getProviders()).rejects.toThrow("Unauthorized");
      expect(fetchMock).toHaveBeenCalledTimes(1);

      const [metric] = client.getMetrics();
      expect(metric.retryCount).toBe(0);
    });

    it("eventually fails after max retries for persistent failures", async () => {
      fetchMock.mockRejectedValue(new Error("Persistent network error"));

      await expect(client.getProviders()).rejects.toThrow(
        "Persistent network error",
      );
      expect(fetchMock.mock.calls.length).toBeGreaterThanOrEqual(3);
    });
  });

  // ---------------------------------------------------------------------------
  // Rate limiting
  // ---------------------------------------------------------------------------

  describe("Rate Limiting", () => {
    beforeEach(() => {
      fetchMock = vi.fn<(input: RequestInfo | URL, init?: RequestInit) => Promise<Response>>();
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      (globalThis as any).fetch = fetchMock;

      client = createSharedBackendClient({
        baseUrl: "https://test-api.example.com",
        authToken: "test-token",
        tenantId: "test-tenant",
        rateLimit: {
          requestsPerSecond: 1,
          burstCapacity: 1,
        },
        retry: {
          maxAttempts: 1,
          baseDelay: 1,
          maxDelay: 1,
          multiplier: 1,
          jitter: false,
        },
        enableLogging: false,
        enableEducationalAnnotations: false,
      });
    });

    it("allows requests within rate limits", async () => {
      const payload: DemoApiResponse<DemoProvider[]> = {
        data: [],
        meta: {
          requestId: "req-rl-1",
          timestamp: new Date().toISOString(),
        },
      };

      fetchMock.mockResolvedValueOnce(
        new Response(JSON.stringify(payload), {
          status: 200,
          headers: { "Content-Type": "application/json" },
        }) as unknown as Response,
      );

      await client.getProviders();
      expect(fetchMock).toHaveBeenCalledTimes(1);
    });

    it("throws error when rate limit exceeded before calling fetch", async () => {
      const payload: DemoApiResponse<DemoProvider[]> = {
        data: [],
        meta: {
          requestId: "req-rl-1",
          timestamp: new Date().toISOString(),
        },
      };

      // First request succeeds and consumes tokens.
      fetchMock.mockResolvedValueOnce(
        new Response(JSON.stringify(payload), {
          status: 200,
          headers: { "Content-Type": "application/json" },
        }) as unknown as Response,
      );
      await client.getProviders();
      expect(fetchMock).toHaveBeenCalledTimes(1);

      // Next call should trip the rate limiter in the client.
      await expect(client.getProviders()).rejects.toThrow(
        NETWORK_ERRORS.NET_004.message,
      );
    });
  });

  // ---------------------------------------------------------------------------
  // Circuit breaker
  // ---------------------------------------------------------------------------

  describe("Circuit Breaker", () => {
    beforeEach(() => {
      fetchMock = vi.fn<(input: RequestInfo | URL, init?: RequestInit) => Promise<Response>>();
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      (globalThis as any).fetch = fetchMock;

      client = createSharedBackendClient({
        baseUrl: "https://test-api.example.com",
        authToken: "test-token",
        tenantId: "test-tenant",
        retry: {
          maxAttempts: 1,
          baseDelay: 1,
          maxDelay: 1,
          multiplier: 1,
          jitter: false,
        },
        circuitBreaker: {
          failureThreshold: 2,
          recoveryTimeout: 200,
          monitoringPeriod: 1_000,
        },
        enableLogging: false,
        enableEducationalAnnotations: false,
      });
    });

    it("opens circuit after repeated failures", async () => {
      fetchMock.mockRejectedValue(new Error("Network error"));

      await expect(client.getProviders()).rejects.toThrow();
      await expect(client.getProviders()).rejects.toThrow();

      const callsBefore = fetchMock.mock.calls.length;

      await expect(client.getProviders()).rejects.toThrow(
        NETWORK_ERRORS.NET_005.message,
      );

      const callsAfter = fetchMock.mock.calls.length;
      expect(callsAfter).toBe(callsBefore);
    });
  });

  // ---------------------------------------------------------------------------
  // Metrics & Monitoring
  // ---------------------------------------------------------------------------

  describe("Metrics & Monitoring", () => {
    it("collects metrics for successful requests", async () => {
      const payload: DemoApiResponse<DemoProvider[]> = {
        data: [],
        meta: {
          requestId: "req-m-1",
          timestamp: new Date().toISOString(),
        },
      };

      fetchMock.mockResolvedValueOnce(
        new Response(JSON.stringify(payload), {
          status: 200,
          headers: { "Content-Type": "application/json" },
        }) as unknown as Response,
      );

      await client.getProviders();

      const metrics = client.getMetrics();
      expect(metrics.length).toBeGreaterThan(0);
      const m = metrics[0];
      expect(m.method).toBe("GET");
      expect(m.endpoint).toBe("/api/v1/providers");
      expect(m.success).toBe(true);
    });

    it("clearMetrics removes stored metrics", async () => {
      const payload: DemoApiResponse<DemoProvider[]> = {
        data: [],
        meta: {
          requestId: "req-m-1",
          timestamp: new Date().toISOString(),
        },
      };

      fetchMock.mockResolvedValueOnce(
        new Response(JSON.stringify(payload), {
          status: 200,
          headers: { "Content-Type": "application/json" },
        }) as unknown as Response,
      );

      await client.getProviders();
      expect(client.getMetrics().length).toBeGreaterThan(0);

      client.clearMetrics();
      expect(client.getMetrics().length).toBe(0);
    });

    it("exposes circuit breaker and rate limiter state", () => {
      const cb = client.getCircuitBreakerState();
      const rl = client.getRateLimiterState();

      expect(
        cb.state === "CLOSED" ||
          cb.state === "OPEN" ||
          cb.state === "HALF_OPEN",
      ).toBe(true);
      expect(typeof cb.failureCount).toBe("number");

      expect(typeof rl.tokens).toBe("number");
      expect(typeof rl.lastRequestTime).toBe("number");
    });
  });
});
