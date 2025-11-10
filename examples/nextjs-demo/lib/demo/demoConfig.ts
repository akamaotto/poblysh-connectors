/**
 * Demo configuration module with environment variable support.
 * 
 * This module handles the detection and validation of demo modes and provides
 * a unified interface for both mock and real API configurations.
 */

/**
 * Demo mode configuration.
 */
export type DemoMode = 'mock' | 'real';

/**
 * Runtime configuration interface.
 */
export interface RuntimeDemoConfig {
  /** Current demo mode */
  mode: DemoMode;
  /** Connectors API base URL (for real mode) */
  connectorsApiBaseUrl?: string;
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
  DEMO_MODE: 'NEXT_PUBLIC_DEMO_MODE',
  CONNECTORS_API_BASE_URL: 'CONNECTORS_API_BASE_URL',
} as const;

/**
 * Allowed demo mode values.
 */
const ALLOWED_DEMO_MODES: DemoMode[] = ['mock', 'real'];

/**
 * Default configuration values.
 */
const DEFAULT_CONFIG: RuntimeDemoConfig = {
  mode: 'mock',
  isValid: true,
  errors: [],
  warnings: [],
};

/**
 * Validates a URL string.
 */
function isValidUrl(url: string): boolean {
  try {
    const parsedUrl = new URL(url);
    return parsedUrl.protocol === 'https:';
  } catch {
    return false;
  }
}

/**
 * Validates the demo mode configuration.
 */
function validateDemoMode(mode: string | undefined): { mode: DemoMode; errors: string[]; warnings: string[] } {
  const errors: string[] = [];
  const warnings: string[] = [];
  
  if (!mode) {
    warnings.push(`${ENV_VARS.DEMO_MODE} not set, using default: mock`);
    return { mode: 'mock', errors, warnings };
  }
  
  if (!ALLOWED_DEMO_MODES.includes(mode as DemoMode)) {
    errors.push(`Invalid ${ENV_VARS.DEMO_MODE}: "${mode}". Must be one of: ${ALLOWED_DEMO_MODES.join(', ')}`);
    return { mode: 'mock', errors, warnings };
  }
  
  return { mode: mode as DemoMode, errors, warnings };
}

/**
 * Validates the Connectors API base URL configuration.
 */
function validateApiBaseUrl(baseUrl: string | undefined, mode: DemoMode): { 
  baseUrl: string | undefined; 
  errors: string[]; 
  warnings: string[]; 
} {
  const errors: string[] = [];
  const warnings: string[] = [];
  
  if (mode === 'real') {
    if (!baseUrl) {
      errors.push(`${ENV_VARS.CONNECTORS_API_BASE_URL} is required when ${ENV_VARS.DEMO_MODE}=real`);
      return { baseUrl: undefined, errors, warnings };
    }
    
    if (!isValidUrl(baseUrl)) {
      errors.push(`${ENV_VARS.CONNECTORS_API_BASE_URL} must be a valid HTTPS URL when ${ENV_VARS.DEMO_MODE}=real`);
      return { baseUrl: undefined, errors, warnings };
    }
    
    // Remove trailing slash for consistency
    const normalizedBaseUrl = baseUrl.replace(/\/$/, '');
    warnings.push(`Using real API mode with Connectors API at: ${normalizedBaseUrl}`);
    return { baseUrl: normalizedBaseUrl, errors, warnings };
  }
  
  if (baseUrl) {
    warnings.push(`${ENV_VARS.CONNECTORS_API_BASE_URL} is set but will be ignored in mock mode`);
  }
  
  return { baseUrl: undefined, errors, warnings };
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
  const modeValue = process.env[ENV_VARS.DEMO_MODE];
  const baseUrlValue = process.env[ENV_VARS.CONNECTORS_API_BASE_URL];
  
  // Validate demo mode
  const { mode, errors: modeErrors, warnings: modeWarnings } = validateDemoMode(modeValue);
  
  // Validate API base URL
  const { 
    baseUrl: connectorsApiBaseUrl, 
    errors: baseUrlErrors, 
    warnings: baseUrlWarnings 
  } = validateApiBaseUrl(baseUrlValue, mode);
  
  // Combine errors and warnings
  const errors = [...modeErrors, ...baseUrlErrors];
  const warnings = [...modeWarnings, ...baseUrlWarnings];
  
  // Determine if configuration is valid
  const isValid = errors.length === 0;
  
  const config: RuntimeDemoConfig = {
    mode,
    connectorsApiBaseUrl,
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
  return config.mode === 'mock';
}

/**
 * Checks if the demo is running in real mode.
 * 
 * @returns true if in real mode, false otherwise
 */
export function isRealMode(): boolean {
  const config = getDemoConfig();
  return config.mode === 'real';
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
    if (config.mode === 'real' && !config.connectorsApiBaseUrl) {
      suggestions.push(`Set ${ENV_VARS.CONNECTORS_API_BASE_URL} to your Connectors API endpoint`);
      suggestions.push('Or switch to mock mode by removing or setting NEXT_PUBLIC_DEMO_MODE=mock');
    }
    
    if (config.mode === 'real' && config.connectorsApiBaseUrl && !isValidUrl(config.connectorsApiBaseUrl)) {
      suggestions.push(`Ensure ${ENV_VARS.CONNECTORS_API_BASE_URL} is a valid HTTPS URL`);
      suggestions.push('Example: https://your-connectors-api.example.com');
    }
  }
  
  if (config.warnings.length > 0) {
    suggestions.push('Consider setting environment variables explicitly to avoid warnings');
    suggestions.push('See .env.example files for configuration examples');
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
export function logConfigurationValidation(result: ConfigValidationResult): void {
  console.group('🔧 Demo Configuration Validation');
  
  // Log mode
  console.log(`✅ Mode: ${result.mode.toUpperCase()}`);
  
  if (result.apiBaseUrl) {
    console.log(`🌐 API Base URL: ${result.apiBaseUrl}`);
  }
  
  // Log errors
  if (result.errors.length > 0) {
    console.group('❌ Errors:');
    result.errors.forEach(error => console.error(`  • ${error}`));
    console.groupEnd();
  }
  
  // Log warnings
  if (result.warnings.length > 0) {
    console.group('⚠️  Warnings:');
    result.warnings.forEach(warning => console.warn(`  • ${warning}`));
    console.groupEnd();
  }
  
  // Log suggestions
  if (result.suggestions && result.suggestions.length > 0) {
    console.group('💡 Suggestions:');
    result.suggestions.forEach(suggestion => console.log(`  • ${suggestion}`));
    console.groupEnd();
  }
  
  // Log final status
  if (result.isValid) {
    console.log('✅ Configuration is valid');
  } else {
    console.error('❌ Configuration has errors that must be fixed');
  }
  
  console.groupEnd();
}