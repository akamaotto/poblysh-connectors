'use client';

import { useState, useEffect } from 'react';
import { useRouter } from 'next/navigation';
import { useDemoUser, useDemoTenant, useDemoConnections, useDemoSignals, useDemoDispatch, addSignals, setLoading, setError } from '@/lib/demo/state';
import { DemoSignal } from '@/lib/demo/types';
import { SignalList } from '@/components/demo/SignalList';
import { generateMockSignals, simulateDelay } from '@/lib/demo/mockData';

/**
 * Signals list page.
 * Shows discovered signals from connected services with filtering and search capabilities.
 */
export default function SignalsPage() {
  const [isScanningAll, setIsScanningAll] = useState(false);
  
  const user = useDemoUser();
  const tenant = useDemoTenant();
  const connections = useDemoConnections();
  const signals = useDemoSignals();
  const dispatch = useDemoDispatch();
  const router = useRouter();

  // Redirect to login if no user or tenant
  useEffect(() => {
    if (!user || !tenant) {
      router.push('/');
    }
  }, [user, tenant, router]);

  // Redirect to integrations if no connections
  useEffect(() => {
    if (user && tenant && connections.length === 0) {
      router.push('/integrations');
    }
  }, [user, tenant, connections, router]);

  const handleScanAll = async () => {
    if (!tenant || connections.length === 0) return;

    setIsScanningAll(true);
    dispatch(setLoading('signals', true));
    dispatch(setError('signals', undefined));

    try {
      // Simulate scanning delay
      await simulateDelay('SCAN');
      
      // Generate signals for all connected connections
      const connectedConnections = connections.filter(c => c.status === 'connected');
      const allNewSignals: DemoSignal[] = [];
      
      for (const connection of connectedConnections) {
        const connectionSignals = generateMockSignals(connection);
        allNewSignals.push(...connectionSignals);
      }
      
      // Add all new signals to state
      dispatch(addSignals(allNewSignals));
      
      // Update last sync time for all connections
      for (const connection of connectedConnections) { // eslint-disable-line @typescript-eslint/no-unused-vars
        // In a real implementation, we'd update each connection's lastSyncAt
      }
    } catch {
      dispatch(setError('signals', 'Failed to scan for signals'));
    } finally {
      setIsScanningAll(false);
      dispatch(setLoading('signals', false));
    }
  };

  const handleSignalClick = (signalId: string) => {
    router.push(`/signals/${signalId}`);
  };

  if (!user || !tenant) {
    return null;
  }

  const hasActiveConnections = connections.some(c => c.status === 'connected');
  const hasSignals = signals.length > 0;

  return (
    <div className="min-h-screen bg-white py-12 px-4">
      <div className="max-w-6xl mx-auto">
        {/* Header */}
        <div className="text-center mb-8">
          <h1 className="text-3xl font-bold text-black mb-4">
            Signal Discovery
          </h1>
          <p className="text-lg text-gray-600">
            Explore activity from your connected services
          </p>
        </div>

        {/* Status Overview */}
        <div className="bg-white border border-gray-200 rounded-lg p-6 mb-8">
          <div className="grid md:grid-cols-4 gap-6">
            <div className="text-center">
              <div className="text-2xl font-bold text-black">
                {signals.length}
              </div>
              <div className="text-sm text-gray-600">Total Signals</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold text-black">
                {connections.filter(c => c.status === 'connected').length}
              </div>
              <div className="text-sm text-gray-600">Active Connections</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold text-black">
                {[...new Set(signals.map(s => s.providerSlug))].length}
              </div>
              <div className="text-sm text-gray-600">Providers</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold text-black">
                {[...new Set(signals.map(s => s.kind))].length}
              </div>
              <div className="text-sm text-gray-600">Signal Types</div>
            </div>
          </div>
        </div>

        {/* Main Content */}
        {hasActiveConnections ? (
          <SignalList
            signals={signals}
            connections={connections}
            onSignalClick={handleSignalClick}
            onScanAll={handleScanAll}
            isScanning={isScanningAll}
          />
        ) : (
          <div className="bg-gray-50 border border-gray-200 rounded-lg p-8 text-center">
            <div className="w-16 h-16 bg-gray-100 rounded-full flex items-center justify-center mx-auto mb-4">
              <svg className="w-8 h-8 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
              </svg>
            </div>
            <h3 className="text-lg font-semibold text-black mb-2">
              No Active Connections
            </h3>
            <p className="text-gray-800 mb-6">
              Connect at least one service to start discovering signals.
            </p>
            <button
              onClick={() => router.push('/integrations')}
              className="inline-flex items-center px-6 py-3 bg-black hover:bg-gray-800 text-white font-medium rounded-lg transition-colors"
            >
              Set Up Integrations
              <svg className="ml-2 w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 7l5 5m0 0l-5 5m5-5H6" />
              </svg>
            </button>
          </div>
        )}

        {/* Educational Content */}
        <div className="mt-8 bg-white border border-gray-200 rounded-lg p-6">
          <h3 className="text-lg font-semibold text-black mb-4">
            Understanding Signals
          </h3>
          <div className="grid md:grid-cols-2 gap-6">
            <div>
              <h4 className="font-medium text-black mb-2">What are Signals?</h4>
              <p className="text-sm text-gray-600 mb-4">
                Signals represent activities, events, and changes from your connected services. They provide raw data about what&apos;s happening across your tools.
              </p>
              
              <h4 className="font-medium text-black mb-2">Signal Types</h4>
              <p className="text-sm text-gray-600">
                Different providers generate different types of signals: commits, pull requests, issues from GitHub; messages, mentions from Zoho Cliq; and more.
              </p>
            </div>
            
            <div>
              <h4 className="font-medium text-black mb-2">Relevance Scoring</h4>
              <p className="text-sm text-gray-600 mb-4">
                Each signal receives a relevance score based on factors like recency, author importance, and content analysis. Higher scores indicate more important signals.
              </p>
              
              <h4 className="font-medium text-black mb-2">Next: Signal Grounding</h4>
              <p className="text-sm text-gray-600">
                Click on any signal to see how grounding works with cross-provider evidence.
              </p>
            </div>
          </div>
        </div>

        {/* Quick Actions */}
        {hasSignals && (
          <div className="mt-8 bg-gray-50 border border-gray-200 rounded-lg p-6">
            <h3 className="text-lg font-semibold text-black mb-4">
              Ready for Signal Grounding?
            </h3>
            <p className="text-gray-800 mb-6">
              Click on any signal to see how grounding works with cross-provider evidence.
            </p>
            <div className="flex flex-wrap gap-4">
              <div className="flex items-center space-x-2 text-sm text-gray-700">
                <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                <span>Click any signal to view details</span>
              </div>
              <div className="flex items-center space-x-2 text-sm text-gray-700">
                <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                <span>Try &quot;Ground this signal&quot; to see cross-provider evidence</span>
              </div>
              <div className="flex items-center space-x-2 text-sm text-gray-700">
                <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                <span>Multiple connections provide stronger evidence</span>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}