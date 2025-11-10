'use client';

import { useState, useEffect } from 'react';
import { useRouter } from 'next/navigation';
import { useDemoUser, useDemoTenant, useDemoConnections, useDemoDispatch, addConnection, updateConnection, setSignals, setLoading, setError } from '@/lib/demo/state';
import { ProviderTile } from '@/components/demo/ProviderTile';
import { DEMO_PROVIDERS } from '@/lib/demo/types';
import { generateMockConnection, generateMockSignals, simulateDelay } from '@/lib/demo/mockData';

/**
 * Integrations hub page.
 * Allows users to connect and manage service integrations.
 */
export default function IntegrationsPage() {
  const [isConnecting, setIsConnecting] = useState<string | null>(null);
  const [isScanning, setIsScanning] = useState<string | null>(null);
  
  const user = useDemoUser();
  const tenant = useDemoTenant();
  const connections = useDemoConnections();
  const dispatch = useDemoDispatch();
  const router = useRouter();

  // Redirect to login if no user or tenant
  useEffect(() => {
    if (!user || !tenant) {
      router.push('/');
    }
  }, [user, tenant, router]);

  // Redirect to signals if no connections
  useEffect(() => {
    if (user && tenant && connections.length === 0) {
      // Don't redirect automatically - let user connect services
    }
  }, [user, tenant, connections, router]);

  const handleConnect = async (providerSlug: string) => {
    if (!tenant) return;

    setIsConnecting(providerSlug);
    dispatch(setLoading('connections', true));
    dispatch(setError('connections', undefined));

    try {
      // Simulate OAuth flow delay
      await simulateDelay('SLOW');
      
      // Create mock connection
      const connection = generateMockConnection(tenant.id, providerSlug, 'connected');
      dispatch(addConnection(connection));
      
      // Show success message briefly
      setTimeout(() => {
        setIsConnecting(null);
      }, 1000);
    } catch {
      dispatch(setError('connections', 'Failed to connect integration'));
      setIsConnecting(null);
    } finally {
      dispatch(setLoading('connections', false));
    }
  };

  const handleDisconnect = async (connectionId: string) => {
    dispatch(setLoading('connections', true));
    
    try {
      // Simulate API delay
      await simulateDelay('NORMAL');
      
      // Update connection to disconnected status
      dispatch(updateConnection(connectionId, { status: 'disconnected' }));
    } catch {
      dispatch(setError('connections', 'Failed to disconnect integration'));
    } finally {
      dispatch(setLoading('connections', false));
    }
  };

  const handleScan = async (connectionId: string) => {
    if (!tenant) return;

    setIsScanning(connectionId);
    dispatch(setLoading('signals', true));
    dispatch(setError('signals', undefined));

    try {
      // Simulate scanning delay
      await simulateDelay('SCAN');
      
      // Find the connection
      const connection = connections.find(c => c.id === connectionId);
      if (!connection) {
        throw new Error('Connection not found');
      }

      // Generate mock signals for this connection
      const signals = generateMockSignals(connection);
      
      // Add signals to existing list (or set if first time)
      dispatch(setSignals(signals));
      
      // Update connection's last sync time
      dispatch(updateConnection(connectionId, { 
        lastSyncAt: new Date().toISOString() 
      }));
      
      // Show success briefly
      setTimeout(() => {
        setIsScanning(null);
      }, 1000);
    } catch {
      dispatch(setError('signals', 'Failed to scan for signals'));
      setIsScanning(null);
    } finally {
      dispatch(setLoading('signals', false));
    }
  };

  const hasAnyConnections = connections.length > 0;
  const hasActiveConnections = connections.some(c => c.status === 'connected');

  if (!user || !tenant) {
    return null;
  }

  return (
    <div className="min-h-screen bg-white py-12 px-4">
      <div className="max-w-6xl mx-auto">
        {/* Header */}
        <div className="text-center mb-8">
          <h1 className="text-3xl font-bold text-black mb-4">
            Connect Your Services
          </h1>
          <p className="text-lg text-gray-600">
            Integrate your tools to discover and ground signals
          </p>
        </div>

        {/* Status Overview */}
        <div className="bg-white border border-gray-200 rounded-lg p-6 mb-8">
          <div className="grid md:grid-cols-3 gap-6">
            <div className="text-center">
              <div className="text-2xl font-bold text-black">
                {connections.length}
              </div>
              <div className="text-sm text-gray-600">Total Connections</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold text-black">
                {connections.filter(c => c.status === 'connected').length}
              </div>
              <div className="text-sm text-gray-600">Active Connections</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold text-black">
                {connections.filter(c => c.lastSyncAt).length}
              </div>
              <div className="text-sm text-gray-600">Recently Synced</div>
            </div>
          </div>
        </div>

        {/* Provider Grid */}
        <div className="grid md:grid-cols-2 gap-6 mb-8">
          {DEMO_PROVIDERS.map((provider) => {
            const connection = connections.find(c => c.providerSlug === provider.slug);
            const isProviderConnecting = isConnecting === provider.slug;
            const isProviderScanning = isScanning === connection?.id;
            
            return (
              <ProviderTile
                key={provider.slug}
                provider={provider}
                connection={connection}
                onConnect={handleConnect}
                onDisconnect={handleDisconnect}
                onScan={handleScan}
                isLoading={isProviderConnecting || isProviderScanning}
              />
            );
          })}
        </div>

        {/* Next Steps */}
        {hasActiveConnections && (
          <div className="bg-gray-50 border border-gray-200 rounded-lg p-6 text-center">
            <h3 className="text-lg font-semibold text-black mb-4">
              Ready to Discover Signals!
            </h3>
            <p className="text-gray-800 mb-6">
              You have active connections. Scan for signals to see activity from your integrated services.
            </p>
            <button
              onClick={() => router.push('/signals')}
              className="inline-flex items-center px-6 py-3 bg-black hover:bg-gray-800 text-white font-medium rounded-lg transition-colors"
            >
              View Signals
              <svg className="ml-2 w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 7l5 5m0 0l-5 5m5-5H6" />
              </svg>
            </button>
          </div>
        )}

        {/* No Connections State */}
        {!hasAnyConnections && (
          <div className="bg-gray-50 border border-gray-200 rounded-lg p-8 text-center">
            <div className="w-16 h-16 bg-gray-100 rounded-full flex items-center justify-center mx-auto mb-4">
              <svg className="w-8 h-8 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
              </svg>
            </div>
            <h3 className="text-lg font-semibold text-black mb-2">
              Start Connecting Your Services
            </h3>
            <p className="text-gray-800 mb-4">
              Connect at least one service to begin discovering signals and experiencing the grounding capabilities.
            </p>
            <p className="text-sm text-gray-700">
              Try connecting GitHub first to see code-related signals, then add Zoho Cliq for collaboration signals.
            </p>
          </div>
        )}

        {/* Educational Content */}
        <div className="mt-8 bg-white border border-gray-200 rounded-lg p-6">
          <h3 className="text-lg font-semibold text-black mb-4">
            Understanding Integrations
          </h3>
          <div className="grid md:grid-cols-2 gap-6">
            <div>
              <h4 className="font-medium text-black mb-2">What are Connections?</h4>
              <p className="text-sm text-gray-600 mb-4">
                Connections represent authorized links between your tenant and external services. Each connection enables signal discovery from that service.
              </p>
              
              <h4 className="font-medium text-black mb-2">OAuth Authentication</h4>
              <p className="text-sm text-gray-600">
                In production, connections use OAuth 2.0 to securely authorize access to your accounts without storing passwords.
              </p>
            </div>
            
            <div>
              <h4 className="font-medium text-black mb-2">Signal Discovery</h4>
              <p className="text-sm text-gray-600 mb-4">
                Once connected, the service can scan for activities like commits, pull requests, messages, and other signals from your integrated tools.
              </p>
              
              <h4 className="font-medium text-black mb-2">Multi-Provider Grounding</h4>
              <p className="text-sm text-gray-600">
                The real power comes from connecting multiple services. Signals from different providers can corroborate and strengthen each other.
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}