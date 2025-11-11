import {
  AppResult,
  AppError,
  Ok,
  asyncResult,
  ValidationError,
  AuthenticationError,
  ConfigurationError,
} from "./types/functional";

import { DemoTenant, DemoApiResponse } from "./types";

import {
  createSharedBackendClientWrapper,
  SharedBackendClientWrapper,
} from "./sharedBackendClientWrapper";

import { getDemoConfig } from "./demoConfig";
import { generateMockTenant } from "./mockData";
import { TenantMapping, addTenantMapping } from "./tenantMapping";

/**
 * Tenant creation input shape for the demo.
 * Matches the tenant creation spec while remaining demo-friendly.
 */
export interface TenantCreationInput {
  name: string;
  metadata?: TenantMetadata;
}

/**
 * Tenant metadata supported by the demo.
 * This is a subset / superset adapter for the backend schema.
 */
export interface TenantMetadata {
  poblysh_tenant_id?: string;
  organization?: string;
  created_by?: string;
  environment?: "local" | "test" | "staging" | "prod" | string;
  // Additional keys allowed to keep demo flexible
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  [key: string]: any;
}

/**
 * Result payload for a tenant creation attempt.
 * - `isReal`: whether the tenant was created via real API (Mode B).
 * - `fallback`: whether we fell back to a mock tenant after real API failure.
 */
export interface TenantCreationResult {
  tenant: DemoTenant;
  isReal: boolean;
  fallback: boolean;
}

/**
 * Normalize and validate tenant input according to spec:
 * - Required name
 * - Non-empty after trimming
 * - Max length 255
 */
function validateTenantInput(
  input: TenantCreationInput,
): AppResult<TenantCreationInput> {
  const trimmed = input.name.trim();

  if (!trimmed) {
    return ValidationError(
      "name",
      "Name is required and cannot be empty or whitespace only",
    );
  }

  if (trimmed.length > 255) {
    return ValidationError("name", "Name must be at most 255 characters");
  }

  const normalized: TenantCreationInput = {
    ...input,
    name: trimmed,
  };

  return Ok(normalized);
}

/**
 * Create a URL-friendly tenant slug.
 */
function slugify(name: string): string {
  return name
    .toLowerCase()
    .replace(/[^a-z0-9\s-]/g, "")
    .replace(/\s+/g, "-")
    .replace(/-+/g, "-")
    .replace(/^-|-$/g, "");
}

/**
 * Build a mock tenant according to the DemoTenant model.
 * Used for:
 * - Mode A (pure mock)
 * - Fallback when real API creation fails
 */
function buildMockTenant(input: TenantCreationInput): TenantCreationResult {
  const base = generateMockTenant(input.name);

  const tenant: DemoTenant = {
    ...base,
    name: input.name,
    slug: slugify(input.name),
    // In mock mode, use the same ID for connectorsTenantId to keep mapping simple.
    connectorsTenantId: base.connectorsTenantId ?? base.id,
    createdAt: base.createdAt,
    plan: base.plan ?? "free",
  };

  return {
    tenant,
    isReal: false,
    fallback: false,
  };
}

/**
 * Raw backend tenant response shape (snake_case) used for normalization.
 */
interface BackendTenantResponse {
  id: string;
  name: string | null;
  created_at: string;
  updated_at?: string | null;
  metadata?: Record<string, unknown>;
}

/**
 * Map a successful real API response into our DemoTenant +
 * create and persist the TenantMapping.
 *
 * This ensures that:
 * - DemoTenant.connectorsTenantId is the real backend tenant UUID.
 * - A TenantMapping is stored for future lookups.
 *
 * The response may already be:
 * - A DemoTenant (from SharedBackendClientWrapper), or
 * - A BackendTenantResponse from the Rust API.
 */
function mapRealTenantResponse(
  input: TenantCreationInput,
  response: DemoApiResponse<DemoTenant | BackendTenantResponse>,
): TenantCreationResult {
  const raw = response.data;

  const id: string =
    "connectorsTenantId" in raw && typeof raw.connectorsTenantId === "string"
      ? raw.connectorsTenantId
      : raw.id;

  const createdAt: string =
    "created_at" in raw && typeof raw.created_at === "string"
      ? raw.created_at
      : "createdAt" in raw && typeof raw.createdAt === "string"
        ? raw.createdAt
        : new Date().toISOString();

  const nameValue: string | null =
    typeof raw.name === "string" ? raw.name : null;

  const displayName = nameValue ?? input.name;

  const tenant: DemoTenant = {
    // Frontend tenant ID (Poblysh Core tenant context for the demo)
    id: `tenant-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`,
    name: displayName,
    slug: slugify(displayName),
    // Backend Connectors tenant UUID: MUST be used in X-Tenant-Id header in real mode
    connectorsTenantId: id,
    createdAt,
    plan: "free",
  };

  const mapping: TenantMapping = {
    frontendTenantId: tenant.id,
    connectorsTenantId: tenant.connectorsTenantId,
    createdAt: new Date().toISOString(),
    name: tenant.name,
  };

  addTenantMapping(mapping);

  return {
    tenant,
    isReal: true,
    fallback: false,
  };
}

/**
 * Update the underlying SharedBackendClient so that:
 * - In real mode, X-Tenant-Id uses connectorsTenantId.
 * - In mock mode, X-Tenant-Id may use the mock tenant ID.
 *
 * This function encodes the spec requirement:
 * "Subsequent Mode B API calls MUST use the returned connectorsTenantId
 * in the X-Tenant-Id header."
 */
function updateClientTenantContext(
  client: SharedBackendClientWrapper,
  result: TenantCreationResult,
): void {
  if (result.isReal && !result.fallback) {
    // Real tenant: use connectorsTenantId for X-Tenant-Id
    client.updateConfig({
      tenantId: result.tenant.connectorsTenantId,
    });
  } else {
    // Mock or fallback: use the frontend tenant ID
    client.updateConfig({
      tenantId: result.tenant.id,
    });
  }
}

/**
 * Create a tenant in Mode A (mock) using Result type.
 */
function createTenantMock(
  input: TenantCreationInput,
): AppResult<TenantCreationResult> {
  const validated = validateTenantInput(input);
  if (validated._tag === "Err") {
    return validated;
  }

  const result = buildMockTenant(validated.value);
  return Ok(result);
}

/**
 * Attempt to create a tenant via the real Connectors API (Mode B).
 *
 * - Validates configuration (base URL, auth).
 * - Uses SharedBackendClientWrapper to call POST /api/v1/tenants.
 * - On success:
 *   - Maps to DemoTenant
 *   - Persists TenantMapping
 *   - Configures X-Tenant-Id with connectorsTenantId
 * - On failure:
 *   - Returns an AppError (no hidden throws)
 */
async function createTenantReal(
  input: TenantCreationInput,
): Promise<AppResult<TenantCreationResult>> {
  const validated = validateTenantInput(input);
  if (validated._tag === "Err") {
    return validated;
  }

  const config = getDemoConfig();

  if (config.mode !== "real") {
    return ConfigurationError("createTenantReal called while not in real mode");
  }

  if (!config.connectorsApiBaseUrl) {
    return ConfigurationError(
      "Mode B (real API) requires CONNECTORS_API_BASE_URL to be configured when running in real mode",
    );
  }

  if (!config.connectorsApiToken) {
    return AuthenticationError(
      "Mode B (real API) requires CONNECTORS_API_TOKEN (operator auth token) to be configured",
    );
  }

  return asyncResult(
    async () => {
      const client = createSharedBackendClientWrapper({
        baseUrl: config.connectorsApiBaseUrl,
        authToken: config.connectorsApiToken,
        tenantId: "", // will be set after creation
        timeout: config.apiTimeout ?? 10000,
        enableLogging: config.enableApiLogging ?? true,
        enableEducationalAnnotations:
          config.enableEducationalAnnotations ?? true,
      });

      const apiResponse = await client.createTenant({
        name: validated.value.name,
        metadata: validated.value.metadata ?? {},
      });

      const creationResult = mapRealTenantResponse(
        validated.value,
        apiResponse,
      );

      // Ensure subsequent real API calls use connectorsTenantId.
      updateClientTenantContext(client, creationResult);

      return creationResult;
    },
    (error: unknown): AppError => {
      if (error instanceof Error) {
        const message = error.message.toLowerCase();

        if (message.includes("unauthorized") || message.includes("forbidden")) {
          return {
            _tag: "AuthenticationError",
            message: "Authentication failed while creating tenant via real API",
          };
        }

        if (
          message.includes("validation") ||
          message.includes("400") ||
          message.includes("validation_failed")
        ) {
          return {
            _tag: "ValidationError",
            field: "tenant",
            message: "Tenant data was rejected by the Connectors API",
          };
        }

        if (
          message.includes("network") ||
          message.includes("fetch") ||
          message.includes("timeout")
        ) {
          return {
            _tag: "NetworkError",
            message:
              "Network error while calling Connectors API for tenant creation",
          };
        }

        return {
          _tag: "NetworkError",
          message: `Real API tenant creation failed: ${error.message}`,
        };
      }

      return {
        _tag: "NetworkError",
        message: "Unknown error during real API tenant creation",
      };
    },
  );
}

/**
 * Public API used by the Tenant page and other flows.
 *
 * Behavior:
 * - If mode === 'mock': creates a mock tenant (Mode A) -> Ok(TenantCreationResult)
 * - If mode === 'real':
 *   - Attempts real API creation (Mode B)
 *   - On success: Ok(TenantCreationResult) with isReal = true, fallback = false
 *   - On failure:
 *       - Returns Err(AppError) so callers can decide whether to:
 *         - Show configuration/auth errors, or
 *         - Trigger an explicit fallback to mock.
 *
 * This function DOES NOT silently fallback to mock. That responsibility is left
 * to the caller so that the UI can clearly communicate fallback behavior.
 */
export async function createTenantSafe(
  input: TenantCreationInput,
): Promise<AppResult<TenantCreationResult>> {
  const config = getDemoConfig();

  if (config.mode === "mock") {
    return createTenantMock(input);
  }

  // Mode B: attempt real creation
  return createTenantReal(input);
}

/**
 * Helper to explicitly perform a "fallback to mock" when real API creation fails.
 *
 * This keeps the behavior explicit:
 * - Call createTenantSafe()
 * - If Err and you want to keep the demo running, call createTenantFallback()
 * - Show a clear banner: "You are in fallback mock mode"
 */
export function createTenantFallback(
  input: TenantCreationInput,
): AppResult<TenantCreationResult> {
  const validated = validateTenantInput(input);
  if (validated._tag === "Err") {
    return validated;
  }

  const baseResult = buildMockTenant(validated.value);

  const fallbackResult: TenantCreationResult = {
    tenant: baseResult.tenant,
    isReal: false,
    fallback: true,
  };

  return Ok(fallbackResult);
}
