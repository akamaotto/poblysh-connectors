'use client';

import { useState } from 'react';
import { DemoConnection, DemoProvider } from '@/lib/demo/types';

/**
 * ProviderTile component.
 * Displays integration options for a specific provider with connection status.
 */
interface ProviderTileProps {
  provider: DemoProvider;
  connection?: DemoConnection;
  onConnect: (providerSlug: string) => void;
  onDisconnect: (connectionId: string) => void;
  onScan: (connectionId: string) => void;
  isLoading?: boolean;
}

export function ProviderTile({
  provider,
  connection,
  onConnect,
  onDisconnect,
  onScan,
  isLoading = false,
}: ProviderTileProps) {
  const [showOAuthModal, setShowOAuthModal] = useState(false);

  const isConnected = connection?.status === 'connected';
  const isConnecting = isLoading && !connection;

  const handleConnectClick = () => {
    if (isConnected) {
      // Show disconnect confirmation or directly disconnect
      onDisconnect(connection!.id);
    } else {
      // Show OAuth modal for connection
      setShowOAuthModal(true);
    }
  };

  const handleOAuthConfirm = async () => {
    setShowOAuthModal(false);
    onConnect(provider.slug);
  };

  return (
    <>
      <div
        className="bg-white rounded-lg shadow-md border border-gray-200 p-6 hover:shadow-lg transition-shadow"
        role="article"
        aria-labelledby={`provider-${provider.slug}-name`}
        aria-describedby={`provider-${provider.slug}-description`}
      >
        {/* Provider Header */}
        <div className="flex items-center justify-between mb-4">
          <div className="flex items-center space-x-3">
            <div className="w-12 h-12 bg-gray-100 rounded-lg flex items-center justify-center" aria-hidden="true">
              {/* Placeholder icon - in real implementation, use provider.iconUrl */}
              <span className="text-lg font-bold text-gray-600">
                {provider.name.charAt(0)}
              </span>
            </div>
            <div>
              <h3
                id={`provider-${provider.slug}-name`}
                className="text-lg font-semibold text-black"
              >
                {provider.name}
              </h3>
              <p
                id={`provider-${provider.slug}-description`}
                className="text-sm text-gray-600"
              >
                {provider.description}
              </p>
            </div>
          </div>
          
          {/* Connection Status Badge */}
          <div className={`px-3 py-1 rounded-full text-xs font-medium ${
            isConnected 
              ? 'bg-gray-100 text-gray-800' 
              : connection?.status === 'error'
              ? 'bg-red-50 text-red-700'
              : 'bg-gray-50 text-gray-600'
          }`}>
            {isConnected ? 'Connected' : connection?.status === 'error' ? 'Error' : 'Not Connected'}
          </div>
        </div>

        {/* Connection Info */}
        {isConnected && connection && (
          <div className="mb-4 p-3 bg-gray-50 border border-gray-200 rounded-md">
            <div className="text-sm text-gray-800">
              <p className="font-medium">Connection Active</p>
              <p className="text-xs mt-1">
                Connected on {new Date(connection.createdAt).toLocaleDateString()}
                {connection.lastSyncAt && (
                  <span> • Last sync {new Date(connection.lastSyncAt).toLocaleDateString()}</span>
                )}
              </p>
            </div>
          </div>
        )}

        {/* Error Info */}
        {connection?.status === 'error' && (
          <div className="mb-4 p-3 bg-red-50 border border-red-200 text-red-700 rounded-md">
            <div className="text-sm">
              <p className="font-medium">Connection Error</p>
              <p className="text-xs mt-1">{connection.error || 'Unknown error occurred'}</p>
            </div>
          </div>
        )}

        {/* Supported Signal Kinds */}
        <div className="mb-6">
          <h4 className="text-sm font-medium text-gray-900 mb-2">Supported Signals:</h4>
          <div className="flex flex-wrap gap-1">
            {provider.supportedSignalKinds.map((kind) => (
              <span
                key={kind}
                className="px-2 py-1 bg-gray-100 text-gray-600 text-xs rounded-full"
              >
                {kind.replace('_', ' ')}
              </span>
            ))}
          </div>
        </div>

        {/* Action Buttons */}
        <div className="flex space-x-3">
          <button
            onClick={handleConnectClick}
            disabled={isConnecting || isLoading}
            className={`flex-1 py-2 px-4 rounded-md text-sm font-medium transition-colors ${
              isConnected
                ? 'border border-red-600 text-red-600 hover:bg-red-50 hover:border-red-700 hover:text-red-700 bg-white'
                : 'bg-black hover:bg-gray-800 text-white disabled:opacity-50 disabled:cursor-not-allowed'
            }`}
          >
            {isConnecting ? (
              <span className="flex items-center justify-center">
                <svg className={`animate-spin -ml-1 mr-2 h-4 w-4 ${isConnected ? 'text-red-600' : 'text-white'}`} xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                  <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                  <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                </svg>
                {isConnected ? 'Disconnecting...' : 'Connecting...'}
              </span>
            ) : isConnected ? (
              'Disconnect'
            ) : (
              'Connect'
            )}
          </button>

          {isConnected && (
            <button
              onClick={() => onScan(connection.id)}
              disabled={isLoading}
              className="flex-1 py-2 px-4 bg-black hover:bg-gray-800 text-white rounded-md text-sm font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              Scan for Signals
            </button>
          )}
        </div>

        {/* Educational Info */}
        <div className="mt-4 p-3 bg-gray-50 border border-gray-200 rounded-md">
          <div className="flex">
            <div className="flex-shrink-0">
              <svg className="h-4 w-4 text-gray-400" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor">
                <path fillRule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clipRule="evenodd" />
              </svg>
            </div>
            <div className="ml-2">
              <p className="text-xs text-gray-700">
                <strong>In Production:</strong> This would trigger a real OAuth flow to {provider.name} and create an API connection.
              </p>
            </div>
          </div>
        </div>
      </div>

      {/* OAuth Modal (Mock) */}
      {showOAuthModal && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg p-6 max-w-md w-full mx-4">
            <h3 className="text-lg font-bold text-black mb-4">
              Connect to {provider.name}
            </h3>
            
            <div className="mb-6">
              <p className="text-sm text-gray-600 mb-4">
                This will authorize Poblysh Connectors to access your {provider.name} account to discover signals.
              </p>
              
              {/* Mock OAuth Scopes */}
              <div className="bg-gray-50 rounded-lg p-4">
                <h4 className="text-sm font-medium text-gray-900 mb-2">Permissions Requested:</h4>
                <ul className="text-xs text-gray-600 space-y-1">
                  <li>• Read repository information</li>
                  <li>• Access commit history</li>
                  <li>• View pull requests and issues</li>
                  <li>• Read organization data</li>
                </ul>
              </div>
            </div>

            <div className="flex space-x-3">
              <button
                onClick={() => setShowOAuthModal(false)}
                className="flex-1 py-2 px-4 bg-gray-100 hover:bg-gray-200 text-gray-800 rounded-md text-sm font-medium transition-colors"
              >
                Cancel
              </button>
              <button
                onClick={handleOAuthConfirm}
                className="flex-1 py-2 px-4 bg-black hover:bg-gray-800 text-white rounded-md text-sm font-medium transition-colors"
              >
                Authorize {provider.name}
              </button>
            </div>

            <div className="mt-4 p-3 bg-gray-50 border border-gray-200 rounded-md">
              <p className="text-xs text-gray-700">
                <strong>Mock OAuth:</strong> This is a simulation. No real authorization will occur.
              </p>
            </div>
          </div>
        </div>
      )}
    </>
  );
}