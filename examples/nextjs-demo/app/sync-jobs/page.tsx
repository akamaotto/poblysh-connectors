"use client";

import { useState, useEffect } from "react";
import SyncJobMonitor from "@/components/demo/SyncJobMonitor";
import { useDemoState } from "@/lib/demo/state";
import { generateMockSyncJobs } from "@/lib/demo/mockData";
import { DemoSyncJob, DemoConnection } from "@/lib/demo/types";

export default function SyncJobsPage() {
  const { state } = useDemoState();
  const [syncJobs, setSyncJobs] = useState<DemoSyncJob[]>([]);

  // Helper function to generate failed sync jobs for demonstration
  const generateFailedSyncJobs = (
    connection: DemoConnection,
  ): DemoSyncJob[] => {
    const failedJobs: DemoSyncJob[] = [
      {
        id: `failed-${connection.id}-${Date.now()}`,
        tenantId: connection.tenantId,
        connectionId: connection.id,
        providerSlug: connection.providerSlug,
        status: "failed",
        kind: "full",
        errorCount: 3,
        lastRunAt: new Date(Date.now() - 30 * 60 * 1000).toISOString(),
        createdAt: new Date(Date.now() - 60 * 60 * 1000).toISOString(),
        errorMessage:
          "API rate limit exceeded - 5000 requests per hour limit reached",
      },
      {
        id: `failed-auth-${connection.id}-${Date.now()}`,
        tenantId: connection.tenantId,
        connectionId: connection.id,
        providerSlug: connection.providerSlug,
        status: "failed",
        kind: "incremental",
        errorCount: 1,
        lastRunAt: new Date(Date.now() - 15 * 60 * 1000).toISOString(),
        createdAt: new Date(Date.now() - 45 * 60 * 1000).toISOString(),
        errorMessage: "Authentication failed - OAuth token expired or revoked",
      },
    ];

    return failedJobs;
  };

  // Generate mock sync jobs when component mounts
  useEffect(() => {
    if (state.connections.length > 0) {
      const jobs = state.connections.flatMap((connection) =>
        generateMockSyncJobs(connection),
      );

      // Add some failed jobs for demonstration
      const failedJobs = state.connections
        .slice(0, 2)
        .flatMap((connection) => generateFailedSyncJobs(connection));

      // Use setTimeout to avoid synchronous setState in effect
      const timer = setTimeout(() => {
        setSyncJobs([...jobs, ...failedJobs]);
      }, 0);

      return () => clearTimeout(timer);
    }
  }, [state.connections]);

  // Mock handlers for job actions
  const handleRetryJob = (jobId: string) => {
    console.log("Retrying job:", jobId);
    // In a real implementation, this would trigger a retry API call
    setSyncJobs((prevJobs) =>
      prevJobs.map((job) =>
        job.id === jobId
          ? {
              ...job,
              status: "running" as const,
              errorMessage: "" as const,
              errorCount: 0,
            }
          : job,
      ),
    );
  };

  const handlePauseJob = (jobId: string) => {
    console.log("Pausing job:", jobId);
    // In a real implementation, this would trigger a pause API call
    setSyncJobs((prevJobs) =>
      prevJobs.map((job) =>
        job.id === jobId ? { ...job, status: "pending" as const } : job,
      ),
    );
  };

  const handleResumeJob = (jobId: string) => {
    console.log("Resuming job:", jobId);
    // In a real implementation, this would trigger a resume API call
    setSyncJobs((prevJobs) =>
      prevJobs.map((job) =>
        job.id === jobId ? { ...job, status: "running" as const } : job,
      ),
    );
  };

  return (
    <div className="min-h-screen bg-gray-50">
      {/* Header */}
      <header className="bg-white border-b border-gray-200">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between items-center py-6">
            <div>
              <h1 className="text-2xl font-bold text-gray-900">Sync Jobs</h1>
              <p className="mt-1 text-sm text-gray-600">
                Monitor and manage sync operations across your connected
                services
              </p>
            </div>

            <nav
              className="flex items-center space-x-4"
              aria-label="Page actions"
            >
              <div className="text-sm text-gray-600" role="status">
                {state.connections.length} connections
              </div>
              <button
                onClick={() => (window.location.href = "/integrations")}
                className="px-4 py-2 bg-black hover:bg-gray-800 text-white rounded-md text-sm font-medium transition-colors focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-black"
                aria-label="Manage connections"
              >
                Manage Connections
              </button>
            </nav>
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8" role="main">
        {state.connections.length === 0 ? (
          <section
            className="bg-white border border-gray-200 rounded-lg p-8 text-center"
            aria-labelledby="no-connections-title"
          >
            <div className="w-16 h-16 bg-gray-100 rounded-full flex items-center justify-center mx-auto mb-4">
              <svg
                className="w-8 h-8 text-gray-600"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
                aria-hidden="true"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M13 10V3L4 14h7v7l9-11h-7z"
                />
              </svg>
            </div>
            <h2
              id="no-connections-title"
              className="text-lg font-medium text-gray-900 mb-2"
            >
              No Connections Found
            </h2>
            <p className="text-gray-600 mb-4">
              Connect services to start monitoring sync jobs
            </p>
            <button
              onClick={() => (window.location.href = "/integrations")}
              className="px-4 py-2 bg-black hover:bg-gray-800 text-white rounded-md text-sm font-medium transition-colors focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-black"
              aria-label="Set up connections to start monitoring"
            >
              Set Up Connections
            </button>
          </section>
        ) : (
          <section aria-label="Sync job monitor">
            <SyncJobMonitor
              syncJobs={syncJobs}
              connections={state.connections}
              onRetryJob={handleRetryJob}
              onPauseJob={handlePauseJob}
              onResumeJob={handleResumeJob}
            />
          </section>
        )}
      </main>
    </div>
  );
}
