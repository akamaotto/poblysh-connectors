'use client';

import MonitoringDashboard from '@/components/demo/MonitoringDashboard';
import { useDemoState } from '@/lib/demo/state';

export default function MonitoringPage() {
  const { state } = useDemoState();

  const handleNavigateToSyncJobs = () => {
    window.location.href = '/sync-jobs';
  };

  const handleNavigateToIntegrations = () => {
    window.location.href = '/integrations';
  };

  return (
    <div className="min-h-screen bg-gray-50">
      {/* Header */}
      <div className="bg-white border-b border-gray-200">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between items-center py-6">
            <div>
              <h1 className="text-2xl font-bold text-gray-900">Monitoring</h1>
              <p className="mt-1 text-sm text-gray-600">
                Real-time monitoring of all connections and sync operations
              </p>
            </div>

            <div className="flex items-center space-x-4">
              <div className="text-sm text-gray-600">
                {state.connections.length} connections
              </div>
              <button
                onClick={handleNavigateToIntegrations}
                className="px-4 py-2 bg-black hover:bg-gray-800 text-white rounded-md text-sm font-medium transition-colors"
              >
                Manage Connections
              </button>
            </div>
          </div>
        </div>
      </div>

      {/* Main Content */}
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <MonitoringDashboard
          connections={state.connections}
          onNavigateToSyncJobs={handleNavigateToSyncJobs}
          onNavigateToIntegrations={handleNavigateToIntegrations}
        />
      </div>
    </div>
  );
}