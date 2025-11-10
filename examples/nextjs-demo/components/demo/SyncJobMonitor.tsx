"use client";

import { useState, useEffect } from "react";
import { DemoSyncJob, DemoConnection } from "@/lib/demo/types";
import {
  CheckCircle2,
  Clock,
  AlertCircle,
  XCircle,
  RefreshCw,
  Play,
  Pause,
  AlertTriangle,
  TrendingUp,
  Activity,
} from "lucide-react";

/**
 * SyncJobMonitor component.
 * Displays active and recent sync jobs with real-time status and progress.
 */
interface SyncJobMonitorProps {
  syncJobs: DemoSyncJob[];
  connections: DemoConnection[];
  onRetryJob?: (jobId: string) => void;
  onPauseJob?: (jobId: string) => void;
  onResumeJob?: (jobId: string) => void;
  className?: string;
}

export function SyncJobMonitor({
  syncJobs,
  connections,
  onRetryJob,
  onPauseJob,
  onResumeJob,
  className = "",
}: SyncJobMonitorProps) {
  const [selectedStatus, setSelectedStatus] = useState<string>("all");
  const [realtimeJobs, setRealtimeJobs] = useState(syncJobs);
  const [now, setNow] = useState(() => Date.now());

  // Simulate real-time updates
  useEffect(() => {
    const interval = setInterval(() => {
      setRealtimeJobs((prevJobs) =>
        prevJobs.map((job) => {
          if (job.status === "running" && Math.random() < 0.1) {
            // 10% chance of running job completing
            return {
              ...job,
              status: "completed" as const,
              lastRunAt: new Date().toISOString(),
            };
          }
          return job;
        }),
      );
      // Advance "now" for time-based UI (e.g., recent job indicator)
      setNow(Date.now());
    }, 3000);

    return () => clearInterval(interval);
  }, []);

  const getConnectionName = (connectionId: string) => {
    const connection = connections.find((c) => c.id === connectionId);
    return connection?.displayName || "Unknown Connection";
  };

  const getProviderIcon = (providerSlug: string) => {
    const icons: Record<string, string> = {
      github: "🐙",
      slack: "💬",
      "google-workspace": "📊",
      jira: "🎯",
      "zoho-cliq": "💭",
    };
    return icons[providerSlug] || "🔗";
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case "completed":
        return <CheckCircle2 className="w-4 h-4 text-green-500" />;
      case "running":
        return <RefreshCw className="w-4 h-4 text-blue-500 animate-spin" />;
      case "pending":
        return <Clock className="w-4 h-4 text-yellow-500" />;
      case "failed":
        return <XCircle className="w-4 h-4 text-red-500" />;
      case "retrying":
        return <RefreshCw className="w-4 h-4 text-orange-500 animate-spin" />;
      default:
        return <AlertCircle className="w-4 h-4 text-gray-500" />;
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case "completed":
        return "bg-green-50 border-green-200 text-green-800";
      case "running":
        return "bg-blue-50 border-blue-200 text-blue-800";
      case "pending":
        return "bg-yellow-50 border-yellow-200 text-yellow-800";
      case "failed":
        return "bg-red-50 border-red-200 text-red-800";
      case "retrying":
        return "bg-orange-50 border-orange-200 text-orange-800";
      default:
        return "bg-gray-50 border-gray-200 text-gray-800";
    }
  };

  const getKindIcon = (kind: string) => {
    switch (kind) {
      case "full":
        return <Activity className="w-3 h-3" />;
      case "incremental":
        return <TrendingUp className="w-3 h-3" />;
      case "webhook_triggered":
        return <AlertTriangle className="w-3 h-3" />;
      default:
        return <Clock className="w-3 h-3" />;
    }
  };

  const formatKind = (kind: string) => {
    return kind
      .split("_")
      .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
      .join(" ");
  };

  const formatDuration = (startTime?: string, endTime?: string) => {
    if (!startTime) return "N/A";

    const start = new Date(startTime);
    const end = endTime ? new Date(endTime) : new Date();
    const duration = end.getTime() - start.getTime();

    if (duration < 1000) return "< 1s";
    if (duration < 60000) return `${Math.floor(duration / 1000)}s`;
    return `${Math.floor(duration / 60000)}m ${Math.floor((duration % 60000) / 1000)}s`;
  };

  // Filter jobs by status
  const filteredJobs =
    selectedStatus === "all"
      ? realtimeJobs
      : realtimeJobs.filter((job) => job.status === selectedStatus);

  // Count jobs by status
  const statusCounts = {
    all: realtimeJobs.length,
    running: realtimeJobs.filter((job) => job.status === "running").length,
    completed: realtimeJobs.filter((job) => job.status === "completed").length,
    failed: realtimeJobs.filter((job) => job.status === "failed").length,
    pending: realtimeJobs.filter((job) => job.status === "pending").length,
    retrying: realtimeJobs.filter((job) => job.status === "retrying").length,
  };

  const statusOptions = [
    { value: "all", label: "All Jobs", count: statusCounts.all },
    { value: "running", label: "Running", count: statusCounts.running },
    { value: "completed", label: "Completed", count: statusCounts.completed },
    { value: "failed", label: "Failed", count: statusCounts.failed },
    { value: "pending", label: "Pending", count: statusCounts.pending },
    { value: "retrying", label: "Retrying", count: statusCounts.retrying },
  ];

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Header */}
      <div className="bg-white border border-gray-200 rounded-lg p-6">
        <div className="flex items-center justify-between mb-6">
          <div className="flex items-center space-x-3">
            <h2 className="text-xl font-semibold text-black">
              Sync Job Monitor
            </h2>
            <div className="flex items-center space-x-2 text-sm text-gray-600">
              <Activity className="w-4 h-4" />
              <span>{statusCounts.running} running</span>
            </div>
          </div>

          <div className="flex items-center space-x-2">
            <div className="text-sm text-gray-600">
              Total: {statusCounts.all} jobs
            </div>
          </div>
        </div>

        {/* Status Filters */}
        <div className="flex flex-wrap gap-2">
          {statusOptions.map((option) => (
            <button
              key={option.value}
              onClick={() => setSelectedStatus(option.value)}
              className={`px-3 py-1 rounded-full text-sm font-medium transition-colors ${
                selectedStatus === option.value
                  ? "bg-black text-white"
                  : "bg-gray-100 text-gray-700 hover:bg-gray-200"
              }`}
            >
              {option.label} ({option.count})
            </button>
          ))}
        </div>
      </div>

      {/* Jobs List */}
      {filteredJobs.length === 0 ? (
        <div className="bg-white border border-gray-200 rounded-lg p-8 text-center">
          <div className="w-16 h-16 bg-gray-100 rounded-full flex items-center justify-center mx-auto mb-4">
            <Clock className="w-8 h-8 text-gray-600" />
          </div>
          <h3 className="text-lg font-medium text-black mb-2">No Sync Jobs</h3>
          <p className="text-gray-600">
            {selectedStatus === "all"
              ? "No sync jobs found. Jobs will appear when connections are synchronized."
              : `No ${selectedStatus} sync jobs found.`}
          </p>
        </div>
      ) : (
        <div className="space-y-4">
          {filteredJobs.map((job) => {
            const isRecent =
              new Date(job.createdAt).getTime() > now - 5 * 60 * 1000; // Last 5 minutes

            return (
              <div
                key={job.id}
                className={`bg-white border rounded-lg p-6 transition-all ${
                  isRecent ? "border-blue-200 shadow-sm" : "border-gray-200"
                }`}
              >
                <div className="flex items-start justify-between">
                  <div className="flex items-start space-x-4 flex-1">
                    {/* Status Icon */}
                    <div
                      className={`p-2 rounded-full ${
                        job.status === "running"
                          ? "bg-blue-100"
                          : job.status === "completed"
                            ? "bg-green-100"
                            : job.status === "failed"
                              ? "bg-red-100"
                              : job.status === "retrying"
                                ? "bg-orange-100"
                                : "bg-gray-100"
                      }`}
                    >
                      {getStatusIcon(job.status)}
                    </div>

                    {/* Job Content */}
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center space-x-3 mb-2">
                        <div className="flex items-center space-x-2">
                          <span className="text-lg">
                            {getProviderIcon(job.providerSlug)}
                          </span>
                          <h3 className="font-medium text-black">
                            {getConnectionName(job.connectionId)}
                          </h3>
                        </div>

                        <div
                          className={`inline-flex items-center px-2 py-1 rounded-full text-xs font-medium border ${getStatusColor(job.status)}`}
                        >
                          {job.status}
                        </div>

                        <div className="flex items-center space-x-1 text-gray-500">
                          {getKindIcon(job.kind)}
                          <span className="text-xs">
                            {formatKind(job.kind)}
                          </span>
                        </div>

                        {isRecent && (
                          <div className="w-2 h-2 bg-blue-500 rounded-full animate-pulse" />
                        )}
                      </div>

                      {/* Progress Bar for Running Jobs */}
                      {job.status === "running" && (
                        <div className="mb-3">
                          <div className="flex items-center justify-between text-xs text-gray-600 mb-1">
                            <span>Processing</span>
                            <span>{formatDuration(job.createdAt)}</span>
                          </div>
                          <div className="w-full bg-gray-200 rounded-full h-2">
                            <div
                              className="bg-blue-500 h-2 rounded-full transition-all duration-500 ease-out"
                              style={{ width: "65%" }} // Simulated progress
                            />
                          </div>
                        </div>
                      )}

                      {/* Error Message for Failed Jobs */}
                      {job.status === "failed" && job.errorMessage && (
                        <div className="mb-3 p-3 bg-red-50 border border-red-200 rounded-md">
                          <div className="flex items-start space-x-2">
                            <XCircle className="w-4 h-4 text-red-500 mt-0.5 flex-shrink-0" />
                            <div className="text-sm text-red-800">
                              <span className="font-medium">Error:</span>{" "}
                              {job.errorMessage}
                            </div>
                          </div>
                          {job.errorCount > 1 && (
                            <div className="text-xs text-red-600 mt-1">
                              Failed after {job.errorCount} retry attempts
                            </div>
                          )}
                        </div>
                      )}

                      {/* Job Metadata */}
                      <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
                        <div>
                          <span className="text-gray-500">Created:</span>
                          <div className="font-medium text-gray-900">
                            {new Date(job.createdAt).toLocaleString()}
                          </div>
                        </div>

                        {job.lastRunAt && (
                          <div>
                            <span className="text-gray-500">Last Run:</span>
                            <div className="font-medium text-gray-900">
                              {new Date(job.lastRunAt).toLocaleString()}
                            </div>
                          </div>
                        )}

                        {job.nextRunAt && (
                          <div>
                            <span className="text-gray-500">Next Run:</span>
                            <div className="font-medium text-gray-900">
                              {new Date(job.nextRunAt).toLocaleString()}
                            </div>
                          </div>
                        )}

                        <div>
                          <span className="text-gray-500">Duration:</span>
                          <div className="font-medium text-gray-900">
                            {formatDuration(job.createdAt, job.lastRunAt)}
                          </div>
                        </div>
                      </div>

                      {/* Cursor for Incremental Jobs */}
                      {job.cursor && (
                        <div className="mt-3 text-xs text-gray-500">
                          <span className="font-medium">Cursor:</span>{" "}
                          {job.cursor}
                        </div>
                      )}
                    </div>
                  </div>

                  {/* Actions */}
                  <div className="flex items-center space-x-2 ml-4">
                    {job.status === "failed" && onRetryJob && (
                      <button
                        onClick={() => onRetryJob(job.id)}
                        className="p-2 text-blue-600 hover:bg-blue-50 rounded-md transition-colors"
                        title="Retry Job"
                      >
                        <RefreshCw className="w-4 h-4" />
                      </button>
                    )}

                    {job.status === "running" && onPauseJob && (
                      <button
                        onClick={() => onPauseJob(job.id)}
                        className="p-2 text-yellow-600 hover:bg-yellow-50 rounded-md transition-colors"
                        title="Pause Job"
                      >
                        <Pause className="w-4 h-4" />
                      </button>
                    )}

                    {(job.status === "pending" || job.status === "failed") &&
                      onResumeJob && (
                        <button
                          onClick={() => onResumeJob(job.id)}
                          className="p-2 text-green-600 hover:bg-green-50 rounded-md transition-colors"
                          title="Resume Job"
                        >
                          <Play className="w-4 h-4" />
                        </button>
                      )}
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}

export default SyncJobMonitor;
