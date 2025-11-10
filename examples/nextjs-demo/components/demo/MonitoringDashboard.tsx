"use client";

import { useState, useEffect } from "react";
import {
  DemoConnection,
  DemoSyncJob,
  DemoWebhook,
  DemoToken,
  DemoRateLimit,
} from "@/lib/demo/types";
import {
  Activity,
  Zap,
  TrendingUp,
  AlertTriangle,
  RefreshCw,
  BarChart3,
  Settings,
} from "lucide-react";
import ConnectionStatusCard from "./ConnectionStatusCard";
import DemoConfiguration from "./DemoConfiguration";
import {
  generateMockSyncJobs,
  generateMockWebhooks,
  generateMockTokens,
  generateMockRateLimits,
  generateFailedSyncJobs,
  generateFailedWebhooks,
  generateRateLimitedConnections,
  generateExpiredTokens,
} from "@/lib/demo/mockData";

/**
 * MonitoringDashboard component.
 * Comprehensive dashboard showing status of all connections and their entities.
 */
interface MonitoringDashboardProps {
  connections: DemoConnection[];
  onNavigateToSyncJobs?: () => void;
  onNavigateToIntegrations?: () => void;
  className?: string;
}

export function MonitoringDashboard({
  connections,
  onNavigateToSyncJobs,
  onNavigateToIntegrations,
  className = "",
}: MonitoringDashboardProps) {
  const [syncJobs, setSyncJobs] = useState<DemoSyncJob[]>([]);
  const [webhooks, setWebhooks] = useState<DemoWebhook[]>([]);
  const [tokens, setTokens] = useState<DemoToken[]>([]);
  const [rateLimits, setRateLimits] = useState<DemoRateLimit[]>([]);
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [now, setNow] = useState(() => Date.now());

  const generateAllMockData = () => {
    const allSyncJobs = connections.flatMap((connection) => [
      ...generateMockSyncJobs(connection),
      ...generateFailedSyncJobs(connection),
    ]);

    const allWebhooks = connections.flatMap((connection) => [
      ...generateMockWebhooks(connection),
      ...generateFailedWebhooks(connection),
    ]);

    const allTokens = connections.flatMap((connection) => [
      ...generateMockTokens(connection),
      ...generateExpiredTokens(connection),
    ]);

    const allRateLimits = connections.flatMap((connection) => [
      ...generateMockRateLimits(connection),
      ...generateRateLimitedConnections(connection),
    ]);

    setSyncJobs(allSyncJobs);
    setWebhooks(allWebhooks);
    setTokens(allTokens);
    setRateLimits(allRateLimits);
    setNow(Date.now());
  };

  const handleRefresh = async () => {
    setIsRefreshing(true);
    await new Promise((resolve) => setTimeout(resolve, 1000));
    generateAllMockData();
    setIsRefreshing(false);
  };

  useEffect(() => {
    if (connections.length > 0) {
      generateAllMockData();
    } else {
      setSyncJobs([]);
      setWebhooks([]);
      setTokens([]);
      setRateLimits([]);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [connections]);

  useEffect(() => {
    const interval = setInterval(() => {
      setNow(Date.now());
    }, 60000);

    return () => clearInterval(interval);
  }, []);

  const stats = {
    connections: {
      total: connections.length,
      connected: connections.filter((c) => c.status === "connected").length,
      issues: connections.filter((c) => c.status !== "connected").length,
    },
    syncJobs: {
      total: syncJobs.length,
      running: syncJobs.filter((j) => j.status === "running").length,
      failed: syncJobs.filter((j) => j.status === "failed").length,
      completed: syncJobs.filter((j) => j.status === "completed").length,
    },
    webhooks: {
      total: webhooks.length,
      processed: webhooks.filter((w) => w.processedAt).length,
      failed: webhooks.filter((w) => !w.verified).length,
      recent: webhooks.filter(
        (w) => new Date(w.createdAt).getTime() > now - 60 * 60 * 1000,
      ).length,
    },
    tokens: {
      total: tokens.length,
      active: tokens.filter((t) => t.status === "active").length,
      expired: tokens.filter((t) => t.status === "expired").length,
      refreshing: tokens.filter((t) => t.status === "refreshing").length,
    },
    rateLimits: {
      total: rateLimits.length,
      healthy: rateLimits.filter((r) => r.remaining > r.currentLimit * 0.1)
        .length,
      warning: rateLimits.filter(
        (r) => r.remaining <= r.currentLimit * 0.1 && r.remaining > 0,
      ).length,
      exhausted: rateLimits.filter((r) => r.remaining === 0).length,
    },
  };

  const rateLimitsColor =
    stats.rateLimits.exhausted > 0 ? "bg-red-500" : "bg-yellow-500";

  return (
    <div className={`space-y-6 ${className}`}>
      <div className="bg-white border border-gray-200 rounded-lg p-6">
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-xl font-semibold text-gray-900">
              Monitoring Dashboard
            </h2>
            <p className="text-sm text-gray-600 mt-1">
              Real-time status of all connections and their operations
            </p>
          </div>

          <div className="flex items-center space-x-3">
            <button
              onClick={handleRefresh}
              disabled={isRefreshing}
              className="flex items-center space-x-2 px-4 py-2 bg-white border border-gray-300 rounded-md hover:bg-gray-50 transition-colors disabled:opacity-50"
            >
              <RefreshCw
                className={`w-4 h-4 ${isRefreshing ? "animate-spin" : ""}`}
              />
              <span>Refresh</span>
            </button>

            <button
              onClick={onNavigateToSyncJobs}
              className="flex items-center space-x-2 px-4 py-2 bg-black hover:bg-gray-800 text-white rounded-md transition-colors"
            >
              <Activity className="w-4 h-4" />
              <span>Sync Jobs</span>
            </button>

            <button
              onClick={onNavigateToIntegrations}
              className="flex items-center space-x-2 px-4 py-2 bg-gray-100 hover:bg-gray-200 text-gray-700 rounded-md transition-colors"
            >
              <Settings className="w-4 h-4" />
              <span>Settings</span>
            </button>
          </div>
        </div>
      </div>
      <DemoConfiguration className="mt-4" />

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        <DashboardStatCard
          title="Connections"
          value={`${stats.connections.connected}/${stats.connections.total}`}
          subtitle={`${stats.connections.issues} need attention`}
          icon={BarChart3}
          color="bg-blue-500"
          trend={stats.connections.issues === 0 ? 5 : -2}
        />

        <DashboardStatCard
          title="Sync Jobs"
          value={`${stats.syncJobs.running} running`}
          subtitle={`${stats.syncJobs.failed} failed, ${stats.syncJobs.completed} completed`}
          icon={Activity}
          color="bg-green-500"
          trend={stats.syncJobs.running > 0 ? 12 : -5}
        />

        <DashboardStatCard
          title="Webhooks"
          value={`${stats.webhooks.recent} recent`}
          subtitle={`${stats.webhooks.processed} processed, ${stats.webhooks.failed} failed`}
          icon={Zap}
          color="bg-purple-500"
          trend={stats.webhooks.recent > 0 ? 8 : 1}
        />

        <DashboardStatCard
          title="Rate Limits"
          value={`${stats.rateLimits.healthy}/${stats.rateLimits.total}`}
          subtitle={`${stats.rateLimits.exhausted} exhausted, ${stats.rateLimits.warning} warning`}
          icon={TrendingUp}
          color={rateLimitsColor}
          trend={stats.rateLimits.exhausted > 0 ? -15 : 3}
        />
      </div>

      <div className="bg-white border border-gray-200 rounded-lg p-6">
        <div className="flex items-center justify-between mb-6">
          <h3 className="text-lg font-semibold text-gray-900">
            Connection Status
          </h3>
          <div className="flex items-center space-x-4 text-sm">
            <div className="flex items-center space-x-2">
              <div className="w-3 h-3 bg-green-500 rounded-full" />
              <span className="text-gray-600">Connected</span>
            </div>
            <div className="flex items-center space-x-2">
              <div className="w-3 h-3 bg-yellow-500 rounded-full" />
              <span className="text-gray-600">Warning</span>
            </div>
            <div className="flex items-center space-x-2">
              <div className="w-3 h-3 bg-red-500 rounded-full" />
              <span className="text-gray-600">Error</span>
            </div>
          </div>
        </div>

        {connections.length === 0 ? (
          <div className="text-center py-8">
            <div className="w-16 h-16 bg-gray-100 rounded-full flex items-center justify-center mx-auto mb-4">
              <BarChart3 className="w-8 h-8 text-gray-600" />
            </div>
            <h4 className="text-lg font-medium text-gray-900 mb-2">
              No Connections
            </h4>
            <p className="text-gray-600 mb-4">
              Set up connections to start monitoring their status
            </p>
            <button
              onClick={onNavigateToIntegrations}
              className="px-4 py-2 bg-black hover:bg-gray-800 text-white rounded-md text-sm font-medium transition-colors"
            >
              Set Up Connections
            </button>
          </div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {connections.map((connection) => {
              const connectionSyncJobs = syncJobs.filter(
                (job) => job.connectionId === connection.id,
              );
              const connectionWebhooks = webhooks.filter(
                (webhook) => webhook.connectionId === connection.id,
              );
              const connectionTokens = tokens.filter(
                (token) => token.connectionId === connection.id,
              );
              const connectionRateLimits = rateLimits.filter(
                (limit) => limit.connectionId === connection.id,
              );

              return (
                <ConnectionStatusCard
                  key={connection.id}
                  connection={connection}
                  syncJobs={connectionSyncJobs}
                  webhooks={connectionWebhooks}
                  tokens={connectionTokens}
                  rateLimits={connectionRateLimits}
                  onClick={() => onNavigateToSyncJobs?.()}
                />
              );
            })}
          </div>
        )}
      </div>

      {(stats.connections.issues > 0 ||
        stats.syncJobs.failed > 0 ||
        stats.rateLimits.exhausted > 0) && (
        <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-6">
          <div className="flex items-start space-x-3">
            <AlertTriangle className="w-5 h-5 text-yellow-600 mt-0.5" />
            <div className="flex-1">
              <h4 className="text-lg font-medium text-yellow-800 mb-2">
                Action Required
              </h4>
              <ul className="text-sm text-yellow-700 space-y-1">
                {stats.connections.issues > 0 && (
                  <li>
                    • {stats.connections.issues} connection
                    {stats.connections.issues > 1 ? "s" : ""} need attention
                  </li>
                )}
                {stats.syncJobs.failed > 0 && (
                  <li>
                    • {stats.syncJobs.failed} failed sync job
                    {stats.syncJobs.failed > 1 ? "s" : ""} require retry
                  </li>
                )}
                {stats.rateLimits.exhausted > 0 && (
                  <li>
                    • {stats.rateLimits.exhausted} rate limit
                    {stats.rateLimits.exhausted > 1 ? "s" : ""} exhausted
                  </li>
                )}
              </ul>
              <div className="mt-4 flex items-center space-x-3">
                <button
                  onClick={onNavigateToSyncJobs}
                  className="px-4 py-2 bg-yellow-600 hover:bg-yellow-700 text-white rounded-md text-sm font-medium transition-colors"
                >
                  Review Issues
                </button>
                <button
                  onClick={handleRefresh}
                  className="px-4 py-2 bg-white hover:bg-gray-50 text-yellow-700 border border-yellow-300 rounded-md text-sm font-medium transition-colors"
                >
                  Refresh Status
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default MonitoringDashboard;

function DashboardStatCard({
  title,
  value,
  subtitle,
  icon: Icon,
  color,
  trend,
}: {
  title: string;
  value: string;
  subtitle: string;
  icon: React.ComponentType<{ className?: string }>;
  color: string;
  trend?: number;
}) {
  return (
    <div className="bg-white border border-gray-200 rounded-lg p-6">
      <div className="flex items-center justify-between">
        <div>
          <p className="text-sm font-medium text-gray-600">{title}</p>
          <p className="text-2xl font-bold text-gray-900">{value}</p>
          <p className="text-sm text-gray-500">{subtitle}</p>
        </div>
        <div className={`p-3 rounded-lg ${color}`}>
          <Icon className="w-6 h-6 text-white" />
        </div>
      </div>
      {trend !== undefined && (
        <div className="mt-4 flex items-center text-sm">
          {trend > 0 ? (
            <TrendingUp className="w-4 h-4 text-green-500 mr-1" />
          ) : (
            <TrendingUp className="w-4 h-4 text-red-500 mr-1 transform rotate-180" />
          )}
          <span className={trend > 0 ? "text-green-600" : "text-red-600"}>
            {Math.abs(trend)}%
          </span>
          <span className="text-gray-500 ml-1">from last hour</span>
        </div>
      )}
    </div>
  );
}
