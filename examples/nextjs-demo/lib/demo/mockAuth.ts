/**
 * Mock JWT Implementation for Educational Purposes
 *
 * IMPORTANT: This is NOT a real JWT implementation and has NO cryptographic security.
 * All tokens are mock tokens for demonstration and educational purposes only.
 *
 * WARNING: These tokens are:
 * - NOT cryptographically signed
 * - NOT secure for production use
 * - Stored in localStorage (insecure for real auth)
 * - Intentionally transparent for learning
 */

import {
  DemoAuthToken,
  DemoAuthSession,
  DemoAuthEvent,
  DemoAuthConfig,
  DemoUser,
  DemoTenant
} from './types';
import { generateId } from './id';

/**
 * Mock authentication configuration.
 * These constants control the behavior of the mock authentication system.
 */
export const MOCK_AUTH_CONFIG: DemoAuthConfig = {
  DEFAULT_TOKEN_TTL_MS: 60 * 60 * 1000, // 1 hour
  DEFAULT_REFRESH_TOKEN_TTL_MS: 7 * 24 * 60 * 60 * 1000, // 7 days
  TOKEN_REFRESH_THRESHOLD_MS: 5 * 60 * 1000, // 5 minutes before expiry
  SESSION_TTL_MS: 24 * 60 * 60 * 1000, // 24 hours
  CROSS_TAB_SYNC_ENABLED: true,
  ERROR_SCENARIOS_ENABLED: true,
  EDUCATION_MODE_ENABLED: true,
};

/**
 * Mock JWT header configuration.
 */
const MOCK_JWT_HEADER = {
  alg: 'HS256', // Mock algorithm - NOT REAL
  typ: 'JWT' as const,
};

/**
 * Mock signature for all tokens.
 * In real JWT, this would be cryptographically signed.
 */
const MOCK_SIGNATURE = 'MOCK_SIGNATURE_NOT_CRYPTOGRAPHICALLY_VALID';

/**
 * Creates a mock JWT-style token for educational purposes.
 *
 * @param type - Token type (access, refresh, id)
 * @param user - User the token is for
 * @param tenant - Tenant the token is scoped to
 * @param sessionId - Session identifier
 * @param scopes - Authorization scopes
 * @param providerId - Optional provider ID for provider-specific tokens
 * @param expiresIn - Token expiration time in milliseconds
 * @returns Mock JWT token object
 */
export function createMockAuthToken({
  type,
  user,
  tenant,
  sessionId,
  scopes = [],
  providerId,
  expiresIn = MOCK_AUTH_CONFIG.DEFAULT_TOKEN_TTL_MS,
}: {
  type: 'access' | 'refresh' | 'id';
  user: DemoUser;
  tenant: DemoTenant;
  sessionId: string;
  scopes?: string[];
  providerId?: string;
  expiresIn?: number;
}): DemoAuthToken {
  const now = Math.floor(Date.now() / 1000);
  const exp = now + Math.floor(expiresIn / 1000);

  const payload = {
    sub: user.id,
    email: user.email,
    tenantId: tenant.id,
    sessionId,
    scopes,
    iat: now,
    exp,
    ...(providerId && { providerId }),
    ...(type === 'id' && { tokenUse: 'id' }),
    meta: {
      type,
      tenantName: tenant.name,
      userName: user.name,
      createdAt: new Date(now * 1000).toISOString(),
    },
  };

  return {
    id: generateId('auth-token'),
    type,
    providerId,
    header: MOCK_JWT_HEADER,
    payload,
    signature: MOCK_SIGNATURE,
  };
}

/**
 * Encodes a mock JWT token to a string format (for display purposes).
 * This mimics the real JWT string format: header.payload.signature
 *
 * @param token - Mock JWT token object
 * @returns JWT-formatted string (base64 encoded)
 */
export function encodeMockJwtToken(token: DemoAuthToken): string {
  const headerEncoded = btoa(JSON.stringify(token.header));
  const payloadEncoded = btoa(JSON.stringify(token.payload));
  const signatureEncoded = btoa(token.signature);

  return `${headerEncoded}.${payloadEncoded}.${signatureEncoded}`;
}

/**
 * Decodes a mock JWT token string back to a token object.
 * This validates the structure but NOT the cryptographic signature.
 *
 * @param jwtString - JWT-formatted string
 * @returns Decoded mock JWT token object
 * @throws Error if token format is invalid
 */
export function decodeMockJwtToken(jwtString: string): DemoAuthToken {
  const parts = jwtString.split('.');
  if (parts.length !== 3) {
    throw new Error('Invalid JWT format: expected header.payload.signature');
  }

  try {
    const header = JSON.parse(atob(parts[0]));
    const payload = JSON.parse(atob(parts[1]));
    const signature = atob(parts[2]);

    // Basic validation of mock JWT structure
    if (header.typ !== 'JWT' || !payload.sub || !payload.exp) {
      throw new Error('Invalid JWT structure: missing required fields');
    }

    return {
      id: payload.sub, // Use user ID as token ID for mock purposes
      type: payload.tokenUse === 'id' ? 'id' : payload.providerId ? 'access' : 'access',
      providerId: payload.providerId,
      header,
      payload,
      signature,
    };
  } catch (error) {
    throw new Error(`Failed to decode JWT token: ${error instanceof Error ? error.message : 'Unknown error'}`);
  }
}

/**
 * Validates a mock JWT token.
 * This checks basic structure and expiration but NOT cryptographic signature.
 *
 * @param token - Mock JWT token object
 * @returns Validation result with reason if invalid
 */
export function validateMockJwtToken(token: DemoAuthToken): {
  valid: boolean;
  reason?: string;
  expiresSoon?: boolean;
} {
  const now = Math.floor(Date.now() / 1000);

  // Check expiration
  if (token.payload.exp < now) {
    return { valid: false, reason: 'Token has expired' };
  }

  // Check if token expires soon (within threshold)
  const expiresAt = token.payload.exp * 1000;
  const threshold = now * 1000 + MOCK_AUTH_CONFIG.TOKEN_REFRESH_THRESHOLD_MS;
  const expiresSoon = expiresAt <= threshold;

  // Basic structure validation
  if (!token.payload.sub || !token.payload.sessionId || !token.payload.tenantId) {
    return { valid: false, reason: 'Token missing required fields' };
  }

  return { valid: true, expiresSoon };
}

/**
 * Creates a mock authentication session.
 *
 * @param user - User the session is for
 * @param tenant - Tenant the session is scoped to
 * @param primaryTokenId - Primary access token ID
 * @param refreshTokenId - Optional refresh token ID
 * @returns Mock authentication session
 */
export function createMockAuthSession({
  user,
  tenant,
  primaryTokenId,
  refreshTokenId,
}: {
  user: DemoUser;
  tenant: DemoTenant;
  primaryTokenId: string;
  refreshTokenId?: string;
}): DemoAuthSession {
  const now = Date.now();
  const expiresAt = now + MOCK_AUTH_CONFIG.SESSION_TTL_MS;

  return {
    id: generateId('auth-session'),
    userId: user.id,
    tenantId: tenant.id,
    status: 'active',
    createdAt: now,
    updatedAt: now,
    expiresAt,
    primaryTokenId,
    refreshTokenId,
    providerAuthIds: [],
    lastActivityAt: now,
    meta: {
      userAgent: typeof window !== 'undefined' ? window.navigator.userAgent : 'unknown',
      createdAt: new Date(now).toISOString(),
      educationalNote: 'This is a mock session for demonstration purposes',
    },
  };
}

/**
 * Logs an authentication event for monitoring and educational purposes.
 *
 * @param event - Authentication event details
 * @returns Created authentication event
 */
export function logAuthEvent(event: Omit<DemoAuthEvent, 'id' | 'timestamp'>): DemoAuthEvent {
  return {
    ...event,
    id: generateId('auth-event'),
    timestamp: Date.now(),
  };
}

/**
 * Refreshes an access token using a refresh token.
 * This simulates the token refresh flow with optional error scenarios.
 *
 * @param refreshToken - Mock refresh token
 * @param user - User the token is for
 * @param tenant - Tenant the token is scoped to
 * @param sessionId - Session identifier
 * @returns New access token or throws error
 */
export function refreshMockAccessToken({
  refreshToken,
  user,
  tenant,
  sessionId,
}: {
  refreshToken: DemoAuthToken;
  user: DemoUser;
  tenant: DemoTenant;
  sessionId: string;
}): DemoAuthToken {
  // Validate refresh token
  const validation = validateMockJwtToken(refreshToken);
  if (!validation.valid) {
    throw new Error(`Refresh token invalid: ${validation.reason}`);
  }

  // Simulate occasional refresh failures (10% chance when error scenarios enabled)
  if (MOCK_AUTH_CONFIG.ERROR_SCENARIOS_ENABLED && Math.random() < 0.1) {
    throw new Error('Token refresh failed: Simulated network error');
  }

  // Create new access token with refresh token scopes
  return createMockAuthToken({
    type: 'access',
    user,
    tenant,
    sessionId,
    scopes: refreshToken.payload.scopes,
    expiresIn: MOCK_AUTH_CONFIG.DEFAULT_TOKEN_TTL_MS,
  });
}

/**
 * Checks if a session is expired or needs refresh.
 *
 * @param session - Authentication session
 * @returns Session status information
 */
export function checkSessionStatus(session: DemoAuthSession): {
  isActive: boolean;
  isExpired: boolean;
  expiresSoon: boolean;
  timeUntilExpiry: number;
} {
  const now = Date.now();
  const isExpired = session.expiresAt <= now;
  const timeUntilExpiry = session.expiresAt - now;
  const expiresSoon = timeUntilExpiry <= MOCK_AUTH_CONFIG.TOKEN_REFRESH_THRESHOLD_MS;
  const isActive = session.status === 'active' && !isExpired;

  return {
    isActive,
    isExpired,
    expiresSoon,
    timeUntilExpiry,
  };
}

/**
 * Mock localStorage key for authentication data.
 */
export const AUTH_STORAGE_KEY = 'poblysh_demo_auth';

/**
 * Saves authentication session to localStorage.
 *
 * @param session - Authentication session to save
 * @param tokens - Associated tokens to save
 */
export function saveAuthToStorage({
  session,
  tokens,
}: {
  session: DemoAuthSession;
  tokens: DemoAuthToken[];
}): void {
  if (typeof window === 'undefined') return;

  try {
    const authData = {
      session,
      tokens,
      savedAt: Date.now(),
      version: '1.0',
    };
    localStorage.setItem(AUTH_STORAGE_KEY, JSON.stringify(authData));
  } catch (error) {
    console.warn('Failed to save auth data to localStorage:', error);
  }
}

/**
 * Loads authentication session from localStorage.
 *
 * @returns Loaded auth data or null if not found
 */
export function loadAuthFromStorage(): {
  session?: DemoAuthSession;
  tokens?: DemoAuthToken[];
} | null {
  if (typeof window === 'undefined') return null;

  try {
    const stored = localStorage.getItem(AUTH_STORAGE_KEY);
    if (!stored) return null;

    const authData = JSON.parse(stored);

    // Validate basic structure
    if (!authData.session || !Array.isArray(authData.tokens)) {
      return null;
    }

    return {
      session: authData.session,
      tokens: authData.tokens,
    };
  } catch (error) {
    console.warn('Failed to load auth data from localStorage:', error);
    return null;
  }
}

/**
 * Clears authentication data from localStorage.
 */
export function clearAuthFromStorage(): void {
  if (typeof window === 'undefined') return;

  try {
    localStorage.removeItem(AUTH_STORAGE_KEY);
  } catch (error) {
    console.warn('Failed to clear auth data from localStorage:', error);
  }
}

/**
 * Cross-tab synchronization event.
 */
export interface AuthSyncEvent {
  type: 'session-updated' | 'session-cleared';
  sessionId?: string;
  timestamp: number;
}

/**
 * Broadcasts authentication state changes to other tabs.
 *
 * @param event - Sync event to broadcast
 */
export function broadcastAuthSync(event: AuthSyncEvent): void {
  if (typeof window === 'undefined' || !MOCK_AUTH_CONFIG.CROSS_TAB_SYNC_ENABLED) return;

  try {
    localStorage.setItem('poblysh_demo_auth_sync', JSON.stringify(event));
    // Clear the sync event after a short delay
    setTimeout(() => {
      localStorage.removeItem('poblysh_demo_auth_sync');
    }, 100);
  } catch (error) {
    console.warn('Failed to broadcast auth sync:', error);
  }
}

/**
 * Listens for cross-tab authentication sync events.
 *
 * @param callback - Function to call when sync event occurs
 * @returns Cleanup function to stop listening
 */
export function listenForAuthSync(callback: (event: AuthSyncEvent) => void): () => void {
  if (typeof window === 'undefined' || !MOCK_AUTH_CONFIG.CROSS_TAB_SYNC_ENABLED) {
    return () => {}; // No-op
  }

  const handleStorageChange = (e: StorageEvent) => {
    if (e.key === 'poblysh_demo_auth_sync' && e.newValue) {
      try {
        const event = JSON.parse(e.newValue) as AuthSyncEvent;
        callback(event);
      } catch (error) {
        console.warn('Failed to parse auth sync event:', error);
      }
    }
  };

  window.addEventListener('storage', handleStorageChange);

  // Return cleanup function
  return () => {
    window.removeEventListener('storage', handleStorageChange);
  };
}

/**
 * Educational JWT inspection utilities.
 * These functions help users understand JWT structure and concepts.
 */

/**
 * Gets human-readable token information for educational display.
 *
 * @param token - Mock JWT token
 * @returns Formatted token information
 */
export function getTokenEducationalInfo(token: DemoAuthToken): {
  type: string;
  issuer: string;
  subject: string;
  audience: string;
  issuedAt: string;
  expiresAt: string;
  scopes: string[];
  timeUntilExpiry: string;
  educationalNotes: string[];
} {
  const now = Math.floor(Date.now() / 1000);
  const timeUntilExpirySeconds = token.payload.exp - now;
  const timeUntilExpiry = timeUntilExpirySeconds > 0
    ? formatDuration(timeUntilExpirySeconds * 1000)
    : 'Expired';

  const educationalNotes: string[] = [
    'This is a MOCK JWT token for educational purposes only',
    'The signature is NOT cryptographically valid',
    'Real JWT tokens would be signed with a secret key',
    'Never store real tokens in localStorage in production',
  ];

  if (token.providerId) {
    educationalNotes.push('This is a provider-specific token for external API access');
  } else {
    educationalNotes.push('This is a core application token for authentication');
  }

  return {
    type: token.type.toUpperCase(),
    issuer: 'Poblysh Demo (Mock)',
    subject: token.payload.sub,
    audience: token.payload.tenantId,
    issuedAt: new Date(token.payload.iat * 1000).toLocaleString(),
    expiresAt: new Date(token.payload.exp * 1000).toLocaleString(),
    scopes: token.payload.scopes,
    timeUntilExpiry,
    educationalNotes,
  };
}

/**
 * Formats a duration in milliseconds to a human-readable string.
 *
 * @param ms - Duration in milliseconds
 * @returns Formatted duration string
 */
function formatDuration(ms: number): string {
  const seconds = Math.floor(ms / 1000);
  const minutes = Math.floor(seconds / 60);
  const hours = Math.floor(minutes / 60);
  const days = Math.floor(hours / 24);

  if (days > 0) return `${days} day${days > 1 ? 's' : ''}`;
  if (hours > 0) return `${hours} hour${hours > 1 ? 's' : ''}`;
  if (minutes > 0) return `${minutes} minute${minutes > 1 ? 's' : ''}`;
  return `${seconds} second${seconds > 1 ? 's' : ''}`;
}