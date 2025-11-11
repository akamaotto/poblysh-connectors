/**
 * Functional Integration Examples
 *
 * This file demonstrates how to integrate Result and Option types into the existing
 * Poblysh Connectors demo codebase to eliminate `any` and `undefined` usage.
 */

import {
  Result,
  Option,
  AppResult,
  AppError,
  Ok,
  Err,
  Some,
  None,
  isOk,
  isErr,
  isSome,
  isNone,
  flatMap,
  matchOption,
  fromNullable,
  fromPromise,
  safeApiCall,
  safeJsonParse,
  safeFind,
  safeHead,
  NetworkError,
  ValidationError,
  // For Option transformations
  mapOption,
  flatMapOption,
  // Educational imports - kept for documentation purposes
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  map,
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  match,
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  NotFoundError,
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  mapError,
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  getOrElse,
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  safeGet,
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  asyncResult,
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  AuthenticationError,
} from '../types/functional'

import {
  DemoUser,
  DemoConnection,
  DemoProvider,
  DemoSignal,
  DemoApiResponse,
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  DemoApiError,
} from '../types'

// ============================================================================
// EXAMPLE 1: Refactoring API Client Methods
// ============================================================================

/**
 * BEFORE: Method that returns undefined on error
 * Kept for educational comparison purposes
 */
// eslint-disable-next-line @typescript-eslint/no-unused-vars
function findUserOld(users: DemoUser[], id: string): DemoUser | undefined {
  return users.find(user => user.id === id)
}

/**
 * AFTER: Method that returns Option<DemoUser>
 */
function findUser(users: DemoUser[], id: string): Option<DemoUser> {
  return fromNullable(users.find(user => user.id === id))
}

/**
 * BEFORE: Method that throws exceptions
 * Kept for educational comparison purposes
 */
// eslint-disable-next-line @typescript-eslint/no-unused-vars
async function fetchConnectionsOld(): Promise<DemoConnection[]> {
  const response = await fetch('/api/connections')
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}`)
  }
  return response.json()
}

/**
 * AFTER: Method that returns AppResult<DemoConnection[]>
 */
async function fetchConnections(): Promise<AppResult<DemoConnection[]>> {
  return safeApiCall<DemoConnection[]>('/api/connections')
}

// ============================================================================
// EXAMPLE 2: Safe Data Transformation
// ============================================================================

/**
 * Example: Transform user data safely
 */
// eslint-disable-next-line @typescript-eslint/no-unused-vars
const transformUserData = (user: DemoUser): Option<{ name: string; email: string }> => {
  return mapOption(
    (u: DemoUser) => ({
      name: u.name,
      email: u.email
    })
  )(fromNullable(user))
}

/**
 * Example: Validate and process connection data
 */
const validateConnection = (connection: Partial<DemoConnection>): AppResult<DemoConnection> => {
  // Validate required fields
  if (!connection.id) {
    return ValidationError('id', 'Connection ID is required')
  }
  if (!connection.providerSlug) {
    return ValidationError('providerSlug', 'Provider slug is required')
  }
  if (!connection.tenantId) {
    return ValidationError('tenantId', 'Tenant ID is required')
  }

  // Create valid connection object
  return Ok({
    id: connection.id,
    tenantId: connection.tenantId,
    providerSlug: connection.providerSlug,
    displayName: connection.displayName || 'Unknown Connection',
    status: connection.status || 'disconnected',
    createdAt: connection.createdAt || new Date().toISOString(),
    lastSyncAt: connection.lastSyncAt,
    error: connection.error,
  } as DemoConnection)
}

// ============================================================================
// EXAMPLE 3: Composing Operations
// ============================================================================

/**
 * Example: Find user and validate their connections
 */
const getUserWithConnections = (
  users: DemoUser[],
  connections: DemoConnection[],
  userId: string
): Option<{ user: DemoUser; activeConnections: DemoConnection[] }> => {
  return flatMapOption((user: DemoUser) => {
    const userConnections = connections.filter(
      conn => conn.tenantId === user.tenantId && conn.status === 'connected'
    )

    return userConnections.length > 0
      ? Some({ user, activeConnections: userConnections })
      : None
  })(findUser(users, userId))
}

/**
 * Example: Process signals with error handling
 */
// eslint-disable-next-line @typescript-eslint/no-unused-vars
const processSignalData = (rawSignal: unknown): AppResult<DemoSignal> => {
  return flatMap((parsed: unknown) => {
    // Basic type validation
    if (!parsed || typeof parsed !== 'object') {
      return ValidationError('signal', 'Invalid signal object')
    }

    const signal = parsed as Record<string, unknown>

    // Validate required fields
    if (!signal.id || typeof signal.id !== 'string') {
      return ValidationError('id', 'Signal ID is required and must be a string')
    }
    if (!signal.kind || typeof signal.kind !== 'string') {
      return ValidationError('kind', 'Signal kind is required and must be a string')
    }
    if (!signal.title || typeof signal.title !== 'string') {
      return ValidationError('title', 'Signal title is required and must be a string')
    }

    // Create valid signal object
    return Ok({
      id: signal.id,
      tenantId: signal.tenantId as string || 'default-tenant',
      providerSlug: signal.providerSlug as string || 'unknown',
      connectionId: signal.connectionId as string || 'unknown',
      kind: signal.kind,
      title: signal.title,
      summary: signal.summary as string || '',
      author: signal.author as string || 'unknown',
      occurredAt: signal.occurredAt as string || new Date().toISOString(),
      discoveredAt: signal.discoveredAt as string || new Date().toISOString(),
      metadata: signal.metadata as Record<string, unknown> || {},
      url: signal.url as string,
      relevanceScore: signal.relevanceScore as number,
      rawPayload: signal.rawPayload as Record<string, unknown> || {},
      processingDetails: signal.processingDetails as {
        fetchTime: number
        processingTime: number
        retryCount: number
        lastRetryAt?: string
      } || {
        fetchTime: 0,
        processingTime: 0,
        retryCount: 0,
      },
      relatedSignals: signal.relatedSignals as string[] || [],
      parentSignalId: signal.parentSignalId as string,
      childSignalIds: signal.childSignalIds as string[] || [],
      categories: signal.categories as string[] || [],
      sentiment: signal.sentiment as 'positive' | 'negative' | 'neutral',
      urgency: signal.urgency as 'low' | 'medium' | 'high' | 'critical' || 'medium',
      impact: signal.impact as {
        scope: 'team' | 'project' | 'organization' | 'public'
        affectedUsers?: number
        estimatedCost?: number
      } || {
        scope: 'team' as const,
      },
      environment: signal.environment as 'production' | 'staging' | 'development' || 'development',
    })
  })(safeJsonParse(JSON.stringify(rawSignal)))
}

// ============================================================================
// EXAMPLE 4: API Response Handling
// ============================================================================

/**
 * Example: Handle API responses safely
 */
// eslint-disable-next-line @typescript-eslint/no-unused-vars
const handleApiResponse = <T>(response: DemoApiResponse<T>): AppResult<T> => {
  try {
    // Check if response has valid data
    if (response.data === null || response.data === undefined) {
      return ValidationError('data', 'API response missing data')
    }

    return Ok(response.data)
  } catch (error) {
    return NetworkError(
      `Failed to process API response: ${error instanceof Error ? error.message : 'Unknown error'}`
    )
  }
}

/**
 * Example: Fetch and process provider data
 */
// eslint-disable-next-line @typescript-eslint/no-unused-vars
const fetchAndProcessProvider = async (providerSlug: string): Promise<AppResult<DemoProvider>> => {
  const result = await safeApiCall<DemoProvider[]>('/api/providers')
  if (isErr(result)) {
    return result
  }

  const providers = result.value
  const provider = safeFind(providers, p => p.slug === providerSlug)
  return matchOption({
    Some: (p: DemoProvider) => Ok(p),
    None: () => Err<AppError, DemoProvider>({ _tag: 'NotFoundError', resource: 'provider', id: providerSlug })
  })(provider)
}

// ============================================================================
// EXAMPLE 5: Error Recovery Patterns
// ============================================================================

/**
 * Example: Retry with fallback
 */
// eslint-disable-next-line @typescript-eslint/no-unused-vars
const fetchWithFallback = async (
  primaryUrl: string,
  fallbackUrl: string
): Promise<AppResult<unknown>> => {
  const result = await safeApiCall(primaryUrl)

  if (isOk(result)) {
    return result
  }

  console.warn(`Primary endpoint failed: ${getErrorMessage(result.error)}, trying fallback`)
  return await safeApiCall(fallbackUrl)
}

/**
 * Helper function to extract error message from AppError
 */
const getErrorMessage = (error: AppError): string => {
  switch (error._tag) {
    case 'NetworkError':
      return error.message
    case 'ValidationError':
      return error.message
    case 'AuthenticationError':
      return error.message
    case 'NotFoundError':
      return `${error.resource} with id '${error.id}' not found`
    case 'PermissionError':
      return error.message
    case 'ConfigurationError':
      return error.message
    case 'DatabaseError':
      return error.message
    default:
      return 'Unknown error'
  }
}

/**
 * Example: Graceful degradation
 */
// eslint-disable-next-line @typescript-eslint/no-unused-vars
const getUserProfile = async (
  userId: string
): Promise<{ user?: DemoUser; error?: string }> => {
  const result = await safeApiCall<DemoUser>(`/api/users/${userId}`)

  if (isOk(result)) {
    return { user: result.value }
  }

  return {
    user: undefined,
    error: result.error._tag === 'NotFoundError'
      ? 'User not found'
      : getErrorMessage(result.error)
  }
}

// ============================================================================
// EXAMPLE 6: Working with Arrays and Collections
// ============================================================================

/**
 * Example: Safe array operations
 */
// eslint-disable-next-line @typescript-eslint/no-unused-vars
const getFirstActiveConnection = (connections: DemoConnection[]): Option<DemoConnection> => {
  return flatMapOption((conn: DemoConnection) =>
    conn.status === 'connected' ? Some(conn) : None
  )(safeHead(connections))
}

/**
 * Example: Process all items safely, collecting errors
 */
// eslint-disable-next-line @typescript-eslint/no-unused-vars
const processAllConnections = (
  connections: DemoConnection[]
): { successful: DemoConnection[]; errors: AppError[] } => {
  return connections.reduce(
    (acc, connection) => {
      const result = validateConnection(connection)

      if (isOk(result)) {
        return {
          successful: [...acc.successful, result.value],
          errors: acc.errors
        }
      } else {
        return {
          successful: acc.successful,
          errors: [...acc.errors, result.error]
        }
      }
    },
    { successful: [] as DemoConnection[], errors: [] as AppError[] }
  )
}

// ============================================================================
// EXAMPLE 7: Configuration and Validation
// ============================================================================

/**
 * Example: Safe configuration loading
 */
// eslint-disable-next-line @typescript-eslint/no-unused-vars
const validateDemoConfig = (config: unknown): AppResult<{
  signalFrequency: 'low' | 'medium' | 'high'
  errorRate: '0%' | '10%' | '20%'
  timingMode: 'fast' | 'realistic'
}> => {
  if (!config || typeof config !== 'object') {
    return ValidationError('config', 'Configuration must be an object')
  }

  const cfg = config as Record<string, unknown>

  const signalFrequency =
    cfg.signalFrequency === 'low' || cfg.signalFrequency === 'medium' || cfg.signalFrequency === 'high'
      ? cfg.signalFrequency
      : 'medium'

  const errorRate =
    cfg.errorRate === '0%' || cfg.errorRate === '10%' || cfg.errorRate === '20%'
      ? cfg.errorRate
      : '0%'

  const timingMode =
    cfg.timingMode === 'fast' || cfg.timingMode === 'realistic'
      ? cfg.timingMode
      : 'realistic'

  return Ok({
    signalFrequency,
    errorRate,
    timingMode
  })
}

// ============================================================================
// EXAMPLE 8: Migration Patterns
// ============================================================================

/**
 * Example: Wrapping existing functions for gradual migration
 */
const wrapAsyncFunction = <T, E>(
  asyncFn: () => Promise<T>,
  errorTransformer: (error: unknown) => E
): Promise<Result<E, T>> => {
  return fromPromise(asyncFn(), errorTransformer)
}

/**
 * Example: Adapter for existing API client
 */
// eslint-disable-next-line @typescript-eslint/no-unused-vars
const adaptApiClient = {
  getUsers: () => wrapAsyncFunction(
    () => fetch('/api/users').then(r => r.json()),
    (error) => ({ _tag: 'NetworkError', message: String(error) } as AppError)
  ),

  getConnection: (id: string) => wrapAsyncFunction(
    () => fetch(`/api/connections/${id}`).then(r => r.json()),
    (error) => ({ _tag: 'NetworkError', message: String(error) } as AppError)
  ),
}

// ============================================================================
// EXAMPLE 9: Testing Helpers
// ============================================================================

/**
 * Example: Test utilities for Result types
 */
export const testUtils = {
  /** Create a test user with optional properties */
  createTestUser: (overrides: Partial<DemoUser> = {}): DemoUser => ({
    id: 'test-user-id',
    email: 'test@example.com',
    name: 'Test User',
    avatarUrl: undefined,
    roles: ['user'],
    tenantId: 'test-tenant',
    ...overrides
  }),

  /** Create a test connection with validation */
  createTestConnection: (overrides: Partial<DemoConnection> = {}): AppResult<DemoConnection> =>
    validateConnection({
      id: 'test-connection-id',
      tenantId: 'test-tenant',
      providerSlug: 'github',
      displayName: 'Test Connection',
      status: 'connected',
      createdAt: new Date().toISOString(),
      ...overrides
    }),

  /** Assert Result is Ok */
  expectOk: <T>(result: AppResult<T>): T => {
    if (isOk(result)) return result.value
    throw new Error(`Expected Ok but got Err: ${getErrorMessage(result.error)}`)
  },

  /** Assert Result is Err */
  expectErr: <T>(result: AppResult<T>): AppError => {
    if (isErr(result)) return result.error
    throw new Error('Expected Err but got Ok')
  },

  /** Assert Option is Some */
  expectSome: <T>(option: Option<T>): T => {
    if (isSome(option)) return option.value
    throw new Error('Expected Some but got None')
  },

  /** Assert Option is None */
  expectNone: <T>(option: Option<T>): void => {
    if (isNone(option)) return
    throw new Error('Expected None but got Some')
  },
}

// ============================================================================
// USAGE EXAMPLES
// ============================================================================

/**
 * Example usage in a component or service
 */
export const exampleUsage = async () => {
  // Example 1: Safe user lookup
  const users = [testUtils.createTestUser()]
  const userOption = findUser(users, 'test-user-id')
  const userName = matchOption({
    Some: (user: DemoUser) => user.name,
    None: () => 'Guest'
  })(userOption)

  // Example 2: Safe API call
  const connectionsResult = await fetchConnections()
  const connections = isOk(connectionsResult) ? connectionsResult.value : []
  if (isErr(connectionsResult)) {
    console.error('Failed to fetch connections:', getErrorMessage(connectionsResult.error))
  }

  // Example 3: Data validation
  const testConnection = testUtils.expectOk(
    testUtils.createTestConnection({ providerSlug: 'github' })
  )

  // Example 4: Composition (create mock connections for the example)
  const mockConnections: DemoConnection[] = [
    testUtils.expectOk(
      testUtils.createTestConnection({
        providerSlug: 'github',
        tenantId: 'test-tenant',
        status: 'connected'
      })
    )
  ]

  const userWithConnections = getUserWithConnections(users, mockConnections, 'test-user-id')
  const profile = matchOption({
    Some: (data: { user: DemoUser; activeConnections: DemoConnection[] }) =>
      `${data.user.name} has ${data.activeConnections.length} connections`,
    None: () => 'User not found or no active connections'
  })(userWithConnections)

  console.log({ userName, connectionsCount: connections.length, testConnection, profile })
}