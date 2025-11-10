'use client';

import React, { createContext, useContext, useReducer, ReactNode } from 'react';
import {
  DemoState,
  DemoAction,
  DemoConfig,
  DemoUser,
  DemoTenant,
  DemoConnection,
  DemoSignal,
  DemoGroundedSignal,
  DemoSyncJob,
  DemoWebhook,
  DemoToken,
  DemoRateLimit
} from './types';

/**
 * Initial demo state.
 * Sets up the starting point for the mock UX demo.
 */
const initialState: DemoState = {
  user: null,
  tenant: null,
  providers: [],
  connections: [],
  signals: [],
  groundedSignals: [],

  // New entity collections
  syncJobs: [],
  webhooks: [],
  tokens: [],
  rateLimits: [],

  loading: {
    connections: false,
    signals: false,
    grounding: false,
    syncJobs: false,
    webhooks: false,
    tokens: false,
    rateLimits: false,
  },
  errors: {
    connections: undefined,
    signals: undefined,
    grounding: undefined,
    syncJobs: undefined,
    webhooks: undefined,
    tokens: undefined,
    rateLimits: undefined,
  },

  // Demo configuration
  config: {
    signalFrequency: 'medium',
    errorRate: '10%',
    timingMode: 'realistic',
    providerComplexity: 'detailed',
  },
};

/**
 * Demo state reducer.
 * Handles all state transitions for the mock demo.
 */
function demoReducer(state: DemoState, action: DemoAction): DemoState {
  switch (action.type) {
    case 'SET_USER':
      return {
        ...state,
        user: action.payload,
      };

    case 'SET_TENANT':
      return {
        ...state,
        tenant: action.payload,
      };

    case 'SET_PROVIDERS':
      return {
        ...state,
        providers: action.payload,
      };

    case 'SET_CONNECTIONS':
      return {
        ...state,
        connections: action.payload,
      };

    case 'ADD_CONNECTION':
      return {
        ...state,
        connections: [...state.connections, action.payload],
      };

    case 'UPDATE_CONNECTION':
      return {
        ...state,
        connections: state.connections.map(conn =>
          conn.id === action.payload.id
            ? { ...conn, ...action.payload.updates }
            : conn
        ),
      };

    case 'SET_SIGNALS':
      return {
        ...state,
        signals: action.payload,
      };

    case 'ADD_SIGNALS':
      return {
        ...state,
        signals: [...state.signals, ...action.payload],
      };

    case 'SET_GROUNDED_SIGNALS':
      return {
        ...state,
        groundedSignals: action.payload,
      };

    case 'ADD_GROUNDED_SIGNAL':
      return {
        ...state,
        groundedSignals: [...state.groundedSignals, action.payload],
      };

    // New entity actions
    case 'SET_SYNC_JOBS':
      return {
        ...state,
        syncJobs: action.payload,
      };

    case 'ADD_SYNC_JOB':
      return {
        ...state,
        syncJobs: [...state.syncJobs, action.payload],
      };

    case 'UPDATE_SYNC_JOB':
      return {
        ...state,
        syncJobs: state.syncJobs.map(job =>
          job.id === action.payload.id
            ? { ...job, ...action.payload.updates }
            : job
        ),
      };

    case 'SET_WEBHOOKS':
      return {
        ...state,
        webhooks: action.payload,
      };

    case 'ADD_WEBHOOK':
      return {
        ...state,
        webhooks: [...state.webhooks, action.payload],
      };

    case 'UPDATE_WEBHOOK':
      return {
        ...state,
        webhooks: state.webhooks.map(webhook =>
          webhook.id === action.payload.id
            ? { ...webhook, ...action.payload.updates }
            : webhook
        ),
      };

    case 'SET_TOKENS':
      return {
        ...state,
        tokens: action.payload,
      };

    case 'ADD_TOKEN':
      return {
        ...state,
        tokens: [...state.tokens, action.payload],
      };

    case 'UPDATE_TOKEN':
      return {
        ...state,
        tokens: state.tokens.map(token =>
          token.id === action.payload.id
            ? { ...token, ...action.payload.updates }
            : token
        ),
      };

    case 'SET_RATE_LIMITS':
      return {
        ...state,
        rateLimits: action.payload,
      };

    case 'UPDATE_RATE_LIMIT':
      return {
        ...state,
        rateLimits: state.rateLimits.map(rateLimit =>
          rateLimit.id === action.payload.id
            ? { ...rateLimit, ...action.payload.updates }
            : rateLimit
        ),
      };

    // Configuration actions
    case 'SET_CONFIG':
      return {
        ...state,
        config: action.payload,
      };

    case 'UPDATE_CONFIG':
      return {
        ...state,
        config: {
          ...state.config,
          ...action.payload,
        },
      };

    case 'SET_LOADING':
      return {
        ...state,
        loading: {
          ...state.loading,
          [action.payload.key]: action.payload.value,
        },
      };

    case 'SET_ERROR':
      return {
        ...state,
        errors: {
          ...state.errors,
          [action.payload.key]: action.payload.value,
        },
      };

    case 'RESET_STATE':
      return initialState;

    default:
      // Type safety: exhaustiveness check
      const _exhaustiveCheck: never = action;
      throw new Error(`Unhandled action type: ${_exhaustiveCheck}`);
  }
}

/**
 * Demo context interface.
 * Defines the shape of the demo context value.
 */
interface DemoContextValue {
  /** Current state */
  state: DemoState;
  /** Dispatch function for actions */
  dispatch: React.Dispatch<DemoAction>;
}

/**
 * Demo context instance.
 * Provides global state management for the mock demo.
 */
const DemoContext = createContext<DemoContextValue | undefined>(undefined);

/**
 * DemoProvider component.
 * Wraps the app with state management context.
 */
export function DemoProvider({ children }: { children: ReactNode }) {
  const [state, dispatch] = useReducer(demoReducer, initialState);

  const value = {
    state,
    dispatch,
  };

  return React.createElement(
    DemoContext.Provider,
    { value },
    children
  );
}

/**
 * Hook to access demo context.
 * Throws an error if used outside of DemoProvider.
 */
function useDemoContext(): DemoContextValue {
  const context = useContext(DemoContext);
  if (context === undefined) {
    throw new Error('useDemoContext must be used within a DemoProvider');
  }
  return context;
}

/**
 * Convenience hooks for accessing specific parts of the demo state.
 */

/**
 * Hook to access the current user.
 */
export function useDemoUser(): DemoUser | null {
  const { state } = useDemoContext();
  return state.user;
}

/**
 * Hook to access the current tenant.
 */
export function useDemoTenant(): DemoTenant | null {
  const { state } = useDemoContext();
  return state.tenant;
}

/**
 * Hook to access available providers.
 */
export function useDemoProviders() {
  const { state } = useDemoContext();
  return state.providers;
}

/**
 * Hook to access user connections.
 */
export function useDemoConnections(): DemoConnection[] {
  const { state } = useDemoContext();
  return state.connections;
}

/**
 * Hook to access signals.
 */
export function useDemoSignals(): DemoSignal[] {
  const { state } = useDemoContext();
  return state.signals;
}

/**
 * Hook to access grounded signals.
 */
export function useDemoGroundedSignals(): DemoGroundedSignal[] {
  const { state } = useDemoContext();
  return state.groundedSignals;
}

// New entity hooks

/**
 * Hook to access sync jobs.
 */
export function useDemoSyncJobs(): DemoSyncJob[] {
  const { state } = useDemoContext();
  return state.syncJobs;
}

/**
 * Hook to access webhook events.
 */
export function useDemoWebhooks(): DemoWebhook[] {
  const { state } = useDemoContext();
  return state.webhooks;
}

/**
 * Hook to access tokens.
 */
export function useDemoTokens(): DemoToken[] {
  const { state } = useDemoContext();
  return state.tokens;
}

/**
 * Hook to access rate limits.
 */
export function useDemoRateLimits(): DemoRateLimit[] {
  const { state } = useDemoContext();
  return state.rateLimits;
}

/**
 * Hook to access demo configuration.
 */
export function useDemoConfig(): DemoConfig {
  const { state } = useDemoContext();
  return state.config;
}

/**
 * Hook to access loading states.
 */
export function useDemoLoading() {
  const { state } = useDemoContext();
  return state.loading;
}

/**
 * Hook to access error states.
 */
export function useDemoErrors() {
  const { state } = useDemoContext();
  return state.errors;
}

/**
 * Hook to access the dispatch function.
 */
export function useDemoDispatch(): React.Dispatch<DemoAction> {
  const { dispatch } = useDemoContext();
  return dispatch;
}

/**
 * Hook to access the complete state and dispatch.
 * Useful for complex operations that need multiple parts of the state.
 */
export function useDemoState() {
  return useDemoContext();
}

/**
 * Action creators for common demo operations.
 * These provide a more convenient interface for dispatching actions.
 */

/**
 * Sets the current user.
 */
export const setUser = (user: DemoUser): DemoAction => ({
  type: 'SET_USER',
  payload: user,
});

/**
 * Sets the current tenant.
 */
export const setTenant = (tenant: DemoTenant): DemoAction => ({
  type: 'SET_TENANT',
  payload: tenant,
});

/**
 * Sets the available providers.
 */
export const setProviders = (providers: import('./types').DemoProvider[]): DemoAction => ({
  type: 'SET_PROVIDERS',
  payload: providers,
});

/**
 * Adds a new connection.
 */
export const addConnection = (connection: DemoConnection): DemoAction => ({
  type: 'ADD_CONNECTION',
  payload: connection,
});

/**
 * Updates an existing connection.
 */
export const updateConnection = (id: string, updates: Partial<DemoConnection>): DemoAction => ({
  type: 'UPDATE_CONNECTION',
  payload: { id, updates },
});

/**
 * Sets the signals list.
 */
export const setSignals = (signals: DemoSignal[]): DemoAction => ({
  type: 'SET_SIGNALS',
  payload: signals,
});

/**
 * Adds new signals to the existing list.
 */
export const addSignals = (signals: DemoSignal[]): DemoAction => ({
  type: 'ADD_SIGNALS',
  payload: signals,
});

/**
 * Adds a new grounded signal.
 */
export const addGroundedSignal = (groundedSignal: DemoGroundedSignal): DemoAction => ({
  type: 'ADD_GROUNDED_SIGNAL',
  payload: groundedSignal,
});

// New entity action creators

/**
 * Sets the sync jobs list.
 */
export const setSyncJobs = (syncJobs: DemoSyncJob[]): DemoAction => ({
  type: 'SET_SYNC_JOBS',
  payload: syncJobs,
});

/**
 * Adds a new sync job.
 */
export const addSyncJob = (syncJob: DemoSyncJob): DemoAction => ({
  type: 'ADD_SYNC_JOB',
  payload: syncJob,
});

/**
 * Updates an existing sync job.
 */
export const updateSyncJob = (id: string, updates: Partial<DemoSyncJob>): DemoAction => ({
  type: 'UPDATE_SYNC_JOB',
  payload: { id, updates },
});

/**
 * Sets the webhook events list.
 */
export const setWebhooks = (webhooks: DemoWebhook[]): DemoAction => ({
  type: 'SET_WEBHOOKS',
  payload: webhooks,
});

/**
 * Adds a new webhook event.
 */
export const addWebhook = (webhook: DemoWebhook): DemoAction => ({
  type: 'ADD_WEBHOOK',
  payload: webhook,
});

/**
 * Updates an existing webhook event.
 */
export const updateWebhook = (id: string, updates: Partial<DemoWebhook>): DemoAction => ({
  type: 'UPDATE_WEBHOOK',
  payload: { id, updates },
});

/**
 * Sets the tokens list.
 */
export const setTokens = (tokens: DemoToken[]): DemoAction => ({
  type: 'SET_TOKENS',
  payload: tokens,
});

/**
 * Adds a new token.
 */
export const addToken = (token: DemoToken): DemoAction => ({
  type: 'ADD_TOKEN',
  payload: token,
});

/**
 * Updates an existing token.
 */
export const updateToken = (id: string, updates: Partial<DemoToken>): DemoAction => ({
  type: 'UPDATE_TOKEN',
  payload: { id, updates },
});

/**
 * Sets the rate limits list.
 */
export const setRateLimits = (rateLimits: DemoRateLimit[]): DemoAction => ({
  type: 'SET_RATE_LIMITS',
  payload: rateLimits,
});

/**
 * Updates an existing rate limit.
 */
export const updateRateLimit = (id: string, updates: Partial<DemoRateLimit>): DemoAction => ({
  type: 'UPDATE_RATE_LIMIT',
  payload: { id, updates },
});

/**
 * Sets the demo configuration.
 */
export const setConfig = (config: DemoConfig): DemoAction => ({
  type: 'SET_CONFIG',
  payload: config,
});

/**
 * Updates the demo configuration.
 */
export const updateConfig = (updates: Partial<DemoConfig>): DemoAction => ({
  type: 'UPDATE_CONFIG',
  payload: updates,
});

/**
 * Sets loading state for a specific key.
 */
export const setLoading = (key: keyof DemoState['loading'], value: boolean): DemoAction => ({
  type: 'SET_LOADING',
  payload: { key, value },
});

/**
 * Sets error state for a specific key.
 */
export const setError = (key: keyof DemoState['errors'], value?: string): DemoAction => ({
  type: 'SET_ERROR',
  payload: { key, value },
});

/**
 * Resets the entire demo state.
 */
export const resetState = (): DemoAction => ({
  type: 'RESET_STATE',
});