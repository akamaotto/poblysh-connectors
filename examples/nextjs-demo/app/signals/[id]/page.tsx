'use client';

import { useState, useEffect } from 'react';
import { useRouter, useParams } from 'next/navigation';
import { useDemoUser, useDemoTenant, useDemoConnections, useDemoSignals, useDemoGroundedSignals, useDemoDispatch, addGroundedSignal, setLoading, setError } from '@/lib/demo/state';
import { SignalDetail } from '@/components/demo/SignalDetail';
import { generateGroundedSignal, simulateDelay } from '@/lib/demo/mockData';

/**
 * Signal detail page.
 * Shows comprehensive information about a specific signal with grounding capabilities.
 */
export default function SignalDetailPage() {
  const [isGrounding, setIsGrounding] = useState(false);
  
  const params = useParams();
  const router = useRouter();
  const signalId = params.id as string;

  const user = useDemoUser();
  const tenant = useDemoTenant();
  const connections = useDemoConnections();
  const signals = useDemoSignals();
  const groundedSignals = useDemoGroundedSignals();
  const dispatch = useDemoDispatch();

  // Find the signal and related connection
  const signal = signals.find(s => s.id === signalId);
  const connection = signal ? connections.find(c => c.id === signal.connectionId) : undefined;
  const groundedSignal = groundedSignals.find(gs => gs.sourceSignalId === signalId);

  // Redirect to login if no user or tenant
  useEffect(() => {
    if (!user || !tenant) {
      router.push('/');
    }
  }, [user, tenant, router]);

  // Redirect to signals list if signal not found
  useEffect(() => {
    if (user && tenant && !signal) {
      router.push('/signals');
    }
  }, [user, tenant, signal, router]);

  const handleGround = async (signalIdToGround: string) => { // eslint-disable-line @typescript-eslint/no-unused-vars
    if (!tenant || !signal) return;

    setIsGrounding(true);
    dispatch(setLoading('grounding', true));
    dispatch(setError('grounding', undefined));

    try {
      // Simulate grounding delay
      await simulateDelay('GROUND');
      
      // Generate grounded signal
      const newGroundedSignal = generateGroundedSignal(signal, signals, connections);
      dispatch(addGroundedSignal(newGroundedSignal));
    } catch {
      dispatch(setError('grounding', 'Failed to ground signal'));
    } finally {
      setIsGrounding(false);
      dispatch(setLoading('grounding', false));
    }
  };

  const handleBack = () => {
    router.back();
  };

  if (!user || !tenant || !signal || !connection) {
    return null;
  }

  return (
    <div className="min-h-screen bg-gray-50 py-12 px-4">
      <div className="max-w-4xl mx-auto">
        {/* Back Navigation */}
        <div className="mb-6">
          <button
            onClick={handleBack}
            className="inline-flex items-center text-gray-600 hover:text-gray-900 transition-colors"
          >
            <svg className="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
            </svg>
            Back to Signals
          </button>
        </div>

        {/* Signal Detail */}
        <SignalDetail
          signal={signal}
          connection={connection}
          groundedSignal={groundedSignal}
          onGround={handleGround}
          isGrounding={isGrounding}
        />

        {/* Educational Content */}
        <div className="mt-8 bg-white rounded-lg shadow-md p-6">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">
            Understanding Signal Grounding
          </h3>
          <div className="grid md:grid-cols-2 gap-6">
            <div>
              <h4 className="font-medium text-gray-900 mb-2">What is Grounding?</h4>
              <p className="text-sm text-gray-600 mb-4">
                Signal grounding is the process of analyzing a signal across all connected providers to find supporting evidence,
                calculate confidence scores, and provide dimensional analysis of the signal&apos;s importance and reliability.
              </p>
              
              <h4 className="font-medium text-gray-900 mb-2">Evidence Discovery</h4>
              <p className="text-sm text-gray-600">
                The grounding engine searches for related activities, mentions, references, and cross-provider correlations 
                that strengthen or weaken the original signal.
              </p>
            </div>
            
            <div>
              <h4 className="font-medium text-gray-900 mb-2">Confidence Scoring</h4>
              <p className="text-sm text-gray-600 mb-4">
                Each grounded signal receives an overall confidence score based on multiple dimensions: relevance, impact, 
                timeliness, authority, and corroboration from evidence.
              </p>
              
              <h4 className="font-medium text-gray-900 mb-2">Multi-Provider Value</h4>
              <p className="text-sm text-gray-600">
                The more providers you connect, the stronger the grounding results become. Cross-provider evidence 
                provides the highest confidence scores and most reliable signal analysis.
              </p>
            </div>
          </div>
        </div>

        {/* Next Steps */}
        {groundedSignal && (
          <div className="mt-8 bg-green-50 border border-green-200 rounded-lg p-6">
            <h3 className="text-lg font-semibold text-green-900 mb-4">
              Signal Grounded Successfully!
            </h3>
            <p className="text-green-800 mb-6">
              This signal has been analyzed with {groundedSignal.evidence.length} pieces of supporting evidence 
              and received a confidence score of {groundedSignal.score}%.
            </p>
            <div className="flex flex-wrap gap-4">
              <button
                onClick={() => router.push('/signals')}
                className="px-4 py-2 bg-green-600 hover:bg-green-700 text-white rounded-md text-sm font-medium transition-colors"
              >
                Explore More Signals
              </button>
              <button
                onClick={() => router.push('/integrations')}
                className="px-4 py-2 bg-white hover:bg-gray-50 text-green-700 border border-green-300 rounded-md text-sm font-medium transition-colors"
              >
                Add More Integrations
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}