/**
 * Authentication Educational Content
 *
 * This module provides educational content and tooltips for authentication concepts
 * demonstrated in the Poblysh Connectors demo. All content is designed to help users
 * understand real-world authentication patterns through interactive learning.
 */

/**
 * Educational tooltip content for authentication concepts.
 */
export const AUTH_EDUCATIONAL_TOOLTIPS = {
  // JWT Concepts
  jwt_token: {
    title: 'JSON Web Token (JWT)',
    content: `
      A JWT is a compact, URL-safe means of representing claims to be transferred between two parties.

      <strong>Structure:</strong>
      • Header: Algorithm and token type
      • Payload: Claims (user info, permissions, expiration)
      • Signature: Cryptographic verification (MOCK in this demo)

      <strong>IMPORTANT:</strong> This demo uses mock JWTs with no real security.
      Real applications would use cryptographically signed tokens and secure storage.
    `,
  },

  token_expiration: {
    title: 'Token Expiration',
    content: `
      Access tokens have short lifespans (typically 1 hour) for security.
      When they expire, applications use refresh tokens to get new access tokens
      without requiring the user to log in again.

      <strong>Why short-lived?</strong>
      • Limits damage if token is compromised
      • Allows revocation of access
      • Forces regular re-authentication
    `,
  },

  oauth_flow: {
    title: 'OAuth 2.0 Flow',
    content: `
      OAuth 2.0 allows applications to access user data on other services without
      sharing passwords.

      <strong>Steps:</strong>
      1. User clicks "Connect" → Redirect to provider
      2. User logs in and consents to permissions
      3. Provider redirects back with authorization code
      4. Application exchanges code for access token
      5. Application uses token to access user data

      This demo simulates this flow without real OAuth calls.
    `,
  },

  scopes: {
    title: 'OAuth Scopes',
    content: `
      Scopes define what permissions an application has. They're requested during
      OAuth flow and enforced by the provider's API.

      <strong>Examples:</strong>
      • 'repo' - Access to repositories (GitHub)
      • 'read:user' - Read user profile data
      • 'emails.readonly' - Read-only email access

      <strong>Principle of Least Privilege:</strong>
      Only request the minimum scopes needed for your application.
    `,
  },

  session_management: {
    title: 'Session Management',
    content: `
      Sessions track user authentication state across requests.

      <strong>Session Components:</strong>
      • Session ID: Unique identifier
      • User context: Who is logged in
      • Tenant context: Which organization/workspace
      • Expiration: When session becomes invalid
      • Activity tracking: Last user interaction

      Real applications store sessions securely on the server with HttpOnly cookies.
    `,
  },

  cross_tab_sync: {
    title: 'Cross-Tab Synchronization',
    content: `
      When users have multiple tabs open, authentication state should stay synchronized.

      <strong>Real Implementation:</strong>
      • Server-side sessions with cookies (automatic sync)
      • BroadcastChannel API for client-side events
      • WebSockets for real-time updates

      <strong>Demo Implementation:</strong>
      • localStorage events for tab communication
      • Explicit storage of session data (educational only)
    `,
  },

  refresh_tokens: {
    title: 'Refresh Tokens',
    content: `
      Refresh tokens allow applications to obtain new access tokens without
      requiring users to log in again.

      <strong>Characteristics:</strong>
      • Longer lifespan (days to weeks)
      • Securely stored (HttpOnly cookies in production)
      • Can be revoked by the server
      • Single-use or rotation-based

      <strong>Security Note:</strong> This demo stores refresh tokens in localStorage
      for educational purposes, which is insecure in production.
    `,
  },

  provider_auth: {
    title: 'Provider-Specific Authentication',
    content: `
      Each external service (GitHub, Google, Zoho) requires its own authentication.

      <strong>Mapping to Connectors:</strong>
      • Demo Provider Auth → Connection Entity
      • Provider Scopes → Connection Permissions
      • Provider Tokens → Connection Credentials

      This shows how Connectors manages authentication for multiple services
      within a single tenant context.
    `,
  },

  security_best_practices: {
    title: 'Security Best Practices',
    content: `
      <strong>Production Security (Not in Demo):</strong>

      • Use HttpOnly, Secure cookies for tokens
      • Implement CSRF protection
      • Use short-lived access tokens
      • Validate JWT signatures with proper secrets
      • Store refresh tokens securely
      • Implement rate limiting
      • Log authentication events for monitoring
      • Use HTTPS everywhere

      <strong>Demo vs Production:</strong>
      This demo intentionally uses insecure patterns for educational visibility.
    `,
  },
} as const;

/**
 * Provider-specific educational content.
 */
export const PROVIDER_EDUCATIONAL_CONTENT = {
  github: {
    name: 'GitHub OAuth',
    description: 'GitHub uses OAuth 2.0 for third-party application access.',
    typicalScopes: [
      { scope: 'repo', description: 'Access to repositories (full control)' },
      { scope: 'read:org', description: 'Read organization membership' },
      { scope: 'read:user', description: 'Read user profile data' },
      { scope: 'user:email', description: 'Read user email addresses' },
    ],
    authFlow: 'Standard OAuth 2.0 with authorization code flow',
    tokenUse: 'Access GitHub API v4 (GraphQL) and v3 (REST)',
    educationalNotes: [
      'GitHub OAuth is great for git operations, repository management, and CI/CD integration',
      'The "repo" scope is very powerful - request minimal scopes needed',
      'GitHub provides webhooks for real-time events (pushes, PRs, issues)',
    ],
  },

  'google-workspace': {
    name: 'Google Workspace OAuth',
    description: 'Google Workspace uses OAuth 2.0 with Google-specific scopes.',
    typicalScopes: [
      { scope: 'gmail.readonly', description: 'Read Gmail messages' },
      { scope: 'drive.readonly', description: 'Read Google Drive files' },
      { scope: 'calendar.readonly', description: 'Read calendar events' },
      { scope: 'contacts.readonly', description: 'Read contacts' },
    ],
    authFlow: 'Google OAuth 2.0 with consent screen',
    tokenUse: 'Access Google Workspace APIs (Gmail, Drive, Calendar, etc.)',
    educationalNotes: [
      'Google Workspace provides unified access to productivity tools',
      'Admin policies can restrict which applications users can authorize',
      'Google implements automatic token refresh and revocation',
    ],
  },

  'zoho-cliq': {
    name: 'Zoho Cliq OAuth',
    description: 'Zoho Cliq uses OAuth 2.0 for team collaboration platform access.',
    typicalScopes: [
      { scope: 'Cliq.messages.READ', description: 'Read team messages' },
      { scope: 'Cliq.channels.READ', description: 'Read channel information' },
      { scope: 'ZohoContacts.users.READ', description: 'Read user information' },
    ],
    authFlow: 'Zoho OAuth 2.0 with organization context',
    tokenUse: 'Access Zoho Cliq API for team communication',
    educationalNotes: [
      'Zoho Cliq is part of the larger Zoho productivity suite',
      'Scopes are prefixed with service names (Cliq., ZohoContacts.)',
      'Organization-level admin controls available',
    ],
  },

  jira: {
    name: 'Atlassian Jira OAuth',
    description: 'Jira uses OAuth 2.0 for project management and issue tracking access.',
    typicalScopes: [
      { scope: 'READ', description: 'Read Jira data' },
      { scope: 'BROWSE', description: 'Browse projects and issues' },
      { scope: 'ADD_COMMENTS', description: 'Add comments to issues' },
    ],
    authFlow: 'Atlassian OAuth 2.0 with product context',
    tokenUse: 'Access Jira REST API for project management',
    educationalNotes: [
      'Jira OAuth can be scoped to specific projects',
      'Atlassian provides unified auth across multiple products',
      'Fine-grained permissions available for enterprise use',
    ],
  },

  slack: {
    name: 'Slack OAuth',
    description: 'Slack uses OAuth 2.0 with bot tokens and user tokens.',
    typicalScopes: [
      { scope: 'channels:read', description: 'Read channel information' },
      { scope: 'users:read', description: 'Read user information' },
      { scope: 'files:read', description: 'Read file information' },
      { scope: 'chat:write', description: 'Send messages' },
    ],
    authFlow: 'Slack OAuth 2.0 with bot and user token types',
    tokenUse: 'Access Slack Web API for team communication',
    educationalNotes: [
      'Slack provides different token types (user vs bot tokens)',
      'Granular permission system with workspace admin controls',
      'Real-time events available through the Slack Events API',
    ],
  },
} as const;

/**
 * Authentication flow step descriptions for educational content.
 */
export const AUTH_FLOW_STEPS = {
  login: {
    title: 'Login Process',
    steps: [
      {
        step: 1,
        title: 'User Enters Credentials',
        description: 'User provides email/password or clicks social login',
        technicalNote: 'Demo uses simple email validation, production would use secure password hashing',
      },
      {
        step: 2,
        title: 'Identity Verification',
        description: 'System verifies user identity and loads tenant context',
        technicalNote: 'Real apps would check password against secure hash database',
      },
      {
        step: 3,
        title: 'Session Creation',
        description: 'Create session with unique ID and expiration time',
        technicalNote: 'Sessions stored server-side with secure random IDs',
      },
      {
        step: 4,
        title: 'Token Generation',
        description: 'Generate JWT access token and refresh token',
        technicalNote: 'Tokens signed with server secret key using HMAC or RSA',
      },
      {
        step: 5,
        title: 'Storage & Response',
        description: 'Store tokens securely and return to client',
        technicalNote: 'Production uses HttpOnly cookies, demo uses localStorage for visibility',
      },
    ],
  },

  provider_connect: {
    title: 'Provider Connection Process',
    steps: [
      {
        step: 1,
        title: 'Initiate OAuth Flow',
        description: 'User clicks "Connect" for a provider (GitHub, Google, etc.)',
        technicalNote: 'Generate OAuth state parameter to prevent CSRF attacks',
      },
      {
        step: 2,
        title: 'Redirect to Provider',
        description: 'Redirect user to provider\'s authorization page',
        technicalNote: 'Include client_id, redirect_uri, scopes, and state parameters',
      },
      {
        step: 3,
        title: 'User Consent',
        description: 'User logs into provider and grants requested permissions',
        technicalNote: 'Provider shows clear consent screen with requested scopes',
      },
      {
        step: 4,
        title: 'Authorization Callback',
        description: 'Provider redirects back with authorization code',
        technicalNote: 'Verify state parameter matches to prevent CSRF',
      },
      {
        step: 5,
        title: 'Token Exchange',
        description: 'Exchange authorization code for access and refresh tokens',
        technicalNote: 'Server-to-server request using client secret',
      },
      {
        step: 6,
        title: 'Store Provider Auth',
        description: 'Store provider tokens and connection details',
        technicalNote: 'Associate with tenant and user for Connectors API',
      },
    ],
  },

  token_refresh: {
    title: 'Token Refresh Process',
    steps: [
      {
        step: 1,
        title: 'Detect Expiration',
        description: 'Client detects access token is expired or will expire soon',
        technicalNote: 'Check token expiration claim before each API call',
      },
      {
        step: 2,
        title: 'Use Refresh Token',
        description: 'Send refresh token to token endpoint',
        technicalNote: 'Include client credentials for authentication',
      },
      {
        step: 3,
        title: 'Validate Refresh Token',
        description: 'Server validates refresh token and user session',
        technicalNote: 'Check if refresh token is revoked or expired',
      },
      {
        step: 4,
        title: 'Issue New Tokens',
        description: 'Generate new access and refresh tokens',
        technicalNote: 'Implement refresh token rotation for better security',
      },
      {
        step: 5,
        title: 'Update Storage',
        description: 'Store new tokens and invalidate old ones',
        technicalNote: 'Update client storage with new token values',
      },
    ],
  },
} as const;

/**
 * Security comparison between demo and production implementations.
 */
export const SECURITY_COMPARISON = {
  storage: {
    demo: 'localStorage (client-side, JavaScript accessible)',
    production: 'HttpOnly, Secure cookies (server-side, JavaScript inaccessible)',
    risks: ['XSS attacks can steal tokens', 'Client-side storage is tamperable'],
    mitigation: 'Use secure cookies and server-side session storage in production',
  },

  token_validation: {
    demo: 'No cryptographic signature validation',
    production: 'Cryptographic signature verification with server secret',
    risks: ['Tokens can be forged', 'Man-in-the-middle attacks possible'],
    mitigation: 'Always validate JWT signatures in production',
  },

  https: {
    demo: 'HTTP/HTTPS depending on development setup',
    production: 'HTTPS required everywhere',
    risks: ['Token interception over HTTP', 'Man-in-the-middle attacks'],
    mitigation: 'Enforce HTTPS with proper SSL/TLS configuration',
  },

  csrf_protection: {
    demo: 'No CSRF protection implemented',
    production: 'CSRF tokens and SameSite cookies',
    risks: ['Cross-site request forgery attacks', 'Unwanted actions on behalf of user'],
    mitigation: 'Implement CSRF tokens and use SameSite cookie attributes',
  },

  token_rotation: {
    demo: 'No token rotation implemented',
    production: 'Refresh token rotation for better security',
    risks: ['Refresh token reuse attacks', 'Long-lived token exposure'],
    mitigation: 'Implement refresh token rotation and secure storage',
  },
} as const;

/**
 * Mapping between demo concepts and real Connectors API concepts.
 */
export const CONNECTORS_MAPPING = {
  demo_user: {
    concept: 'DemoUser',
    production: 'Poblysh Core User',
    description: 'User identity managed by Poblysh Core authentication system',
  },

  demo_tenant: {
    concept: 'DemoTenant',
    production: 'Poblysh Core Tenant',
    description: 'Organization/workspace that owns connections and data',
  },

  demo_provider_auth: {
    concept: 'DemoProviderAuth',
    production: 'Connectors Connection Entity',
    description: 'Represents authentication state for external provider connection',
  },

  demo_auth_token: {
    concept: 'DemoAuthToken',
    production: 'Connectors Connection Credentials',
    description: 'Encrypted tokens stored in Connection entity for API access',
  },

  demo_scopes: {
    concept: 'OAuth Scopes',
    production: 'Connection Permissions',
    description: 'Defines what actions the connection can perform on the provider',
  },

  demo_session: {
    concept: 'DemoAuthSession',
    production: 'Poblysh Core Session',
    description: 'User session in Poblysh Core, not directly used by Connectors API',
  },
} as const;

/**
 * Type definitions for educational content.
 */
export type EducationalTooltipKey = keyof typeof AUTH_EDUCATIONAL_TOOLTIPS;
export type ProviderEducationalKey = keyof typeof PROVIDER_EDUCATIONAL_CONTENT;
export type AuthFlowKey = keyof typeof AUTH_FLOW_STEPS;

/**
 * Gets educational tooltip content by key.
 */
export function getEducationalTooltip(key: EducationalTooltipKey) {
  return AUTH_EDUCATIONAL_TOOLTIPS[key];
}

/**
 * Gets provider educational content by key.
 */
export function getProviderEducationalContent(key: ProviderEducationalKey) {
  return PROVIDER_EDUCATIONAL_CONTENT[key];
}

/**
 * Gets auth flow steps by key.
 */
export function getAuthFlowSteps(key: AuthFlowKey) {
  return AUTH_FLOW_STEPS[key];
}