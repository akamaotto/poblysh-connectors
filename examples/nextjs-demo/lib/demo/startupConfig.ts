/**
 * Startup configuration validation module.
 * 
 * This module provides utilities for validating demo configuration at application
 * startup and providing helpful feedback to developers about any issues.
 */

import React from 'react';
import { validateConfiguration, logConfigurationValidation } from './demoConfig';

/**
 * Performs startup configuration validation.
 * 
 * This function should be called during application initialization to ensure
 * the demo configuration is valid and to provide early feedback about any issues.
 * 
 * @returns Configuration validation result
 */
export function performStartupValidation() {
  // Only perform validation in development or test environments
  if (typeof window !== 'undefined' && process.env.NODE_ENV === 'production') {
    return {
      isValid: true,
      mode: 'mock' as const,
      errors: [],
      warnings: [],
    };
  }
  
  const result = validateConfiguration();
  
  // Log validation results in development
  if (typeof window !== 'undefined' && process.env.NODE_ENV !== 'test') {
    logConfigurationValidation(result);
  }
  
  return result;
}

/**
 * Configuration validation hook for React components.
 * 
 * This hook provides a convenient way to access configuration validation
 * results within React components and handle configuration-related UI states.
 * 
 * @returns Configuration validation result and loading state
 */
export function useConfigValidation() {
  const [validationResult, setValidationResult] = React.useState(() => 
    performStartupValidation()
  );
  
  React.useEffect(() => {
    // Re-validate when environment variables might change
    // (typically during development when .env.local is modified)
    const handleVisibilityChange = () => {
      if (document.visibilityState === 'visible') {
        const newResult = performStartupValidation();
        setValidationResult(prev => {
          // Only update if something actually changed
          if (
            prev.isValid !== newResult.isValid ||
            prev.mode !== newResult.mode ||
            JSON.stringify(prev.errors) !== JSON.stringify(newResult.errors) ||
            JSON.stringify(prev.warnings) !== JSON.stringify(newResult.warnings)
          ) {
            return newResult;
          }
          return prev;
        });
      }
    };
    
    document.addEventListener('visibilitychange', handleVisibilityChange);
    
    return () => {
      document.removeEventListener('visibilitychange', handleVisibilityChange);
    };
  }, []);
  
  return validationResult;
}

/**
 * Configuration validation utility for environment variable documentation.
 * 
 * This function generates comprehensive documentation about the current
 * configuration state for debugging and educational purposes.
 * 
 * @returns Formatted configuration documentation string
 */
export function generateConfigDocumentation(): string {
  const result = validateConfiguration();
  
  const lines: string[] = [
    '# Poblysh Connectors Demo Configuration',
    '',
    `Current Mode: ${result.mode.toUpperCase()}`,
    `Configuration Valid: ${result.isValid ? '✅ YES' : '❌ NO'}`,
    '',
  ];
  
  if (result.apiBaseUrl) {
    lines.push(`API Base URL: ${result.apiBaseUrl}`);
    lines.push('');
  }
  
  if (result.errors.length > 0) {
    lines.push('## Configuration Errors');
    lines.push('');
    result.errors.forEach((error, index) => {
      lines.push(`${index + 1}. ${error}`);
    });
    lines.push('');
  }
  
  if (result.warnings.length > 0) {
    lines.push('## Configuration Warnings');
    lines.push('');
    result.warnings.forEach((warning, index) => {
      lines.push(`${index + 1}. ${warning}`);
    });
    lines.push('');
  }
  
  if (result.suggestions && result.suggestions.length > 0) {
    lines.push('## Suggested Actions');
    lines.push('');
    result.suggestions.forEach((suggestion, index) => {
      lines.push(`${index + 1}. ${suggestion}`);
    });
    lines.push('');
  }
  
  lines.push('## Environment Variables');
  lines.push('');
  lines.push('```bash');
  lines.push('# Demo mode (mock or real)');
  lines.push(`NEXT_PUBLIC_DEMO_MODE=${process.env.NEXT_PUBLIC_DEMO_MODE || 'mock'}`);
  lines.push('');
  lines.push('# API base URL (required for real mode)');
  lines.push(`CONNECTORS_API_BASE_URL=${process.env.CONNECTORS_API_BASE_URL || '(not set)'}`);
  lines.push('```');
  lines.push('');
  
  lines.push('## Getting Help');
  lines.push('');
  lines.push('- See `.env.example` files for configuration examples');
  lines.push('- Check the README for detailed setup instructions');
  lines.push('- Review inline documentation in `lib/demo/demoConfig.ts`');
  lines.push('');
  
  return lines.join('\n');
}

/**
 * Development configuration check utility.
 * 
 * This function provides a quick check that can be used in development
 * to ensure the demo is properly configured for the intended mode.
 * 
 * @returns Check result with actionable messages
 */
export function performDevelopmentCheck() {
  const result = validateConfiguration();
  
  if (result.isValid) {
    return {
      status: 'success' as const,
      message: `Demo is properly configured in ${result.mode} mode`,
      details: result.mode === 'real' 
        ? `Connected to: ${result.apiBaseUrl}`
        : 'Using mock data (no external dependencies)',
    };
  }
  
  // Provide specific error guidance
  if (result.mode === 'real' && !result.apiBaseUrl) {
    return {
      status: 'error' as const,
      message: 'Real mode requires API base URL',
      details: 'Set CONNECTORS_API_BASE_URL to your Connectors API endpoint, or switch to mock mode',
    };
  }
  
  if (result.mode === 'real' && result.apiBaseUrl && !result.apiBaseUrl.startsWith('https://')) {
    return {
      status: 'error' as const,
      message: 'API base URL must use HTTPS',
      details: 'Update CONNECTORS_API_BASE_URL to use HTTPS protocol',
    };
  }
  
  return {
    status: 'error' as const,
    message: 'Configuration has errors',
    details: result.errors.join('; '),
  };
}