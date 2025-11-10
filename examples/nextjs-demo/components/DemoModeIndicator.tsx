'use client';

import React from 'react';
import { useDemoConfig } from '@/lib/demo/state';
import { getApiModeInfo } from '@/lib/demo/apiRouter';

/**
 * Demo mode indicator component.
 * 
 * This component displays the current demo mode (mock vs real) and provides
 * configuration information for educational purposes. It helps developers
 * understand which mode they're currently using and whether the configuration
 * is valid.
 */
export function DemoModeIndicator() {
  const config = useDemoConfig();
  const modeInfo = getApiModeInfo();

  // Don't render in production or if configuration is not available
  if (typeof window === 'undefined' || !config) {
    return null;
  }

  const getModeColor = () => {
    if (!config.isConfigValid) return 'bg-red-100 border-red-300 text-red-800';
    return config.mode === 'mock' 
      ? 'bg-blue-100 border-blue-300 text-blue-800'
      : 'bg-green-100 border-green-300 text-green-800';
  };

  const getModeIcon = () => {
    if (!config.isConfigValid) return '⚠️';
    return config.mode === 'mock' ? '🎭' : '🌐';
  };

  const getModeLabel = () => {
    if (!config.isConfigValid) return 'Configuration Error';
    return config.mode === 'mock' ? 'Mock Mode' : 'Real API Mode';
  };

  const getModeDescription = () => {
    if (!config.isConfigValid) {
      return 'Demo configuration has errors that must be fixed';
    }
    return config.mode === 'mock' 
      ? 'Using locally generated mock data'
      : `Making real API calls to ${config.connectorsApiBaseUrl}`;
  };

  return (
    <div className={`border rounded-lg p-4 mb-4 ${getModeColor()}`}>
      <div className="flex items-center justify-between">
        <div className="flex items-center space-x-2">
          <span className="text-2xl">{getModeIcon()}</span>
          <div>
            <h3 className="font-semibold text-lg">{getModeLabel()}</h3>
            <p className="text-sm opacity-90">{getModeDescription()}</p>
          </div>
        </div>
        
        <div className="text-right">
          <div className="text-xs font-medium uppercase tracking-wide opacity-75">
            Environment
          </div>
          <div className="text-sm font-semibold">
            {process.env.NODE_ENV || 'development'}
          </div>
        </div>
      </div>

      {/* Configuration details */}
      <div className="mt-3 pt-3 border-t border-current border-opacity-20">
        <div className="text-sm space-y-1">
          <div>
            <span className="font-medium">Mode:</span> {config.mode}
          </div>
          
          {config.connectorsApiBaseUrl && (
            <div>
              <span className="font-medium">API URL:</span> {config.connectorsApiBaseUrl}
            </div>
          )}

          {config.configWarnings.length > 0 && (
            <div className="mt-2">
              <span className="font-medium">Warnings:</span>
              <ul className="mt-1 ml-4 list-disc text-xs">
                {config.configWarnings.map((warning, index) => (
                  <li key={index}>{warning}</li>
                ))}
              </ul>
            </div>
          )}

          {config.configErrors.length > 0 && (
            <div className="mt-2">
              <span className="font-medium text-red-700">Errors:</span>
              <ul className="mt-1 ml-4 list-disc text-xs">
                {config.configErrors.map((error, index) => (
                  <li key={index}>{error}</li>
                ))}
              </ul>
            </div>
          )}
        </div>
      </div>

      {/* Educational note */}
      <div className="mt-3 pt-3 border-t border-current border-opacity-20 text-xs opacity-75">
        <p>
          <strong>Development Note:</strong> This demo supports two modes:
        </p>
        <ul className="mt-1 ml-4 list-disc">
          <li><strong>Mock Mode:</strong> Uses generated data, no API calls required</li>
          <li><strong>Real Mode:</strong> Connects to actual Connectors API service</li>
        </ul>
        <p className="mt-2">
          Configure mode by setting <code>NEXT_PUBLIC_DEMO_MODE</code> in your environment.
          See <code>.env.example</code> files for configuration examples.
        </p>
      </div>
    </div>
  );
}