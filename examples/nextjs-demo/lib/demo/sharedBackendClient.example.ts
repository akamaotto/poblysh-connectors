/**
 * Shared Backend Client + API Router Usage Examples
 *
 * This file provides lightweight, self-contained examples that:
 * - Demonstrate how to use the shared backend client singleton
 * - Show how to consume the `DemoApiClient` via `getApiClient()`
 * - Validate (by usage) that the OpenSpec contracts are satisfied
 *
 * These examples are intentionally simple and side-effect aware.
 * They can be used as:
 * - Documentation for engineers browsing the codebase
 * - A starting point for real unit/integration tests
 *
 * NOTE:
 * - This file is not imported by production code.
 * - You can turn these examples into tests by wiring them to your test runner.
 */

import {
  getApiClient,
  getApiModeInfo,
  isMockMode,
  isRealMode,
  hasEnhancedFeatures,
  getSharedBackendClient,
} from "./apiRouter";
import {
  DemoProvider,
  DemoConnection,
  DemoApiResponse,
  DemoApiError,
} from "./types";

/**
 * Example 1: Obtain the shared DemoApiClient (mode-agnostic)
 *
 * Demonstrates:
 * - `getApiClient()` always returning a `DemoApiClient`
 * - No need for callers to branch on mock vs real
 */
export async function exampleGetProviders(): Promise<
  DemoApiResponse<DemoProvider[]>
> {
  const api = getApiClient();

  const response = await api.getProviders();

  // Type-level guarantees:
  // - response.data is DemoProvider[]
  // - response.meta has requestId and timestamp (per DemoApiResponse)
  const first = response.data[0];
  if (first) {
    if (!first.slug || !first.name) {
      throw new Error(
        "exampleGetProviders: provider missing required fields (slug/name)",
      );
    }
  }

  return response;
}

/**
 * Example 2: Use the shared client singleton directly for advanced features
 *
 * Demonstrates:
 * - `getSharedBackendClient()` is non-null and returns a singleton
 * - Singleton semantics across calls
 */
export async function exampleSharedClientSingletonBehavior(): Promise<void> {
  const c1 = getSharedBackendClient();
  const c2 = getSharedBackendClient();

  // Must be the same instance (singleton per runtime)
  if (c1 !== c2) {
    throw new Error(
      "exampleSharedClientSingletonBehavior: expected shared backend client singleton",
    );
  }

  // Call a method to ensure interface is usable.
  // In some environments this may fail depending on configuration; that's fine.
  try {
    await c1.getRateLimits();
  } catch {
    // Intentionally ignored: this example is focused on typing and singleton behavior.
  }
}

/**
 * Example 3: Inspect mode and use enhanced features safely
 *
 * Demonstrates:
 * - How to gate "real API" UX on the current mode
 * - How to still rely on `getApiClient()` for core operations
 */
export async function exampleModeAwareUsage(): Promise<void> {
  const modeInfo = getApiModeInfo();
  const api = getApiClient();

  if (isMockMode()) {
    const connections = await api.getConnections();
    console.log(
      "[exampleModeAwareUsage] Mock mode connections:",
      connections.data.length,
    );
    return;
  }

  if (isRealMode() && hasEnhancedFeatures()) {
    const shared = getSharedBackendClient();

    try {
      const me = await shared.getCurrentUser();
      console.log(
        "[exampleModeAwareUsage] Real mode current user:",
        me.data.email,
      );
    } catch (error) {
      const apiError = error as Partial<DemoApiError>;
      console.warn(
        "[exampleModeAwareUsage] Failed to fetch current user",
        apiError.code,
        apiError.message,
      );
    }
  } else {
    throw new Error(
      `exampleModeAwareUsage: Unexpected mode configuration: ${modeInfo.mode}`,
    );
  }
}

/**
 * Example 4: Strict error handling with SharedBackendClient
 *
 * Demonstrates:
 * - How to narrow errors thrown by the shared client into DemoApiError shape
 * - How to write robust consumer code around the structured error model
 */
export async function exampleStrictErrorHandling(): Promise<void> {
  const shared = getSharedBackendClient();

  try {
    await shared.getProviders();
  } catch (error) {
    const e = error as Partial<DemoApiError>;

    if (typeof e.code !== "string" || typeof e.message !== "string") {
      throw new Error(
        "exampleStrictErrorHandling: Received error does not satisfy DemoApiError contract",
      );
    }

    if (e.code.startsWith("AUTH_")) {
      console.warn(
        "[exampleStrictErrorHandling] Authentication-related error",
        e.code,
        e.message,
      );
    } else if (e.code.startsWith("NET_")) {
      console.warn(
        "[exampleStrictErrorHandling] Network-related error",
        e.code,
        e.message,
      );
    } else {
      console.warn(
        "[exampleStrictErrorHandling] Generic error",
        e.code,
        e.message,
      );
    }
  }
}

/**
 * Example 5: Creating a connection via DemoApiClient (mode independent)
 *
 * Demonstrates:
 * - How to rely solely on the `DemoApiClient` interface
 * - Keeps components agnostic to mock vs real implementations
 */
export async function exampleCreateConnection(): Promise<
  DemoApiResponse<DemoConnection>
> {
  const api = getApiClient();

  const payload: Omit<DemoConnection, "id" | "createdAt"> = {
    tenantId: "demo-tenant",
    providerSlug: "github",
    displayName: "Demo GitHub Connection",
    status: "connected",
    lastSyncAt: new Date().toISOString(),
    error: undefined,
  };

  const created = await api.createConnection(payload);

  if (!created.data.id) {
    throw new Error(
      "exampleCreateConnection: created connection missing id (violates spec)",
    );
  }

  return created;
}
