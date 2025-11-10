'use client';

import { useMemo, useState, useEffect } from 'react';
import { DemoConnection, DemoSyncJob, DemoWebhook, DemoToken, DemoRateLimit } from '@/lib/demo/types';
import {
  CheckCircle2,
  AlertTriangle,
  XCircle,
  Clock,
  TrendingUp,
  Activity,
  Zap
} from 'lucide-react';

/**
 * ConnectionStatusCard component.
 * Displays a compact status overview for a single connection.
 */
interface ConnectionStatusCardProps {
  connection: DemoConnection;
  syncJobs?: DemoSyncJob[];
  webhooks?: DemoWebhook[];
  tokens?: DemoToken[];
  rateLimits?: DemoRateLimit[];
  onClick?: () => void;
}

export function ConnectionStatusCard({
  connection,
  syncJobs = [],
  webhooks = [],
  tokens = [],
  rateLimits = [],
  onClick,
}: ConnectionStatusCardProps) {
  const [currentTime, setCurrentTime] = useState(() => Date.now());

  // Update current time every minute to refresh "recent" calculations
  useEffect(() => {
    const interval = setInterval(() => {
      setCurrentTime(Date.now());
    }, 60000); // Update every minute

    return () => clearInterval(interval);
  }, []);

  const getStatusIcon = () => {
    switch (connection.status) {
      case 'connected':
        return <CheckCircle2 className="w-5 h-5 text-green-500" />;
      case 'disconnected':
        return <XCircle className="w-5 h-5 text-red-500" />;
      case 'error':
        return <AlertTriangle className="w-5 h-5 text-orange-500" />;
      default:
        return <Clock className="w-5 h-5 text-gray-500" />;
    }
  };

  const getStatusColor = () => {
    switch (connection.status) {
      case 'connected':
        return 'border-green-200 bg-green-50';
      case 'disconnected':
        return 'border-red-200 bg-red-50';
      case 'error':
        return 'border-orange-200 bg-orange-50';
      default:
        return 'border-gray-200 bg-gray-50';
    }
  };

  const getProviderIcon = (providerSlug: string) => {
    const icons: Record<string, string> = {
      github: '🐙',
      slack: '💬',
      'google-workspace': '📊',
      jira: '🎯',
      'zoho-cliq': '💭',
    };
    return icons[providerSlug] || '🔗';
  };

  // Calculate metrics - memoize to avoid purity issues
  const metrics = useMemo(() => {
    const now = currentTime; // Use state time instead of Date.now()
    const runningJobs = syncJobs.filter(job => job.status === 'running').length;
    const failedJobs = syncJobs.filter(job => job.status === 'failed').length;
    const recentWebhooks = webhooks.filter(webhook =>
      new Date(webhook.createdAt).getTime() > now - 60 * 60 * 1000
    ).length;
    const activeTokens = tokens.filter(token => token.status === 'active').length;
    const exhaustedLimits = rateLimits.filter(limit => limit.remaining === 0).length;

    return {
      runningJobs,
      failedJobs,
      recentWebhooks,
      activeTokens,
      exhaustedLimits,
    };
  }, [syncJobs, webhooks, tokens, rateLimits, currentTime]);

  const hasIssues = connection.status !== 'connected' || metrics.failedJobs > 0 || metrics.exhaustedLimits > 0;

  return (
    <div
      onClick={onClick}
      className={`border rounded-lg p-4 cursor-pointer transition-all hover:shadow-md ${
        hasIssues ? getStatusColor() : 'border-gray-200 bg-white'
      }`}
    >
      <div className="flex items-start justify-between mb-3">
        <div className="flex items-center space-x-3">
          <div className="text-2xl">
            {getProviderIcon(connection.providerSlug)}
          </div>
          <div>
            <h3 className="font-medium text-gray-900">{connection.displayName}</h3>
            <p className="text-sm text-gray-600">{connection.providerSlug}</p>
          </div>
        </div>

        <div className="flex items-center space-x-2">
          {getStatusIcon()}
          {metrics.runningJobs > 0 && (
            <div className="w-2 h-2 bg-blue-500 rounded-full animate-pulse" />
          )}
        </div>
      </div>

      {/* Status */}
      <div className="mb-3">
        <div className={`inline-flex items-center px-2 py-1 rounded-full text-xs font-medium ${
          connection.status === 'connected'
            ? 'bg-green-100 text-green-800'
            : connection.status === 'error'
            ? 'bg-red-100 text-red-800'
            : 'bg-gray-100 text-gray-800'
        }`}>
          {connection.status}
        </div>
      </div>

      {/* Metrics Grid */}
      <div className="grid grid-cols-2 gap-3 text-sm">
        {/* Sync Jobs */}
        <div className="flex items-center space-x-2">
          <Activity className="w-4 h-4 text-gray-500" />
          <div>
            <div className="font-medium text-gray-900">
              {syncJobs.length > 0 ? `${metrics.runningJobs}/${syncJobs.length}` : '0'}
            </div>
            <div className="text-gray-500">Jobs</div>
          </div>
        </div>

        {/* Webhooks */}
        <div className="flex items-center space-x-2">
          <Zap className="w-4 h-4 text-gray-500" />
          <div>
            <div className="font-medium text-gray-900">{metrics.recentWebhooks}</div>
            <div className="text-gray-500">Webhooks</div>
          </div>
        </div>

        {/* Tokens */}
        <div className="flex items-center space-x-2">
          <CheckCircle2 className="w-4 h-4 text-gray-500" />
          <div>
            <div className="font-medium text-gray-900">{metrics.activeTokens}</div>
            <div className="text-gray-500">Tokens</div>
          </div>
        </div>

        {/* Rate Limits */}
        <div className="flex items-center space-x-2">
          <TrendingUp className="w-4 h-4 text-gray-500" />
          <div>
            <div className={`font-medium ${metrics.exhaustedLimits > 0 ? 'text-red-600' : 'text-gray-900'}`}>
              {rateLimits.length > 0 ? `${rateLimits.length - metrics.exhaustedLimits}/${rateLimits.length}` : '0'}
            </div>
            <div className="text-gray-500">Limits</div>
          </div>
        </div>
      </div>

      {/* Issues */}
      {hasIssues && (
        <div className="mt-3 pt-3 border-t border-gray-200">
          <div className="flex items-center space-x-2 text-xs text-orange-600">
            <AlertTriangle className="w-3 h-3" />
            <span>
              {connection.status !== 'connected' && 'Connection issue'}
              {connection.status !== 'connected' && metrics.failedJobs > 0 && ' • '}
              {metrics.failedJobs > 0 && `${metrics.failedJobs} failed job${metrics.failedJobs > 1 ? 's' : ''}`}
              {metrics.failedJobs > 0 && metrics.exhaustedLimits > 0 && ' • '}
              {metrics.exhaustedLimits > 0 && `${metrics.exhaustedLimits} rate limit${metrics.exhaustedLimits > 1 ? 's' : ''} exceeded`}
            </span>
          </div>
        </div>
      )}

      {/* Last Sync */}
      {connection.lastSyncAt && (
        <div className="mt-3 pt-3 border-t border-gray-200 text-xs text-gray-500">
          Last sync: {new Date(connection.lastSyncAt).toLocaleString()}
        </div>
      )}
    </div>
  );
}

export default ConnectionStatusCard;