/**
 * Demo configuration module with environment variable support.
 *
 * This module handles the detection and validation of demo modes and provides
 * a unified interface for both mock and real API configurations.
 */

/**
 * Demo mode configuration.
 */
export type DemoMode = "mock" | "real";

/**
 * Runtime configuration interface.
 */
export interface RuntimeDemoConfig {
  /** Current demo mode */
  mode: DemoMode;
  /** Connectors API base URL (for real mode) */
  connectorsApiBaseUrl?: string;
  /** Connectors API authentication token (for real mode) */
  connectorsApiToken?: string;
  /** Tenant ID for multi-tenant requests */
  tenantId?: string;
  /** API request timeout in milliseconds */
  apiTimeout?: number;
  /** Maximum number of API retry attempts */
  apiMaxRetries?: number;
  /** Whether to enable API request logging */
  enableApiLogging?: boolean;
  /** Whether to enable educational annotations */
  enableEducationalAnnotations?: boolean;
  /** Whether the configuration is valid */
  isValid: boolean;
  /** Configuration validation errors */
  errors: string[];
  /** Configuration warnings */
  warnings: string[];
}

/**
 * Environment variable names.
 */
export const ENV_VARS = {
  DEMO_MODE: "NEXT_PUBLIC_DEMO_MODE",
  CONNECTORS_API_BASE_URL: "CONNECTORS_API_BASE_URL",
  CONNECTORS_API_BASE_URL_PUBLIC: "NEXT_PUBLIC_CONNECTORS_API_BASE_URL",
} as const;

/**
 * Allowed demo mode values.
 */
const ALLOWED_DEMO_MODES: DemoMode[] = ["mock", "real"];

/**
 * Validates a URL string.
 */
function isValidUrl(url: string): boolean {
  try {
    const parsedUrl = new URL(url);

    if (parsedUrl.protocol === "https:") {
      return true;
    }

    if (
      parsedUrl.protocol === "http:" &&
      ["localhost", "127.0.0.1", "0.0.0.0"].includes(parsedUrl.hostname)
    ) {
      return true;
    }

    return false;
  } catch {
    return false;
  }
}

/**
 * Validates the demo mode configuration.
 */
function validateDemoMode(mode: string | undefined): {
  mode: DemoMode;
  errors: string[];
  warnings: string[];
} {
  const errors: string[] = [];
  const warnings: string[] = [];

  if (!mode) {
    return { mode: "mock", errors, warnings };
  }

  const normalized = mode.trim().toLowerCase();

  if (ALLOWED_DEMO_MODES.includes(normalized as DemoMode)) {
    return { mode: normalized as DemoMode, errors, warnings };
  }

  const reportedValue = mode.trim().length > 0 ? mode.trim() : "<empty>";
  warnings.push(
    `Invalid NEXT_PUBLIC_DEMO_MODE value '${reportedValue}'. Expected 'mock' or 'real', falling back to mock mode.`,
  );

  return { mode: "mock", errors, warnings };
}

/**
 * Validation result for the Connectors API base URL.
 */
interface ApiBaseUrlValidationResult {
  baseUrl?: string;
  errors: string[];
  warnings: string[];
  isValid: boolean;
}

/**
 * Validates the Connectors API base URL configuration.
 */
function validateApiBaseUrl(
  baseUrl: string | undefined,
  mode: DemoMode,
): ApiBaseUrlValidationResult {
  const errors: string[] = [];
  const warnings: string[] = [];
  const normalizedBaseUrl = baseUrl?.trim();

  if (mode !== "real") {
    return {
      baseUrl: normalizedBaseUrl,
      errors,
      warnings,
      isValid: true,
    };
  }

  const fallbackMessage =
    "Invalid CONNECTORS_API_BASE_URL for real mode. Expected valid HTTPS URL, falling back to mock mode.";

  if (!normalizedBaseUrl) {
    errors.push(fallbackMessage);
    return {
      baseUrl: undefined,
      errors,
      warnings,
      isValid: false,
    };
  }

  if (!isValidUrl(normalizedBaseUrl)) {
    errors.push(fallbackMessage);
    return {
      baseUrl: undefined,
      errors,
      warnings,
      isValid: false,
    };
  }

  return {
    baseUrl: normalizedBaseUrl,
    errors,
    warnings,
    isValid: true,
  };
}

function validateTenantEndpoints(
  baseUrl: string | undefined,
  mode: DemoMode,
): {
  isValid: boolean;
  errors: string[];
  warnings: string[];
} {
  const errors: string[] = [];
  const warnings: string[] = [];

  if (mode === "real" && baseUrl) {
    // In real mode, validate that tenant endpoints would be accessible
    // We can't actually make HTTP requests here, but we can validate the URL structure
    try {
      const tenantEndpointUrl = `${baseUrl}/api/v1/tenants`;
      const testUrl = new URL(tenantEndpointUrl);

      // For local development we allow http://localhost-style URLs; in all other
      // cases we still require HTTPS for security.
      const isLocalhost =
        testUrl.hostname === "localhost" ||
        testUrl.hostname === "127.0.0.1" ||
        testUrl.hostname === "0.0.0.0";

      if (testUrl.protocol !== "https:" && !isLocalhost) {
        errors.push(
          "Tenant API endpoints must use HTTPS unless using localhost in development",
        );
      }

      // Add informational warning about tenant endpoints
      warnings.push("Tenant creation API endpoints will be used in real mode");
    } catch {
      errors.push(
        `Invalid tenant API endpoint URL constructed: ${baseUrl}/api/v1/tenants`,
      );
    }
  }

  const isValid = errors.length === 0;
  return { isValid, errors, warnings };
}

/**
 * Loads and validates the demo configuration from environment variables.
 *
 * This function should be called at application startup to validate the configuration.
 * It performs comprehensive validation and provides helpful error messages.
 *
 * @returns Runtime configuration with validation results
 */
export function loadDemoConfig(): RuntimeDemoConfig {
  const modeValue = process.env.NEXT_PUBLIC_DEMO_MODE;
  const baseUrlValue =
    process.env.CONNECTORS_API_BASE_URL ??
    process.env.NEXT_PUBLIC_CONNECTORS_API_BASE_URL;

  // Validate (routing between mock and real modes)
  const {
    mode: configuredMode,
    errors: modeErrors,
    warnings: modeWarnings,
  } = validateDemoMode(modeValue);

  const baseUrlValidation = validateApiBaseUrl(baseUrlValue, configuredMode);

  let mode = configuredMode;
  let connectorsApiBaseUrl = baseUrlValidation.baseUrl;

  if (configuredMode === "real" && !baseUrlValidation.isValid) {
    mode = "mock";
    connectorsApiBaseUrl = undefined;
  }

  // Validate tenant endpoints after resolving the final mode/base URL.
  const { errors: tenantErrors, warnings: tenantWarnings } =
    validateTenantEndpoints(connectorsApiBaseUrl, mode);

  // Get additional configuration from environment variables
  // Force a placeholder token and tenant ID for local real-mode demo.
  const connectorsApiToken =
    process.env.CONNECTORS_API_TOKEN || "demo-local-operator-token";
  const tenantId = process.env.CONNECTORS_TENANT_ID || "demo-tenant";
  const apiTimeout = process.env.CONNECTORS_API_TIMEOUT
    ? parseInt(process.env.CONNECTORS_API_TIMEOUT, 10)
    : undefined;
  const apiMaxRetries = process.env.CONNECTORS_API_MAX_RETRIES
    ? parseInt(process.env.CONNECTORS_API_MAX_RETRIES, 10)
    : undefined;
  const enableApiLogging =
    process.env.CONNECTORS_ENABLE_API_LOGGING !== "false";
  const enableEducationalAnnotations =
    process.env.CONNECTORS_ENABLE_EDUCATIONAL !== "false";

  // Combine errors and warnings
  const errors = [
    ...modeErrors,
    ...baseUrlValidation.errors,
    ...tenantErrors,
  ];
  const warnings = [
    ...modeWarnings,
    ...baseUrlValidation.warnings,
    ...tenantWarnings,
  ];

  // Determine if configuration is valid
  const isValid = errors.length === 0;

  const config: RuntimeDemoConfig = {
    mode,
    connectorsApiBaseUrl,
    connectorsApiToken,
    tenantId,
    apiTimeout,
    apiMaxRetries,
    enableApiLogging,
    enableEducationalAnnotations,
    isValid,
    errors,
    warnings,
  };

  return config;
}

/**
 * Gets the current demo configuration.
 *
 * This is a convenience function that returns the cached configuration.
 * For most use cases, use this function instead of calling loadDemoConfig() directly.
 *
 * @returns Current runtime demo configuration
 */
export function getDemoConfig(): RuntimeDemoConfig {
  // In a real application, you might want to cache this result
  // For now, we'll call loadDemoConfig() each time to ensure fresh environment variables
  return loadDemoConfig();
}

/**
 * Checks if the demo is running in mock mode.
 *
 * @returns true if in mock mode, false otherwise
 */
export function isMockMode(): boolean {
  const config = getDemoConfig();
  return config.mode === "mock";
}

/**
 * Checks if the demo is running in real mode.
 *
 * @returns true if in real mode, false otherwise
 */
export function isRealMode(): boolean {
  const config = getDemoConfig();
  return config.mode === "real";
}

/**
 * Gets the Connectors API base URL for real mode.
 *
 * @returns API base URL if in real mode, undefined otherwise
 */
export function getConnectorsApiBaseUrl(): string | undefined {
  const config = getDemoConfig();
  return config.connectorsApiBaseUrl;
}

/**
 * Configuration validation result for startup logging.
 */
export interface ConfigValidationResult {
  /** Whether the configuration is valid */
  isValid: boolean;
  /** Current demo mode */
  mode: DemoMode;
  /** API base URL (if configured) */
  apiBaseUrl?: string;
  /** Validation errors */
  errors: string[];
  /** Validation warnings */
  warnings: string[];
  /** Suggested fix for common issues */
  suggestions?: string[];
}

/**
 * Performs comprehensive configuration validation and returns detailed results.
 *
 * This function is designed to be called at application startup to provide
 * comprehensive feedback about the configuration state.
 *
 * @returns Detailed validation results
 */
export function validateConfiguration(): ConfigValidationResult {
  const config = getDemoConfig();

  const suggestions: string[] = [];

  // Generate suggestions based on common issues
  if (config.errors.length > 0) {
    if (config.mode === "real" && !config.connectorsApiBaseUrl) {
      suggestions.push(
        `Set ${ENV_VARS.CONNECTORS_API_BASE_URL} (or ${ENV_VARS.CONNECTORS_API_BASE_URL_PUBLIC}) to your Connectors API endpoint`,
      );
      suggestions.push(
        "Or switch to mock mode by removing or setting NEXT_PUBLIC_DEMO_MODE=mock",
      );
    }

    if (
      config.mode === "real" &&
      config.connectorsApiBaseUrl &&
      !isValidUrl(config.connectorsApiBaseUrl)
    ) {
      suggestions.push(
        `Ensure ${ENV_VARS.CONNECTORS_API_BASE_URL} is a valid HTTPS URL`,
      );
      suggestions.push("Example: https://your-connectors-api.example.com");
    }
  }

  if (config.warnings.length > 0) {
    suggestions.push(
      "Consider setting environment variables explicitly to avoid warnings",
    );
    suggestions.push("See .env.example files for configuration examples");
  }

  return {
    isValid: config.isValid,
    mode: config.mode,
    apiBaseUrl: config.connectorsApiBaseUrl,
    errors: config.errors,
    warnings: config.warnings,
    suggestions: suggestions.length > 0 ? suggestions : undefined,
  };
}

/**
 * Logs configuration validation results to the console.
 *
 * This function provides formatted console output for debugging and
 * configuration verification during development.
 *
 * @param result Validation results to log
 */
export function logConfigurationValidation(
  result: ConfigValidationResult,
): void {
  console.group("🔧 Demo Configuration Validation");

  // Log mode
  console.log(`✅ Mode: ${result.mode.toUpperCase()}`);

  if (result.apiBaseUrl) {
    console.log(`🌐 API Base URL: ${result.apiBaseUrl}`);
  }

  // Log errors
  if (result.errors.length > 0) {
    console.group("❌ Errors:");
    result.errors.forEach((error) => console.error(`  • ${error}`));
    console.groupEnd();
  }

  // Log warnings
  if (result.warnings.length > 0) {
    console.group("⚠️  Warnings:");
    result.warnings.forEach((warning) => console.warn(`  • ${warning}`));
    console.groupEnd();
  }

  // Log suggestions
  if (result.suggestions && result.suggestions.length > 0) {
    console.group("💡 Suggestions:");
    result.suggestions.forEach((suggestion) =>
      console.log(`  • ${suggestion}`),
    );
    console.groupEnd();
  }

  // Log final status
  if (result.isValid) {
    console.log("✅ Configuration is valid");
  } else {
    console.error("❌ Configuration has errors that must be fixed");
  }

  console.groupEnd();
}
