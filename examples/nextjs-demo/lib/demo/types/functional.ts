/**
 * Functional Programming Types for Poblysh Connectors Demo
 *
 * This module provides Result and Option types to eliminate `any` and `undefined`
 * from the codebase while providing robust error handling and nullable value management.
 */

// ============================================================================
// RESULT TYPE - For operations that can fail
// ============================================================================

/**
 * Result type for operations that can either succeed with a value or fail with an error.
 * This replaces functions that return `T | undefined` or use `any` for error handling.
 */
export type Result<E, A> =
  | { readonly _tag: 'Ok'; readonly value: A }
  | { readonly _tag: 'Err'; readonly error: E }

// Result constructors
export const Ok = <A, E = never>(value: A): Result<E, A> => ({
  _tag: 'Ok',
  value
})

export const Err = <E, A = never>(error: E): Result<E, A> => ({
  _tag: 'Err',
  error
})

// Type guards
export const isOk = <E, A>(r: Result<E, A>): r is { readonly _tag: 'Ok'; readonly value: A } =>
  r._tag === 'Ok'

export const isErr = <E, A>(r: Result<E, A>): r is { readonly _tag: 'Err'; readonly error: E } =>
  r._tag === 'Err'

// Result transformations
export const map = <E, A, B>(f: (a: A) => B) => (r: Result<E, A>): Result<E, B> =>
  isOk(r) ? Ok(f(r.value)) : r

export const mapError = <E, A, B>(f: (e: E) => B) => (r: Result<E, A>): Result<B, A> =>
  isErr(r) ? Err(f(r.error)) : r

export const flatMap = <E, A, B>(f: (a: A) => Result<E, B>) =>
  (r: Result<E, A>): Result<E, B> =>
  isOk(r) ? f(r.value) : r

export const bimap = <E, A, F, B>(onError: (e: E) => F, onSuccess: (a: A) => B) =>
  (r: Result<E, A>): Result<F, B> =>
  r._tag === 'Ok' ? Ok(onSuccess(r.value)) : Err(onError(r.error))

// Result pattern matching
export const match = <E, A, B>(patterns: {
  readonly Ok: (value: A) => B
  readonly Err: (error: E) => B
}) => (r: Result<E, A>): B =>
  r._tag === 'Ok' ? patterns.Ok(r.value) : patterns.Err(r.error)

// Result utilities
export const getOrElse = <E, A>(defaultValue: A) => (r: Result<E, A>): A =>
  isOk(r) ? r.value : defaultValue

export const getOrThrow = <E, A>(r: Result<E, A>): A => {
  if (isOk(r)) return r.value
  throw new Error(`Result is Err: ${String(r.error)}`)
}

export const fromPromise = async <E, A>(promise: Promise<A>, onError: (error: unknown) => E): Promise<Result<E, A>> => {
  try {
    const value = await promise
    return Ok(value)
  } catch (error) {
    return Err(onError(error))
  }
}

// ============================================================================
// OPTION TYPE - For values that may or may not exist
// ============================================================================

/**
 * Option type for values that may or may not exist.
 * This replaces `T | undefined` and `T | null` with a type-safe alternative.
 */
export type Option<A> =
  | { readonly _tag: 'Some'; readonly value: A }
  | { readonly _tag: 'None' }

// Option constructors
export const Some = <A>(value: A): Option<A> => ({ _tag: 'Some', value })
export const None: Option<never> = { _tag: 'None' }

// Type guards
export const isSome = <A>(o: Option<A>): o is { readonly _tag: 'Some'; readonly value: A } =>
  o._tag === 'Some'

export const isNone = <A>(o: Option<A>): o is { readonly _tag: 'None' } =>
  o._tag === 'None'

// Option transformations
export const mapOption = <A, B>(f: (a: A) => B) => (o: Option<A>): Option<B> =>
  isSome(o) ? Some(f(o.value)) : o

export const flatMapOption = <A, B>(f: (a: A) => Option<B>) =>
  (o: Option<A>): Option<B> =>
  isSome(o) ? f(o.value) : o

export const filter = <A>(predicate: (a: A) => boolean) => (o: Option<A>): Option<A> =>
  isSome(o) && predicate(o.value) ? o : None

// Option pattern matching
export const matchOption = <A, B>(patterns: {
  readonly Some: (value: A) => B
  readonly None: () => B
}) => (o: Option<A>): B =>
  o._tag === 'Some' ? patterns.Some(o.value) : patterns.None()

// Option utilities
export const fromNullable = <A>(a: A | null | undefined): Option<A> =>
  a == null ? None : Some(a)

export const fromUndefined = <A>(a: A | undefined): Option<A> =>
  a === undefined ? None : Some(a)

export const fromNull = <A>(a: A | null): Option<A> =>
  a === null ? None : Some(a)

export const getOrElseOption = <A>(defaultValue: A) => (o: Option<A>): A =>
  isSome(o) ? o.value : defaultValue

export const toUndefined = <A>(o: Option<A>): A | undefined =>
  isSome(o) ? o.value : undefined

export const toNullable = <A>(o: Option<A>): A | null =>
  isSome(o) ? o.value : null

// ============================================================================
// DOMAIN-SPECIFIC TYPES
// ============================================================================

/**
 * Application-specific error types for the Poblysh Connectors demo.
 */
export type AppError =
  | { readonly _tag: 'NetworkError'; readonly message: string; readonly statusCode?: number }
  | { readonly _tag: 'ValidationError'; readonly field: string; readonly message: string }
  | { readonly _tag: 'AuthenticationError'; readonly message: string }
  | { readonly _tag: 'NotFoundError'; readonly resource: string; readonly id: string }
  | { readonly _tag: 'PermissionError'; readonly required: string; readonly message: string }
  | { readonly _tag: 'ConfigurationError'; readonly message: string }
  | { readonly _tag: 'DatabaseError'; readonly query?: string; readonly message: string }

// Error constructors
export const NetworkError = (message: string, statusCode?: number) =>
  Err<AppError, never>({ _tag: 'NetworkError', message, statusCode })

export const ValidationError = (field: string, message: string) =>
  Err<AppError, never>({ _tag: 'ValidationError', field, message })

export const AuthenticationError = (message: string) =>
  Err<AppError, never>({ _tag: 'AuthenticationError', message })

export const NotFoundError = (resource: string, id: string) =>
  Err<AppError, never>({ _tag: 'NotFoundError', resource, id })

export const PermissionError = (required: string, message: string) =>
  Err<AppError, never>({ _tag: 'PermissionError', required, message })

export const ConfigurationError = (message: string) =>
  Err<AppError, never>({ _tag: 'ConfigurationError', message })

export const DatabaseError = (query: string | undefined, message: string) =>
  Err<AppError, never>({ _tag: 'DatabaseError', query, message })

/**
 * Result type with application-specific errors.
 */
export type AppResult<A> = Result<AppError, A>

// ============================================================================
// ASYNC HELPERS
// ============================================================================

/**
 * Async Result helper for common API operations.
 */
export const asyncResult = async <A>(
  operation: () => Promise<A>,
  onError: (error: unknown) => AppError
): Promise<AppResult<A>> => {
  try {
    const result = await operation()
    return Ok(result)
  } catch (error) {
    return Err(onError(error))
  }
}

/**
 * Safe async wrapper that never throws - always returns a Result.
 */
export const safeAsync = <A>(
  operation: () => Promise<A>
): Promise<AppResult<A>> => {
  return asyncResult(operation, (error) => ({
    _tag: 'NetworkError',
    message: error instanceof Error ? error.message : 'Unknown error'
  }))
}

// ============================================================================
// EXAMPLE IMPLEMENTATIONS
// ============================================================================

/**
 * Example: Safe API client method
 */
export const safeApiCall = async <T>(
  url: string,
  options?: RequestInit
): Promise<AppResult<T>> => {
  return asyncResult(
    async () => {
      const response = await fetch(url, options)
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`)
      }
      return response.json()
    },
    (error) => ({
      _tag: 'NetworkError',
      message: error instanceof Error ? error.message : 'API call failed',
      statusCode: error instanceof Error && error.message.includes('HTTP')
        ? parseInt(error.message.split(' ')[1])
        : undefined
    })
  )
}

/**
 * Example: Safe property access
 */
export const safeGet = <T, K extends keyof T>(
  obj: T,
  key: K
): Option<T[K]> => fromNullable(obj[key])

/**
 * Example: Safe array operations
 */
export const safeHead = <T>(array: readonly T[]): Option<T> =>
  array.length > 0 ? Some(array[0]) : None

export const safeFind = <T>(
  array: readonly T[],
  predicate: (item: T) => boolean
): Option<T> => {
  const found = array.find(predicate)
  return fromNullable(found)
}

/**
 * Example: Safe JSON parsing
 */
export const safeJsonParse = (text: string): AppResult<unknown> => {
  try {
    const parsed = JSON.parse(text)
    return Ok(parsed)
  } catch (error) {
    return Err({
      _tag: 'ValidationError',
      field: 'json',
      message: error instanceof Error ? error.message : 'Invalid JSON'
    })
  }
}

/**
 * Example: Safe type guard
 */
export const safeTypeGuard = <T>(
  value: unknown,
  guard: (v: unknown) => v is T,
  errorMessage = 'Type guard failed'
): AppResult<T> => {
  return guard(value)
    ? Ok(value)
    : Err({
        _tag: 'ValidationError',
        field: 'type',
        message: errorMessage
      })
}