'use client';

import { useState } from 'react';
import { DemoSignal, DemoConnection, DemoGroundedSignal } from '@/lib/demo/types';

/**
 * SignalDetail component.
 * Shows comprehensive information about a signal with grounding capabilities.
 */
interface SignalDetailProps {
  signal: DemoSignal;
  connection: DemoConnection;
  groundedSignal?: DemoGroundedSignal;
  onGround: (signalId: string) => void;
  isGrounding?: boolean;
}

export function SignalDetail({
  signal,
  connection,
  groundedSignal,
  onGround,
  isGrounding = false,
}: SignalDetailProps) {
  const [showRawData, setShowRawData] = useState(false);

  const formatSignalKind = (kind: string) => {
    return kind.replace(/_/g, ' ').replace(/\b\w/g, l => l.toUpperCase());
  };

  const getProviderIcon = (providerSlug: string) => {
    return providerSlug.charAt(0).toUpperCase();
  };

  const getRelevanceColor = (score?: number) => {
    if (!score) return 'bg-gray-100 text-gray-600';
    if (score >= 80) return 'bg-gray-800 text-white';
    if (score >= 60) return 'bg-gray-600 text-white';
    return 'bg-gray-400 text-white';
  };

  const getConfidenceColor = (confidence: string) => {
    switch (confidence) {
      case 'high': return 'bg-gray-800 text-white';
      case 'medium': return 'bg-gray-600 text-white';
      case 'low': return 'bg-gray-400 text-white';
      default: return 'bg-gray-100 text-gray-600';
    }
  };

  return (
    <div className="space-y-6">
      {/* Signal Header */}
      <div className="bg-white border border-gray-200 rounded-lg p-6">
        <div className="flex items-start justify-between mb-6">
          <div className="flex items-start space-x-4">
            <div className="w-12 h-12 bg-gray-100 rounded-lg flex items-center justify-center">
              <span className="text-lg font-bold text-gray-600">
                {getProviderIcon(signal.providerSlug)}
              </span>
            </div>
            
            <div className="flex-1">
              <h1 className="text-2xl font-bold text-black mb-2">
                {signal.title}
              </h1>
              
              <div className="flex flex-wrap items-center gap-2 text-sm text-gray-600">
                <span className="font-medium">{signal.author}</span>
                <span>•</span>
                <span>{formatSignalKind(signal.kind)}</span>
                <span>•</span>
                <span>{new Date(signal.occurredAt).toLocaleDateString()}</span>
                <span>•</span>
                <span className={`px-2 py-1 rounded-full text-xs font-medium ${getRelevanceColor(signal.relevanceScore)}`}>
                  Relevance: {signal.relevanceScore || 0}%
                </span>
              </div>
            </div>
          </div>

          {signal.url && (
            <a
              href={signal.url}
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex items-center px-3 py-2 bg-gray-100 hover:bg-gray-200 text-gray-700 rounded-md text-sm font-medium transition-colors"
            >
              View Original
              <svg className="ml-2 w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0 0V6m0 0L10 14" />
              </svg>
            </a>
          )}
        </div>

        <div className="prose max-w-none">
          <p className="text-gray-700 leading-relaxed">
            {signal.summary}
          </p>
        </div>
      </div>

      {/* Grounding Section */}
      <div className="bg-white border border-gray-200 rounded-lg p-6">
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-xl font-semibold text-black">
            Signal Grounding
          </h2>
          
          {!groundedSignal && (
            <button
              onClick={() => onGround(signal.id)}
              disabled={isGrounding}
              className="px-4 py-2 bg-black hover:bg-gray-800 text-white rounded-md text-sm font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {isGrounding ? (
                <span className="flex items-center">
                  <svg className="animate-spin -ml-1 mr-2 h-4 w-4 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                    <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                    <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                  </svg>
                  Grounding...
                </span>
              ) : (
                'Ground This Signal'
              )}
            </button>
          )}
        </div>

        {!groundedSignal ? (
          <div className="text-center py-8">
            <div className="w-16 h-16 bg-gray-100 rounded-full flex items-center justify-center mx-auto mb-4">
              <svg className="w-8 h-8 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
            </div>
            <h3 className="text-lg font-medium text-black mb-2">
              Signal Not Grounded Yet
            </h3>
            <p className="text-gray-600 mb-4">
              Ground this signal to discover evidence from other providers and calculate a confidence score.
            </p>
            <button
              onClick={() => onGround(signal.id)}
              disabled={isGrounding}
              className="px-6 py-3 bg-black hover:bg-gray-800 text-white rounded-md text-sm font-medium transition-colors disabled:opacity-50"
            >
              {isGrounding ? 'Grounding...' : 'Ground This Signal'}
            </button>
          </div>
        ) : (
          <div className="space-y-6">
            {/* Grounding Results */}
            <div className="grid md:grid-cols-2 gap-6">
              <div>
                <h3 className="text-lg font-medium text-black mb-4">Overall Score</h3>
                <div className="text-center">
                  <div className={`inline-flex items-center justify-center w-24 h-24 rounded-full text-3xl font-bold ${getConfidenceColor(groundedSignal.confidence)}`}>
                    {groundedSignal.score}%
                  </div>
                  <p className="mt-2 text-sm font-medium text-black">
                    {groundedSignal.confidence.charAt(0).toUpperCase() + groundedSignal.confidence.slice(1)} Confidence
                  </p>
                </div>
              </div>

              <div>
                <h3 className="text-lg font-medium text-black mb-4">Dimensional Scores</h3>
                <div className="space-y-3">
                  {groundedSignal.dimensions.map((dimension) => (
                    <div key={dimension.label}>
                      <div className="flex justify-between text-sm mb-1">
                        <span className="font-medium text-gray-900">{dimension.label}</span>
                        <span className="text-gray-600">{dimension.score}%</span>
                      </div>
                      <div className="w-full bg-gray-200 rounded-full h-2">
                        <div
                          className="bg-black h-2 rounded-full"
                          style={{ width: `${dimension.score}%` }}
                        />
                      </div>
                      <p className="text-xs text-gray-500 mt-1">{dimension.description}</p>
                    </div>
                  ))}
                </div>
              </div>
            </div>

            {/* Evidence */}
            <div>
              <h3 className="text-lg font-medium text-black mb-4">
                Supporting Evidence ({groundedSignal.evidence.length})
              </h3>
              
              {groundedSignal.evidence.length === 0 ? (
                <div className="text-center py-6 bg-gray-50 rounded-lg">
                  <p className="text-gray-600">No supporting evidence found from other providers.</p>
                </div>
              ) : (
                <div className="space-y-4">
                  {groundedSignal.evidence.map((evidence) => (
                    <div key={evidence.id} className="border border-gray-200 rounded-lg p-4">
                      <div className="flex items-start justify-between mb-2">
                        <div className="flex items-center space-x-2">
                          <span className="px-2 py-1 bg-gray-100 text-gray-800 text-xs rounded-full">
                            {evidence.providerSlug}
                          </span>
                          <span className="px-2 py-1 bg-gray-100 text-gray-600 text-xs rounded-full">
                            {evidence.type.replace('_', ' ')}
                          </span>
                          <span className="px-2 py-1 bg-gray-100 text-gray-800 text-xs rounded-full">
                            Strength: {evidence.strength}%
                          </span>
                        </div>
                      </div>
                      <p className="text-sm text-gray-700">{evidence.description}</p>
                    </div>
                  ))}
                </div>
              )}
            </div>

            {/* Summary */}
            <div className="bg-gray-50 border border-gray-200 rounded-lg p-4">
              <h4 className="font-medium text-black mb-2">Grounding Summary</h4>
              <p className="text-sm text-gray-800">{groundedSignal.summary}</p>
            </div>
          </div>
        )}

        {/* Educational Info */}
        <div className="mt-6 p-4 bg-gray-50 border border-gray-200 rounded-md">
          <div className="flex">
            <div className="flex-shrink-0">
              <svg className="h-5 w-5 text-gray-400" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor">
                <path fillRule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clipRule="evenodd" />
              </svg>
            </div>
            <div className="ml-3">
              <p className="text-sm text-gray-700">
                <strong>Signal Grounding:</strong> This process analyzes signals across all connected providers to find supporting evidence, 
                calculate confidence scores, and provide dimensional analysis. Multiple connected providers strengthen the grounding results.
              </p>
            </div>
          </div>
        </div>
      </div>

      {/* Signal Metadata */}
      <div className="bg-white border border-gray-200 rounded-lg p-6">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-xl font-semibold text-black">
            Signal Metadata
          </h2>
          <button
            onClick={() => setShowRawData(!showRawData)}
            className="px-3 py-1 bg-gray-100 hover:bg-gray-200 text-gray-700 rounded-md text-sm font-medium transition-colors"
          >
            {showRawData ? 'Hide' : 'Show'} Raw Data
          </button>
        </div>

        <div className="grid md:grid-cols-2 gap-6">
          <div>
            <h3 className="text-sm font-medium text-black mb-3">Basic Information</h3>
            <dl className="space-y-2 text-sm">
              <div className="flex justify-between">
                <dt className="text-gray-600">Signal ID:</dt>
                <dd className="font-mono text-gray-900">{signal.id.substring(0, 8)}...</dd>
              </div>
              <div className="flex justify-between">
                <dt className="text-gray-600">Provider:</dt>
                <dd className="text-gray-900">{signal.providerSlug}</dd>
              </div>
              <div className="flex justify-between">
                <dt className="text-gray-600">Connection:</dt>
                <dd className="text-gray-900">{connection.displayName}</dd>
              </div>
              <div className="flex justify-between">
                <dt className="text-gray-600">Occurred:</dt>
                <dd className="text-gray-900">{new Date(signal.occurredAt).toLocaleString()}</dd>
              </div>
              <div className="flex justify-between">
                <dt className="text-gray-600">Discovered:</dt>
                <dd className="text-gray-900">{new Date(signal.discoveredAt).toLocaleString()}</dd>
              </div>
            </dl>
          </div>

          <div>
            <h3 className="text-sm font-medium text-black mb-3">Technical Details</h3>
            <dl className="space-y-2 text-sm">
              <div className="flex justify-between">
                <dt className="text-gray-600">Signal Kind:</dt>
                <dd className="text-gray-900">{signal.kind}</dd>
              </div>
              <div className="flex justify-between">
                <dt className="text-gray-600">Tenant ID:</dt>
                <dd className="font-mono text-gray-900">{signal.tenantId.substring(0, 8)}...</dd>
              </div>
              <div className="flex justify-between">
                <dt className="text-gray-600">Connection ID:</dt>
                <dd className="font-mono text-gray-900">{signal.connectionId.substring(0, 8)}...</dd>
              </div>
              <div className="flex justify-between">
                <dt className="text-gray-600">Relevance Score:</dt>
                <dd className="text-gray-900">{signal.relevanceScore || 'N/A'}</dd>
              </div>
              <div className="flex justify-between">
                <dt className="text-gray-600">Has URL:</dt>
                <dd className="text-gray-900">{signal.url ? 'Yes' : 'No'}</dd>
              </div>
            </dl>
          </div>
        </div>

        {showRawData && (
          <div className="mt-6 pt-6 border-t border-gray-200">
            <h3 className="text-sm font-medium text-black mb-3">Raw Signal Data</h3>
            <pre className="bg-gray-50 p-4 rounded-md text-xs text-gray-700 overflow-x-auto">
              {JSON.stringify(signal, null, 2)}
            </pre>
          </div>
        )}
      </div>
    </div>
  );
}