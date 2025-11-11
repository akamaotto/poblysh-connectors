# TypeScript Type Safety Migration Guide

This guide provides a step-by-step approach to eliminate `any` and `undefined` from the Poblysh Connectors demo codebase using Result and Option types.

## Current State Analysis

### Areas with `any` usage:
1. **Test files** - Mock function types and global jest access
2. **Type assertions** - `(globalThis as any).jest` patterns
3. **Mock responses** - `} as any as Response` casting

### Areas with `undefined` usage:
1. **Optional fields** - Interface properties with `?:` syntax
2. **Return values** - Functions returning `T | undefined`
3. **Conditional logic** - `condition ? value : undefined` patterns
4. **Environment checks** - `typeof window !== 'undefined'`

## Migration Strategy

### Phase 1: Setup Functional Types
✅ **COMPLETED** - Created `lib/demo/types/functional.ts` with Result and Option types

### Phase 2: Refactor High-Impact Areas

#### 2.1 Configuration and State Management

**BEFORE:**
```typescript
// lib/demo/demoConfig.ts
export function getConnectorsApiBaseUrl(): string | undefined {
  const config = getDemoConfig();
  return config.mode === "real" ? config.connectorsApiBaseUrl : undefined;
}
```

**AFTER:**
```typescript
import { Option, fromNullable, Some, None } from '../types/functional';

export function getConnectorsApiBaseUrl(): Option<string> {
  const config = getDemoConfig();
  return config.mode === "real"
    ? fromNullable(config.connectorsApiBaseUrl)
    : None;
}
```

#### 2.2 Data Access Patterns

**BEFORE:**
```typescript
// lib/demo/mockData.ts
function findUser(users: DemoUser[], id: string): DemoUser | undefined {
  return users.find(user => user.id === id);
}
```

**AFTER:**
```typescript
import { Option, safeFind } from '../types/functional';

function findUser(users: DemoUser[], id: string): Option<DemoUser> {
  return safeFind(users, user => user.id === id);
}
```

#### 2.3 API Client Methods

**BEFORE:**
```typescript
// lib/demo/sharedBackendClient.ts
async function makeRequest<T>(...): Promise<DemoApiResponse<T>> {
  // ... implementation
  if (method === "DELETE" && response.status === 204) {
    return {
      data: null as T, // Uses `any` implicitly
      meta: { /* ... */ }
    };
  }
}
```

**AFTER:**
```typescript
import { AppResult, Ok, Err, asyncResult } from '../types/functional';

async function makeRequest<T>(...): Promise<AppResult<DemoApiResponse<T>>> {
  return asyncResult(
    async () => {
      // ... implementation
      if (method === "DELETE" && response.status === 204) {
        return Ok({
          data: null,
          meta: { /* ... */ }
        });
      }
      // ... rest of implementation
    },
    (error) => ({ _tag: 'NetworkError', message: String(error) })
  );
}
```

### Phase 3: Eliminate `any` from Tests

#### 3.1 Better Mock Types

**BEFORE:**
```typescript
// lib/demo/__tests__/sharedBackendClient.test.ts
(...args: any[]): any;
(globalThis as any).jest.fn() as MockFunction;
} as any as Response);
```

**AFTER:**
```typescript
import { AppResult, Ok } from '../types/functional';

// Define proper mock interfaces
interface MockResponse {
  ok: boolean;
  status: number;
  json: () => Promise<any>;
  headers: Headers;
}

interface MockFetch {
  (url: string, options?: RequestInit): Promise<MockResponse>;
}

// Type-safe mock creation
const createMockResponse = <T>(data: T, status = 200): MockResponse => ({
  ok: status >= 200 && status < 300,
  status,
  json: async () => data,
  headers: new Headers(),
});

// Usage
const mockResponse = createMockResponse({ data: 'test' }, 200);
```

### Phase 4: Replace Optional Fields

#### 4.1 Interface Refactoring

**BEFORE:**
```typescript
interface DemoConnection {
  id: string;
  displayName: string;
  status: 'connected' | 'disconnected' | 'error';
  lastSyncAt?: string;  // Optional field
  error?: string;       // Optional field
}
```

**AFTER:**
```typescript
import { Option } from '../types/functional';

interface DemoConnection {
  id: string;
  displayName: string;
  status: 'connected' | 'disconnected' | 'error';
  lastSyncAt: Option<string>;  // Explicit Option type
  error: Option<string>;       // Explicit Option type
}
```

#### 4.2 Migration Helper for Optional Fields

```typescript
// lib/demo/utils/migration.ts
import { Option, fromNullable, toNullable } from '../types/functional';

/**
 * Helper to migrate from optional fields to Option types
 */
export const migrateToOption = <T>(
  value: T | undefined | null
): Option<T> => fromNullable(value);

/**
 * Helper to migrate from Option back to optional for external APIs
 */
export const migrateFromOption = <T>(
  option: Option<T>
): T | undefined => toUndefined(option);

/**
 * Batch migration for object properties
 */
export const migrateObjectOptions = <T extends Record<string, any>>(
  obj: T,
  optionFields: (keyof T)[]
): T => {
  const migrated = { ...obj };
  for (const field of optionFields) {
    migrated[field] = migrateToOption(obj[field]);
  }
  return migrated;
};
```

### Phase 5: Environment and Platform Detection

**BEFORE:**
```typescript
// lib/demo/state.ts
if (typeof window !== 'undefined') {
  // Client-side logic
}
```

**AFTER:**
```typescript
import { Option, matchOption } from '../types/functional';

type Platform = 'browser' | 'server' | 'worker';

const getPlatform = (): Platform => {
  if (typeof window !== 'undefined') return 'browser';
  if (typeof global !== 'undefined' && global.process?.versions?.node) return 'server';
  return 'worker';
};

const withClientSide = <T>(clientSideLogic: () => T, serverSideFallback: T): T => {
  return getPlatform() === 'browser' ? clientSideLogic() : serverSideFallback;
};
```

## Implementation Examples

### Example 1: Safe Configuration Loading

```typescript
// lib/demo/config/safe-config.ts
import { AppResult, ValidationError, Ok, Err } from '../types/functional';

interface SafeConfig {
  apiBaseUrl: Option<string>;
  timeout: number;
  maxRetries: number;
}

export const loadSafeConfig = (config: unknown): AppResult<SafeConfig> => {
  if (!config || typeof config !== 'object') {
    return Err(ValidationError('config', 'Configuration must be an object'));
  }

  const cfg = config as Record<string, unknown>;

  // Required fields with validation
  const timeout = typeof cfg.timeout === 'number' ? cfg.timeout : 30000;
  const maxRetries = typeof cfg.maxRetries === 'number' ? cfg.maxRetries : 3;

  // Optional fields as Option types
  const apiBaseUrl = fromNullable(
    typeof cfg.apiBaseUrl === 'string' ? cfg.apiBaseUrl : undefined
  );

  return Ok({
    apiBaseUrl,
    timeout,
    maxRetries,
  });
};
```

### Example 2: Safe Data Processing Pipeline

```typescript
// lib/demo/processing/safe-pipeline.ts
import { AppResult, Option, flatMap, map, match, matchOption } from '../types/functional';

export const processSignalPipeline = async (
  rawSignal: unknown
): Promise<AppResult<ProcessedSignal>> => {
  return flatMap(validateSignal)(await enrichSignal(rawSignal));
};

const validateSignal = (signal: unknown): AppResult<ValidatedSignal> => {
  // Validation logic returning Result
};

const enrichSignal = async (signal: unknown): Promise<AppResult<EnrichedSignal>> => {
  // Enrichment logic returning Result
};

const processValidatedSignal = (
  signal: ValidatedSignal
): AppResult<ProcessedSignal> => {
  // Processing logic returning Result
};
```

### Example 3: Component Integration

```typescript
// lib/demo/components/safe-user-profile.tsx
import { Option, matchOption } from '../types/functional';
import { DemoUser } from '../types';

interface SafeUserProfileProps {
  user: Option<DemoUser>;
  onEdit: (user: DemoUser) => void;
}

export const SafeUserProfile: React.FC<SafeUserProfileProps> = ({
  user,
  onEdit,
}) => {
  const content = matchOption({
    Some: (userData) => (
      <div>
        <h2>{userData.name}</h2>
        <p>{userData.email}</p>
        <button onClick={() => onEdit(userData)}>Edit</button>
      </div>
    ),
    None: () => (
      <div>
        <p>User not found</p>
      </div>
    ),
  })(user);

  return <div className="user-profile">{content}</div>;
};
```

## Gradual Migration Checklist

### ✅ Completed:
- [x] Created Result and Option types
- [x] Added comprehensive examples
- [x] Identified `any` and `undefined` usage patterns

### 🔄 In Progress:
- [ ] Refactor configuration management
- [ ] Update API client methods
- [ ] Migrate test mocks

### 📋 To Do:
- [ ] Replace optional fields in interfaces
- [ ] Update environment detection
- [ ] Refactor data access patterns
- [ ] Add linting rules to prevent new `any` usage
- [ ] Update documentation
- [ ] Add migration tests

## Best Practices

### 1. Start with New Code
- Apply Result and Option patterns to new features first
- Gradually refactor existing code during maintenance

### 2. Provide Migration Helpers
- Create adapter functions for external APIs
- Use gradual migration patterns for large codebases

### 3. Update Development Tools
- Configure ESLint to forbid `any` usage
- Add TypeScript compiler options for strict checking

### 4. Team Training
- Educate team on functional programming patterns
- Provide code examples and templates
- Review pull requests for type safety

### 5. Testing Strategy
- Write tests for error paths
- Use Result types in test assertions
- Create test utilities for functional types

## Linting Configuration

Add to your `.eslintrc.js`:

```javascript
module.exports = {
  rules: {
    '@typescript-eslint/no-explicit-any': 'error',
    '@typescript-eslint/prefer-nullish-coalescing': 'error',
    '@typescript-eslint/prefer-optional-chain': 'error',
    '@typescript-eslint/no-non-null-assertion': 'error',
  },
};
```

## TypeScript Configuration

Update `tsconfig.json`:

```json
{
  "compilerOptions": {
    "strict": true,
    "noImplicitAny": true,
    "strictNullChecks": true,
    "strictFunctionTypes": true,
    "noImplicitReturns": true,
    "noImplicitThis": true
  }
}
```

This migration guide provides a comprehensive approach to eliminating `any` and `undefined` while maintaining functionality and improving type safety throughout the codebase.