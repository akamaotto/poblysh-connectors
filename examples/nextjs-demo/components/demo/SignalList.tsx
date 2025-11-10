'use client';

import { useState } from 'react';
import { DemoSignal, DemoConnection } from '@/lib/demo/types';

/**
 * SignalList component.
 * Displays a list of discovered signals with filtering and sorting options.
 */
interface SignalListProps {
  signals: DemoSignal[];
  connections: DemoConnection[];
  onSignalClick: (signalId: string) => void;
  onScanAll?: () => void;
  isScanning?: boolean;
}

export function SignalList({
  signals,
  connections,
  onSignalClick,
  onScanAll,
  isScanning = false,
}: SignalListProps) {
  const [selectedProvider, setSelectedProvider] = useState<string>('all');
  const [selectedKind, setSelectedKind] = useState<string>('all');
  const [searchTerm, setSearchTerm] = useState('');
  const [sortBy, setSortBy] = useState<'recent' | 'relevance'>('recent');

  // Get unique providers and kinds for filters
  const providers = [...new Set(signals.map(s => s.providerSlug))];
  const kinds = [...new Set(signals.map(s => s.kind))];

  // Filter signals
  const filteredSignals = signals.filter(signal => {
    const matchesProvider = selectedProvider === 'all' || signal.providerSlug === selectedProvider;
    const matchesKind = selectedKind === 'all' || signal.kind === selectedKind;
    const matchesSearch = searchTerm === '' || 
      signal.title.toLowerCase().includes(searchTerm.toLowerCase()) ||
      signal.summary.toLowerCase().includes(searchTerm.toLowerCase()) ||
      signal.author.toLowerCase().includes(searchTerm.toLowerCase());
    
    return matchesProvider && matchesKind && matchesSearch;
  });

  // Sort signals
  const sortedSignals = [...filteredSignals].sort((a, b) => {
    if (sortBy === 'recent') {
      return new Date(b.occurredAt).getTime() - new Date(a.occurredAt).getTime();
    } else {
      return (b.relevanceScore || 0) - (a.relevanceScore || 0);
    }
  });

  const getConnectionName = (connectionId: string) => {
    const connection = connections.find(c => c.id === connectionId);
    return connection?.displayName || 'Unknown Connection';
  };

  const formatSignalKind = (kind: string) => {
    return kind.replace(/_/g, ' ').replace(/\b\w/g, l => l.toUpperCase());
  };

  const getProviderIcon = (providerSlug: string) => {
    // Simple text-based icon for now
    return providerSlug.charAt(0).toUpperCase();
  };

  const getRelevanceColor = (score?: number) => {
    if (!score) return 'bg-gray-100 text-gray-600';
    if (score >= 80) return 'bg-gray-800 text-white';
    if (score >= 60) return 'bg-gray-600 text-white';
    return 'bg-gray-400 text-white';
  };

  return (
    <div className="space-y-6">
      {/* Filters and Controls */}
      <div className="bg-white border border-gray-200 rounded-lg p-6">
        <div className="flex flex-col lg:flex-row lg:items-center lg:justify-between gap-4 mb-6">
          <h2 className="text-xl font-semibold text-black">
            Discovered Signals ({filteredSignals.length})
          </h2>
          
          {onScanAll && (
            <button
              onClick={onScanAll}
              disabled={isScanning}
              className="px-4 py-2 bg-black hover:bg-gray-800 text-white rounded-md text-sm font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {isScanning ? (
                <span className="flex items-center">
                  <svg className="animate-spin -ml-1 mr-2 h-4 w-4 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                    <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                    <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                  </svg>
                  Scanning...
                </span>
              ) : (
                'Scan All Connections'
              )}
            </button>
          )}
        </div>

        {/* Search */}
        <div className="mb-4">
          <input
            type="text"
            placeholder="Search signals..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-black focus:border-black"
          />
        </div>

        {/* Filters */}
        <div className="grid md:grid-cols-4 gap-4">
          <div>
            <label className="block text-sm font-medium text-gray-900 mb-1">Provider</label>
            <select
              value={selectedProvider}
              onChange={(e) => setSelectedProvider(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-black focus:border-black"
            >
              <option value="all">All Providers</option>
              {providers.map(provider => (
                <option key={provider} value={provider}>
                  {provider.charAt(0).toUpperCase() + provider.slice(1)}
                </option>
              ))}
            </select>
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-900 mb-1">Signal Type</label>
            <select
              value={selectedKind}
              onChange={(e) => setSelectedKind(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-black focus:border-black"
            >
              <option value="all">All Types</option>
              {kinds.map(kind => (
                <option key={kind} value={kind}>
                  {formatSignalKind(kind)}
                </option>
              ))}
            </select>
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-900 mb-1">Sort By</label>
            <select
              value={sortBy}
              onChange={(e) => setSortBy(e.target.value as 'recent' | 'relevance')}
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-black focus:border-black"
            >
              <option value="recent">Most Recent</option>
              <option value="relevance">Highest Relevance</option>
            </select>
          </div>

          <div className="flex items-end">
            <button
              onClick={() => {
                setSelectedProvider('all');
                setSelectedKind('all');
                setSearchTerm('');
                setSortBy('recent');
              }}
              className="w-full px-3 py-2 bg-gray-100 hover:bg-gray-200 text-gray-800 rounded-md text-sm font-medium transition-colors"
            >
              Clear Filters
            </button>
          </div>
        </div>
      </div>

      {/* Signals List */}
      {sortedSignals.length === 0 ? (
        <div className="bg-white border border-gray-200 rounded-lg p-8 text-center">
          <div className="w-16 h-16 bg-gray-100 rounded-full flex items-center justify-center mx-auto mb-4">
            <svg className="w-8 h-8 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
            </svg>
          </div>
          <h3 className="text-lg font-medium text-black mb-2">No Signals Found</h3>
          <p className="text-gray-600 mb-4">
            {signals.length === 0 
              ? "Connect services and scan for signals to see activity from your integrated tools."
              : "Try adjusting your filters or search terms to find signals."
            }
          </p>
          {signals.length === 0 && onScanAll && (
            <button
              onClick={onScanAll}
              disabled={isScanning}
              className="px-4 py-2 bg-black hover:bg-gray-800 text-white rounded-md text-sm font-medium transition-colors disabled:opacity-50"
            >
              Scan for Signals
            </button>
          )}
        </div>
      ) : (
        <div className="bg-white border border-gray-200 rounded-lg overflow-hidden">
          <div className="divide-y divide-gray-200">
            {sortedSignals.map((signal) => (
              <div
                key={signal.id}
                className="p-6 hover:bg-gray-50 cursor-pointer transition-colors"
                onClick={() => onSignalClick(signal.id)}
              >
                <div className="flex items-start justify-between">
                  <div className="flex items-start space-x-4 flex-1">
                    {/* Provider Icon */}
                    <div className="flex-shrink-0 w-10 h-10 bg-gray-100 rounded-lg flex items-center justify-center">
                      <span className="text-sm font-bold text-gray-600">
                        {getProviderIcon(signal.providerSlug)}
                      </span>
                    </div>

                    {/* Signal Content */}
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center space-x-2 mb-1">
                        <h3 className="text-sm font-medium text-black truncate">
                          {signal.title}
                        </h3>
                        <span className={`px-2 py-1 text-xs rounded-full font-medium ${getRelevanceColor(signal.relevanceScore)}`}>
                          {signal.relevanceScore || 0}%
                        </span>
                      </div>
                      
                      <p className="text-sm text-gray-600 mb-2 line-clamp-2">
                        {signal.summary}
                      </p>
                      
                      <div className="flex items-center space-x-4 text-xs text-gray-500">
                        <span className="font-medium">{signal.author}</span>
                        <span>•</span>
                        <span>{formatSignalKind(signal.kind)}</span>
                        <span>•</span>
                        <span>{new Date(signal.occurredAt).toLocaleDateString()}</span>
                        <span>•</span>
                        <span>{getConnectionName(signal.connectionId)}</span>
                      </div>
                    </div>
                  </div>

                  {/* Arrow */}
                  <div className="flex-shrink-0 ml-4">
                    <svg className="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
                    </svg>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}