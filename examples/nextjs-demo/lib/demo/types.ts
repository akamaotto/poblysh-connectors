/**
 * Mock domain types for the Poblysh Connectors demo.
 * 
 * These types mirror real Connectors API concepts but are simplified
 * for demonstration purposes. All data is generated locally with
 * deterministic patterns.
 */

/**
 * Enhanced user representation with additional authentication properties.
 * In production, this would come from real authentication.
 */
export interface DemoUser {
  /** Unique user identifier */
  id: string;
  /** User email address */
  email: string;
  /** User display name (generated from email) */
  name: string;
  /** Optional user avatar URL */
  avatarUrl?: string;
  /** User roles within the tenant */
  roles: string[];
  /** Tenant ID this user belongs to */
  tenantId: string;
}

/**
 * Enhanced tenant representation with additional authentication context.
 * Shows the mapping between Poblysh Core tenant and Connectors service tenant.
 */
export interface DemoTenant {
  /** Poblysh Core tenant ID */
  id: string;
  /** Tenant display name */
  name: string;
  /** Tenant slug for URLs */
  slug: string;
  /** Connectors service tenant ID (used in X-Tenant-Id header) */
  connectorsTenantId: string;
  /** Tenant creation timestamp */
  createdAt: string;
  /** Tenant plan tier */
  plan: 'free' | 'pro' | 'enterprise';
}

/**
 * Enhanced provider configuration for integrations.
 */
export interface DemoProvider {
  /** Provider slug/identifier */
  slug: string;
  /** Provider display name */
  name: string;
  /** Provider description */
  description: string;
  /** Provider icon URL */
  iconUrl: string;
  /** Supported signal kinds */
  supportedSignalKinds: string[];
  /** Authentication type for this provider */
  authType: 'oauth2' | 'api_key' | 'webhook' | 'hybrid';
  /** Rate limiting configuration */
  rateLimit?: {
    /** Requests per hour limit */
    requestsPerHour: number;
    /** Requests per minute limit */
    requestsPerMinute: number;
  };
  /** Supported webhook events */
  webhookEvents: string[];
  /** Default OAuth scopes */
  defaultScopes: string[];
  /** Provider capabilities */
  features: {
    /** Supports real-time webhooks */
    realtimeWebhooks: boolean;
    /** Supports historical data sync */
    historicalSync: boolean;
    /** Supports incremental sync */
    incrementalSync: boolean;
    /** Supports cross-provider correlation */
    crossProviderCorrelation: boolean;
  };
}

/**
 * Connection state for a provider integration.
 * Represents the OAuth connection between tenant and provider.
 */
export interface DemoConnection {
  /** Connection identifier */
  id: string;
  /** Tenant ID that owns this connection */
  tenantId: string;
  /** Provider being connected */
  providerSlug: string;
  /** Connection display name */
  displayName: string;
  /** Current connection status */
  status: 'disconnected' | 'connected' | 'error';
  /** Connection creation timestamp */
  createdAt: string;
  /** Last successful sync timestamp */
  lastSyncAt?: string;
  /** Connection error message (if any) */
  error?: string;
}

/**
 * Raw signal from a provider.
 * Represents an event or activity discovered from connected services.
 */
export interface DemoSignal {
  /** Signal identifier */
  id: string;
  /** Tenant ID that owns this signal */
  tenantId: string;
  /** Provider that generated this signal */
  providerSlug: string;
  /** Connection ID for this signal */
  connectionId: string;
  /** Signal type/kind */
  kind: string;
  /** Signal title */
  title: string;
  /** Signal summary/description */
  summary: string;
  /** Signal author/actor */
  author: string;
  /** When the signal occurred */
  occurredAt: string;
  /** When signal was discovered */
  discoveredAt: string;
  /** Additional signal metadata */
  metadata: Record<string, unknown>;
  /** Signal URL (if applicable) */
  url?: string;
  /** Signal relevance score (0-100) */
  relevanceScore?: number;

  // Extended metadata (new additions)
  /** Raw payload from provider API */
  rawPayload: Record<string, unknown>;
  /** Signal processing details */
  processingDetails: {
    /** Time to fetch from provider (ms) */
    fetchTime: number;
    /** Time to process (ms) */
    processingTime: number;
    /** Number of retry attempts */
    retryCount: number;
    /** Last retry timestamp */
    lastRetryAt?: string;
  };
  /** IDs of related signals */
  relatedSignals: string[];
  /** ID of parent signal (for grouped/threaded activities) */
  parentSignalId?: string;
  /** IDs of child signals (for activities that spawn other activities) */
  childSignalIds: string[];
  /** Signal classification categories */
  categories: string[];
  /** Signal sentiment analysis */
  sentiment?: 'positive' | 'negative' | 'neutral';
  /** Signal urgency level */
  urgency: 'low' | 'medium' | 'high' | 'critical';
  /** Signal impact context */
  impact: {
    /** Scope of impact */
    scope: 'team' | 'project' | 'organization' | 'public';
    /** Number of affected users */
    affectedUsers?: number;
    /** Estimated cost impact */
    estimatedCost?: number;
  };
  /** Signal environment */
  environment: 'production' | 'staging' | 'development';
}

/**
 * Evidence item supporting signal grounding.
 * Shows how signals from different providers reinforce each other.
 */
export interface DemoEvidenceItem {
  /** Evidence identifier */
  id: string;
  /** Source signal ID */
  sourceSignalId: string;
  /** Source provider slug */
  providerSlug: string;
  /** Evidence type */
  type: 'reference' | 'mention' | 'related_activity' | 'cross_reference';
  /** Evidence description */
  description: string;
  /** Evidence strength score (0-100) */
  strength: number;
  /** Related signal ID (if applicable) */
  relatedSignalId?: string;
}

/**
 * Grounded signal with scoring and evidence.
 * Represents the output of processing raw signals through the grounding engine.
 */
export interface DemoGroundedSignal {
  /** Grounded signal identifier */
  id: string;
  /** Source signal ID that was grounded */
  sourceSignalId: string;
  /** Tenant ID that owns this grounded signal */
  tenantId: string;
  /** Overall grounding score (0-100) */
  score: number;
  /** Dimensional scores */
  dimensions: Array<{
    /** Dimension label */
    label: string;
    /** Dimension score (0-100) */
    score: number;
    /** Dimension description */
    description: string;
  }>;
  /** Evidence items supporting the grounding */
  evidence: DemoEvidenceItem[];
  /** Grounding creation timestamp */
  createdAt: string;
  /** Confidence level */
  confidence: 'low' | 'medium' | 'high';
  /** Summary of grounding results */
  summary: string;
}

/**
 * Mock sync job representation.
 * Represents scheduled or webhook-triggered synchronization tasks.
 */
export interface DemoSyncJob {
  /** Sync job identifier */
  id: string;
  /** Tenant ID that owns this sync job */
  tenantId: string;
  /** Connection ID this sync job is for */
  connectionId: string;
  /** Provider being synced */
  providerSlug: string;
  /** Current sync job status */
  status: 'pending' | 'running' | 'completed' | 'failed' | 'retrying';
  /** Type of sync job */
  kind: 'full' | 'incremental' | 'webhook_triggered';
  /** Cursor position for incremental syncs */
  cursor?: string;
  /** Number of retry attempts */
  errorCount: number;
  /** When the sync job was last run */
  lastRunAt?: string;
  /** When the sync job is scheduled to run next */
  nextRunAt?: string;
  /** When the sync job was created */
  createdAt: string;
  /** Error message (if any) */
  errorMessage?: string;
}

/**
 * Mock webhook event representation.
 * Represents incoming webhook events from providers.
 */
export interface DemoWebhook {
  /** Webhook event identifier */
  id: string;
  /** Tenant ID that owns this webhook */
  tenantId: string;
  /** Connection ID this webhook is for */
  connectionId: string;
  /** Provider that sent this webhook */
  providerSlug: string;
  /** Webhook event type */
  eventType: string;
  /** Webhook payload data */
  payload: Record<string, unknown>;
  /** Webhook signature (if provided) */
  signature?: string;
  /** Whether webhook signature was verified */
  verified: boolean;
  /** When webhook was processed */
  processedAt?: string;
  /** When webhook was received */
  createdAt: string;
}

/**
 * Mock OAuth token representation.
 * Represents authentication tokens for provider connections.
 */
export interface DemoToken {
  /** Token identifier */
  id: string;
  /** Connection ID this token is for */
  connectionId: string;
  /** Type of token */
  tokenType: 'oauth' | 'api_key' | 'service_account';
  /** OAuth scopes granted to this token */
  scopes: string[];
  /** When token expires */
  expiresAt?: string;
  /** When token was last refreshed */
  lastRefreshed?: string;
  /** Current token status */
  status: 'active' | 'expired' | 'revoked' | 'refreshing';
}

/**
 * Mock rate limit representation.
 * Represents API rate limiting state for connections.
 */
export interface DemoRateLimit {
  /** Rate limit identifier */
  id: string;
  /** Connection ID this rate limit is for */
  connectionId: string;
  /** Provider this rate limit applies to */
  providerSlug: string;
  /** API endpoint or resource */
  endpoint: string;
  /** Current rate limit ceiling */
  currentLimit: number;
  /** Remaining requests in current window */
  remaining: number;
  /** When rate limit window resets */
  resetAt: string;
  /** Seconds to wait before next request */
  retryAfter?: number;
}

/**
 * Mock API response wrapper.
 * Mimics the structure of real Connectors API responses.
 */
export interface DemoApiResponse<T> {
  /** Response data */
  data: T;
  /** Response metadata */
  meta: {
    /** Request ID */
    requestId: string;
    /** Response timestamp */
    timestamp: string;
    /** Pagination info (if applicable) */
    pagination?: {
      /** Current page */
      page: number;
      /** Items per page */
      limit: number;
      /** Total items */
      total: number;
      /** Total pages */
      totalPages: number;
    };
  };
}

/**
 * Mock API error response.
 * Mimics the structure of real Connectors API error responses.
 */
export interface DemoApiError {
  /** Error code (screaming snake case as per spec) */
  code: string;
  /** Human-readable error message */
  message: string;
  /** Additional error details */
  details?: Record<string, unknown>;
  /** Request ID */
  requestId: string;
  /** Error timestamp */
  timestamp: string;
}

/**
 * Demo mode configuration.
 * Defines whether the demo runs in mock or real API mode.
 */
export type DemoMode = 'mock' | 'real';

/**
 * Demo configuration interface.
 * Defines configurable parameters for demo behavior.
 */
export interface DemoConfig {
  /** Signal generation frequency */
  signalFrequency: 'low' | 'medium' | 'high';
  /** Error rate for demo scenarios */
  errorRate: '0%' | '10%' | '20%';
  /** Timing mode for demo operations */
  timingMode: 'fast' | 'realistic';
  /** Provider complexity level */
  providerComplexity: 'simple' | 'detailed';
  
  // Runtime mode configuration
  /** Current demo mode (mock vs real API) */
  mode: DemoMode;
  /** Connectors API base URL (for real mode) */
  connectorsApiBaseUrl?: string;
  /** Whether the current configuration is valid */
  isConfigValid: boolean;
  /** Configuration validation errors */
  configErrors: string[];
  /** Configuration validation warnings */
  configWarnings: string[];
}

// ============================================================================
// ENHANCED AUTHENTICATION DOMAIN MODELS
// ============================================================================

/**
 * Mock JWT-style token for educational purposes.
 * Represents a JWT token with visible structure for learning.
 *
 * IMPORTANT: This is NOT a real JWT and has NO cryptographic security.
 * The signature is a mock string for demonstration only.
 */
export interface DemoAuthToken {
  /** Token identifier */
  id: string;
  /** Token type */
  type: 'access' | 'refresh' | 'id';
  /** Provider ID this token is for (null for core auth) */
  providerId?: string;
  /** JWT header (mock) */
  header: {
    alg: string; // Algorithm (e.g., "HS256" - MOCK)
    typ: 'JWT'; // Token type
  };
  /** JWT payload (mock) */
  payload: {
    sub: string; // User ID
    email?: string; // User email
    tenantId: string; // Tenant ID
    sessionId: string; // Session identifier
    scopes: string[]; // Auth scopes
    iat: number; // Issued at timestamp
    exp: number; // Expires at timestamp
    providerId?: string; // Provider ID (for provider tokens)
    tokenUse?: string; // Token use (e.g., "id", "access")
    meta?: Record<string, unknown>; // Additional metadata
  };
  /** Mock signature (NOT cryptographically valid) */
  signature: string;
}

/**
 * Authentication session representing a user's login state.
 * Manages the core session lifecycle for the demo.
 */
export interface DemoAuthSession {
  /** Session identifier */
  id: string;
  /** User ID this session belongs to */
  userId: string;
  /** Tenant ID this session is scoped to */
  tenantId: string;
  /** Current session status */
  status: 'active' | 'expired' | 'revoked';
  /** Session creation timestamp */
  createdAt: number;
  /** Session last updated timestamp */
  updatedAt: number;
  /** Session expiration timestamp */
  expiresAt: number;
  /** Primary access token ID */
  primaryTokenId: string;
  /** Refresh token ID (if available) */
  refreshTokenId?: string;
  /** Associated provider authentication IDs */
  providerAuthIds: string[];
  /** Last activity timestamp */
  lastActivityAt?: number;
  /** Additional session metadata */
  meta?: Record<string, unknown>;
}

/**
 * Provider-specific authentication state.
 * Represents the OAuth connection between tenant and provider.
 */
export interface DemoProviderAuth {
  /** Provider authentication identifier */
  id: string;
  /** Provider being authenticated */
  providerId: string;
  /** Session ID this auth belongs to */
  sessionId: string;
  /** Granted OAuth scopes */
  scopes: string[];
  /** Current authentication status */
  status: 'connected' | 'expired' | 'revoked' | 'error';
  /** Access token ID */
  accessTokenId?: string;
  /** Refresh token ID */
  refreshTokenId?: string;
  /** Authentication creation timestamp */
  createdAt: number;
  /** Authentication last updated timestamp */
  updatedAt: number;
  /** Additional provider metadata */
  meta?: {
    displayName?: string;
    accountId?: string;
    accountName?: string;
    orgName?: string;
    avatarUrl?: string;
  };
}

/**
 * Authentication event for security monitoring and educational insights.
 * Tracks all authentication-related activities in the demo.
 */
export interface DemoAuthEvent {
  /** Event identifier */
  id: string;
  /** Session ID this event relates to */
  sessionId?: string;
  /** User ID this event relates to */
  userId?: string;
  /** Tenant ID this event relates to */
  tenantId?: string;
  /** Provider ID this event relates to */
  providerId?: string;
  /** Event type */
  type:
    | 'LOGIN_SUCCESS'
    | 'LOGIN_FAILURE'
    | 'LOGOUT'
    | 'TOKEN_ISSUED'
    | 'TOKEN_REFRESHED'
    | 'TOKEN_REFRESH_FAILED'
    | 'TOKEN_REVOKED'
    | 'PROVIDER_CONNECTED'
    | 'PROVIDER_DISCONNECTED'
    | 'SECURITY_WARNING'
    | 'SCOPE_CHANGED'
    | 'SESSION_EXPIRED';
  /** Event severity level */
  severity: 'info' | 'warning' | 'error' | 'debug';
  /** Event timestamp */
  timestamp: number;
  /** Additional event details */
  details?: Record<string, unknown>;
}

/**
 * Authentication error scenario configuration.
 * Defines mock error scenarios for educational purposes.
 */
export interface DemoAuthErrorScenario {
  /** Error scenario identifier */
  id: string;
  /** Error scenario name */
  name: string;
  /** Error scenario description */
  description: string;
  /** Error type to simulate */
  errorType: 'network_timeout' | 'token_refresh_failed' | 'permission_denied' | 'rate_limit' | 'invalid_credentials';
  /** Probability of occurrence (0-1) */
  probability: number;
  /** Error message template */
  messageTemplate: string;
  /** Recovery steps suggestion */
  recoverySteps: string[];
}

/**
 * Authentication configuration constants.
 */
export interface DemoAuthConfig {
  /** Default token TTL in milliseconds */
  DEFAULT_TOKEN_TTL_MS: number;
  /** Default refresh token TTL in milliseconds */
  DEFAULT_REFRESH_TOKEN_TTL_MS: number;
  /** Token refresh threshold (milliseconds before expiry) */
  TOKEN_REFRESH_THRESHOLD_MS: number;
  /** Session TTL in milliseconds */
  SESSION_TTL_MS: number;
  /** Cross-tab sync enabled */
  CROSS_TAB_SYNC_ENABLED: boolean;
  /** Mock authentication error scenarios enabled */
  ERROR_SCENARIOS_ENABLED: boolean;
  /** Educational annotations enabled */
  EDUCATION_MODE_ENABLED: boolean;
}

/**
 * Demo state context interface.
 * Defines the shape of the global demo state.
 */
export interface DemoState {
  /** Current authenticated user */
  user: DemoUser | null;
  /** Current tenant */
  tenant: DemoTenant | null;
  /** Available providers */
  providers: DemoProvider[];
  /** User's connections */
  connections: DemoConnection[];
  /** Discovered signals */
  signals: DemoSignal[];
  /** Grounded signals */
  groundedSignals: DemoGroundedSignal[];

  // Entity collections
  /** Sync jobs for connections */
  syncJobs: DemoSyncJob[];
  /** Webhook events received */
  webhooks: DemoWebhook[];
  /** OAuth tokens for connections */
  tokens: DemoToken[];
  /** Rate limiting status */
  rateLimits: DemoRateLimit[];

  // Enhanced authentication collections
  /** Authentication sessions */
  authSessions: DemoAuthSession[];
  /** JWT-style tokens */
  authTokens: DemoAuthToken[];
  /** Provider-specific authentication states */
  providerAuths: DemoProviderAuth[];
  /** Authentication events for monitoring */
  authEvents: DemoAuthEvent[];
  /** Authentication configuration */
  authConfig: DemoAuthConfig;

  /** Loading states */
  loading: {
    /** Connection creation/loading */
    connections: boolean;
    /** Signal scanning/loading */
    signals: boolean;
    /** Signal grounding */
    grounding: boolean;
    /** Sync job loading */
    syncJobs: boolean;
    /** Webhook loading */
    webhooks: boolean;
    /** Token loading */
    tokens: boolean;
    /** Rate limit loading */
    rateLimits: boolean;
    /** Authentication session loading */
    authSessions: boolean;
    /** Authentication token loading */
    authTokens: boolean;
    /** Provider authentication loading */
    providerAuths: boolean;
    /** Authentication events loading */
    authEvents: boolean;
  };
  /** Error states */
  errors: {
    /** Connection errors */
    connections?: string;
    /** Signal errors */
    signals?: string;
    /** Grounding errors */
    grounding?: string;
    /** Sync job errors */
    syncJobs?: string;
    /** Webhook errors */
    webhooks?: string;
    /** Token errors */
    tokens?: string;
    /** Rate limit errors */
    rateLimits?: string;
    /** Authentication session errors */
    authSessions?: string;
    /** Authentication token errors */
    authTokens?: string;
    /** Provider authentication errors */
    providerAuths?: string;
    /** Authentication event errors */
    authEvents?: string;
  };

  /** Demo configuration */
  config: DemoConfig;
}

/**
 * Demo state action types.
 * Defines the actions that can modify the demo state.
 */
export type DemoAction =
  | { type: 'SET_USER'; payload: DemoUser }
  | { type: 'SET_TENANT'; payload: DemoTenant }
  | { type: 'SET_PROVIDERS'; payload: DemoProvider[] }
  | { type: 'SET_CONNECTIONS'; payload: DemoConnection[] }
  | { type: 'ADD_CONNECTION'; payload: DemoConnection }
  | { type: 'UPDATE_CONNECTION'; payload: { id: string; updates: Partial<DemoConnection> } }
  | { type: 'SET_SIGNALS'; payload: DemoSignal[] }
  | { type: 'ADD_SIGNALS'; payload: DemoSignal[] }
  | { type: 'SET_GROUNDED_SIGNALS'; payload: DemoGroundedSignal[] }
  | { type: 'ADD_GROUNDED_SIGNAL'; payload: DemoGroundedSignal }

  // Entity actions
  | { type: 'SET_SYNC_JOBS'; payload: DemoSyncJob[] }
  | { type: 'ADD_SYNC_JOB'; payload: DemoSyncJob }
  | { type: 'UPDATE_SYNC_JOB'; payload: { id: string; updates: Partial<DemoSyncJob> } }
  | { type: 'SET_WEBHOOKS'; payload: DemoWebhook[] }
  | { type: 'ADD_WEBHOOK'; payload: DemoWebhook }
  | { type: 'UPDATE_WEBHOOK'; payload: { id: string; updates: Partial<DemoWebhook> } }
  | { type: 'SET_TOKENS'; payload: DemoToken[] }
  | { type: 'ADD_TOKEN'; payload: DemoToken }
  | { type: 'UPDATE_TOKEN'; payload: { id: string; updates: Partial<DemoToken> } }
  | { type: 'SET_RATE_LIMITS'; payload: DemoRateLimit[] }
  | { type: 'UPDATE_RATE_LIMIT'; payload: { id: string; updates: Partial<DemoRateLimit> } }

  // Enhanced authentication actions
  | { type: 'SET_AUTH_SESSIONS'; payload: DemoAuthSession[] }
  | { type: 'ADD_AUTH_SESSION'; payload: DemoAuthSession }
  | { type: 'UPDATE_AUTH_SESSION'; payload: { id: string; updates: Partial<DemoAuthSession> } }
  | { type: 'SET_AUTH_TOKENS'; payload: DemoAuthToken[] }
  | { type: 'ADD_AUTH_TOKEN'; payload: DemoAuthToken }
  | { type: 'UPDATE_AUTH_TOKEN'; payload: { id: string; updates: Partial<DemoAuthToken> } }
  | { type: 'SET_PROVIDER_AUTHS'; payload: DemoProviderAuth[] }
  | { type: 'ADD_PROVIDER_AUTH'; payload: DemoProviderAuth }
  | { type: 'UPDATE_PROVIDER_AUTH'; payload: { id: string; updates: Partial<DemoProviderAuth> } }
  | { type: 'SET_AUTH_EVENTS'; payload: DemoAuthEvent[] }
  | { type: 'ADD_AUTH_EVENT'; payload: DemoAuthEvent }
  | { type: 'SET_AUTH_CONFIG'; payload: DemoAuthConfig }

  // Configuration actions
  | { type: 'SET_CONFIG'; payload: DemoConfig }
  | { type: 'UPDATE_CONFIG'; payload: Partial<DemoConfig> }

  | { type: 'SET_LOADING'; payload: { key: keyof DemoState['loading']; value: boolean } }
  | { type: 'SET_ERROR'; payload: { key: keyof DemoState['errors']; value?: string } }
  | { type: 'RESET_STATE' };

/**
 * Provider configuration constants.
 * Predefined provider configurations for the demo.
 */
export const DEMO_PROVIDERS: DemoProvider[] = [
  {
    slug: 'github',
    name: 'GitHub',
    description: 'Connect to GitHub repositories to track commits, pull requests, issues, and releases.',
    iconUrl: '/icons/github.svg',
    supportedSignalKinds: [
      'commit',
      'pull_request_opened',
      'pull_request_closed',
      'pull_request_merged',
      'issue_opened',
      'issue_closed',
      'release_published'
    ],
    authType: 'oauth2',
    rateLimit: {
      requestsPerHour: 5000,
      requestsPerMinute: 60
    },
    webhookEvents: ['push', 'pull_request', 'issues', 'release'],
    defaultScopes: ['repo', 'read:org', 'read:user'],
    features: {
      realtimeWebhooks: true,
      historicalSync: true,
      incrementalSync: true,
      crossProviderCorrelation: true
    }
  },
  {
    slug: 'zoho-cliq',
    name: 'Zoho Cliq',
    description: 'Connect to Zoho Cliq to track team conversations, mentions, and collaborative activities.',
    iconUrl: '/icons/zoho-cliq.svg',
    supportedSignalKinds: [
      'message_sent',
      'message_received',
      'mention',
      'thread_started',
      'thread_replied'
    ],
    authType: 'oauth2',
    rateLimit: {
      requestsPerHour: 10000,
      requestsPerMinute: 100
    },
    webhookEvents: ['message', 'mention', 'reaction'],
    defaultScopes: ['Cliq.messages.READ', 'Cliq.channels.READ'],
    features: {
      realtimeWebhooks: true,
      historicalSync: true,
      incrementalSync: true,
      crossProviderCorrelation: true
    }
  },
  {
    slug: 'slack',
    name: 'Slack',
    description: 'Connect to Slack to track team conversations, channels, mentions, reactions, and file sharing.',
    iconUrl: '/icons/slack.svg',
    supportedSignalKinds: [
      'message_sent',
      'message_received',
      'mention',
      'reaction_added',
      'file_shared',
      'channel_created',
      'user_added'
    ],
    authType: 'oauth2',
    rateLimit: {
      requestsPerHour: 5000,
      requestsPerMinute: 200
    },
    webhookEvents: ['message', 'mention', 'reaction', 'file_shared', 'channel_created'],
    defaultScopes: ['channels:read', 'users:read', 'files:read'],
    features: {
      realtimeWebhooks: true,
      historicalSync: true,
      incrementalSync: true,
      crossProviderCorrelation: true
    }
  },
  {
    slug: 'google-workspace',
    name: 'Google Workspace',
    description: 'Connect to Google Workspace to track emails, documents, calendar events, and Drive activities.',
    iconUrl: '/icons/google-workspace.svg',
    supportedSignalKinds: [
      'email_sent',
      'email_received',
      'document_created',
      'document_shared',
      'calendar_event_created',
      'drive_file_modified',
      'spreadsheet_updated'
    ],
    authType: 'oauth2',
    rateLimit: {
      requestsPerHour: 10000,
      requestsPerMinute: 100
    },
    webhookEvents: ['mail.receive', 'drive.change', 'calendar.event'],
    defaultScopes: ['gmail.readonly', 'drive.readonly', 'calendar.readonly'],
    features: {
      realtimeWebhooks: true,
      historicalSync: true,
      incrementalSync: true,
      crossProviderCorrelation: true
    }
  },
  {
    slug: 'jira',
    name: 'Jira',
    description: 'Connect to Jira to track issues, sprints, workflow transitions, and project management activities.',
    iconUrl: '/icons/jira.svg',
    supportedSignalKinds: [
      'issue_created',
      'issue_updated',
      'issue_assigned',
      'sprint_started',
      'workflow_transition',
      'comment_added',
      'resolution_set'
    ],
    authType: 'oauth2',
    rateLimit: {
      requestsPerHour: 2500,
      requestsPerMinute: 60
    },
    webhookEvents: ['jira:issue_created', 'jira:sprint_started', 'jira:worklog_updated'],
    defaultScopes: ['READ', 'BROWSE', 'ADD_COMMENTS'],
    features: {
      realtimeWebhooks: true,
      historicalSync: true,
      incrementalSync: true,
      crossProviderCorrelation: true
    }
  }
];