# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) and other AI assistants when working with code in this repository.

<!-- OPENSPEC:START -->
# OpenSpec Instructions

These instructions are for AI assistants working in this project.

Always open `@/openspec/AGENTS.md` when the request:
- Mentions planning or proposals (words like proposal, spec, change, plan)
- Introduces new capabilities, breaking changes, architecture shifts, or big performance/security work
- Sounds ambiguous and you need the authoritative spec before coding

Use `@/openspec/AGENTS.md` to learn:
- How to create and apply change proposals
- Spec format and conventions
- Project structure and guidelines
- How backend and frontend are organized in this repo
- Which commands and package managers to prefer for each part of the stack

Keep this managed block so 'openspec update' can refresh the instructions.

<!-- OPENSPEC:END -->

## Visual Analysis for Screenshots

When working with screenshots or visual content:
- Use the visual-expert agent (`@.claude/agents/visual-expert.md`) for analyzing screenshots, extracting design requirements, layout details, component specifications, and styling information
- The visual-expert agent specializes in visual analysis and can provide detailed insights about UI patterns, design systems, spacing, colors, and component structure
- For UI component creation tasks with screenshots, the ui-component-creator agent will automatically coordinate with the visual-expert agent

## Development Commands

This repository has two main parts:

- Backend: Rust-based Poblysh Connectors API (Axum + SeaORM + PostgreSQL)
- Frontend: Next.js App Router demo in `examples/nextjs-demo` used as a mock UX sandbox

Assistants must:
- Use the Rust/Cargo commands for backend work
- Use Bun-based commands (not npm) for the Next.js demo frontend

### Backend (Rust / Cargo) Core Commands
- **Run the service**: `cargo run`
- **Run tests**: `cargo test`
- **Check code**: `cargo check`
- **Format code**: `cargo fmt`
- **Run clippy**: `cargo clippy`
- **Build for release**: `cargo build --release`

### Backend Database Operations
- **Run migrations manually**: `cargo run -- migrate up`
- **Rollback migration**: `cargo run -- migrate down`
- **Check migration status**: `cargo run -- migrate status`

### Backend: Running Specific Tests
- **Unit tests only**: `cargo test --lib`
- **Integration tests**: `cargo test --test '*'`
- **Specific test**: `cargo test test_name`

### Backend Configuration
The service uses layered `.env` files with `POBLYSH_*` environment variables. The main configuration file is `src/config/mod.rs`. See `README.md` for the complete configuration precedence and available variables.

## Architecture Overview

This repository contains:

1. **Poblysh Connectors API v0.1 (Backend)**  
   A Rust-based service for managing integrations with collaboration tools (GitHub, Jira, Google Workspace, Slack, Zoho).

2. **Next.js Demo App (Frontend)**  
   A Next.js App Router sandbox under `examples/nextjs-demo` that demonstrates mock Poblysh ↔ Connectors flows.  
   - Uses Bun as the preferred package manager and runtime.
   - Uses modern Next.js app directory patterns and client/server components.

### Backend Tech Stack
- **Language**: Rust (2024 edition) with Tokio async runtime
- **Web Framework**: Axum with utoipa for OpenAPI/Swagger documentation
- **Database**: PostgreSQL with SeaORM for data access
- **Configuration**: Layered `.env` files with `POBLYSH_*` environment variables
- **Testing**: Cargo test with testcontainers for database integration tests

### Frontend Tech Stack
- **Framework**: Next.js App Router (current major; treat as Next.js 16+ compatible)
- **Language**: TypeScript + React
- **Package Manager / Runtime**: Bun (preferred over npm/yarn)
- **Location**: `examples/nextjs-demo`
- **Usage**: Mock UX only (no real backend calls by default), aligned with OpenSpec domain model

### Backend Directory Structure
```
src/
├── main.rs              # CLI entry point (server + migrations)
├── lib.rs               # Library exports
├── server.rs            # Axum server setup and routing
├── config/              # Configuration loading
├── models/              # SeaORM entities (Provider, Connection)
├── repositories/        # Database access layer
├── handlers/            # HTTP request handlers
├── seeds/              # Database seeding utilities
└── db.rs                # Database connection pool
```

### Frontend Directory Structure (Next.js Demo)
```
examples/nextjs-demo/
├── app/                 # App Router entrypoints (pages, routes)
├── components/          # UI and demo components
├── lib/demo/            # Mock domain model, state, generators
├── public/              # Static assets
├── package.json         # Metadata (prefer Bun to run scripts)
└── bun.lock             # Indicates Bun usage is preferred
```

### Key Components

#### Data Layer
- **Provider Entity**: Global catalog of supported integrations
- **Connection Entity**: Tenant-scoped authorizations with encrypted tokens
- **SeaORM Integration**: Clean entity models with proper relationships

#### Configuration System
- **Layered Loading**: `.env` → `.env.local` → `.env.{profile}` → environment variables
- **Profile Support**: `local`, `test` with automatic migrations
- **Validation**: Type-safe configuration with error handling

#### API Layer
- **Axum Framework**: Modern async web server
- **OpenAPI Integration**: Automatic Swagger UI at `/docs`
- **State Management**: Shared database connection pool

### OpenSpec System
This project uses a specification-driven development workflow for both backend and frontend:

Before implementing new features (API or demo):
1. Check `openspec/specs/` for existing specifications.
2. Review `openspec/changes/` for pending proposals.
3. Follow the workflow in `openspec/AGENTS.md`.
4. Ensure frontend behavior (Next.js demo) remains consistent with the domain model and flows defined in OpenSpec.

### Development Patterns

Backend:
- **Layered Architecture**: API → Services → Repositories → Database
- **Error Handling**: `thiserror` for libraries, `anyhow` for application
- **Problem Details Codes**: Use screaming snake case (e.g., `INTERNAL_SERVER_ERROR`) for the `code` field in problem+json responses and specs
- **Testing**: Unit tests alongside code, integration tests in `tests/`
- **Database**: Automatic migrations for `local` and `test` profiles

Frontend (Next.js demo):
- **App Router** and React Server/Client Components
- Use `"use client"` only where needed; keep most domain/model logic in shared modules under `lib/demo/`
- Prefer Bun for running scripts (`bun install`, `bun dev`, `bun test` where applicable)
- Treat the demo as a consumer of the specified Connectors domain model; keep it aligned with OpenSpec requirements

### Important Files for Context
- `openspec/AGENTS.md` - Authoritative OpenSpec workflow, including backend/frontend guidance
- `openspec/project.md` - Project conventions
- `README.md` - Backend setup and usage instructions
- `src/main.rs` - Backend CLI commands and entry point
- `src/config/mod.rs` - Backend configuration system
- `src/models/` - SeaORM entity definitions
- `plan/nextjs-demo/` - PRD, tech specs, API plan, and UI plan for the Next.js mock UX demo
- `examples/nextjs-demo/` - Next.js App Router mock UX sandbox that demonstrates Poblysh ↔ Connectors flows (Mode A: pure mock, no real API calls; Bun preferred)

### Current Implementation Status
✅ Database foundation with SeaORM
✅ Core entities (Provider, Connection)
✅ Configuration layer
✅ Basic API with OpenAPI docs
✅ Testing framework

🔄 Connector SDK (pending)
🔄 OAuth flows (pending)
🔄 Token management (pending)
🔄 Sync engine (pending)

## TypeScript Guidelines

When working in the `examples/nextjs-demo` frontend or any TypeScript code in this repo:

### Functional Programming with Result and Option Types

**CRITICAL**: This project uses functional programming patterns to eliminate `any` and `undefined` types. All new TypeScript code MUST use the functional types from `lib/demo/types/functional.ts`.

#### Required Tools
- **Result<E, A>**: Type-safe error handling that replaces exceptions and `any` types
- **Option<T>**: Null-safe value handling that replaces `T | undefined` and `T | null`
- **AppError**: Domain-specific error types for consistent error handling

#### Import Required Types
```typescript
import {
  Result, Option, AppResult, AppError,
  Ok, Err, Some, None,
  map, flatMap, match, matchOption,
  fromNullable, asyncResult,
  NetworkError, ValidationError, AuthenticationError
} from './lib/demo/types/functional';
```

#### Mandatory Patterns

**1. Replace Functions Returning undefined**
```typescript
// ❌ FORBIDDEN
function findUser(id: string): User | undefined {
  return users.find(u => u.id === id);
}

// ✅ REQUIRED
function findUser(id: string): Option<User> {
  return fromNullable(users.find(u => u.id === id));
}
```

**2. Replace Functions That Throw Exceptions**
```typescript
// ❌ FORBIDDEN
async function fetchUser(id: string): Promise<User> {
  const response = await fetch(`/api/users/${id}`);
  if (!response.ok) throw new Error(`HTTP ${response.status}`);
  return response.json();
}

// ✅ REQUIRED
async function fetchUser(id: string): Promise<AppResult<User>> {
  return asyncResult(
    async () => {
      const response = await fetch(`/api/users/${id}`);
      if (!response.ok) {
        return Err(NetworkError(`HTTP ${response.status}`, response.status));
      }
      return await response.json();
    },
    (error) => NetworkError(error.message)
  );
}
```

**3. Replace Optional Interface Fields**
```typescript
// ❌ FORBIDDEN
interface DemoConnection {
  id: string;
  status: 'connected' | 'disconnected';
  lastSyncAt?: string;  // Optional field
  error?: string;       // Optional field
}

// ✅ REQUIRED
interface DemoConnection {
  id: string;
  status: 'connected' | 'disconnected';
  lastSyncAt: Option<string>;  // Explicit Option type
  error: Option<string>;       // Explicit Option type
}
```

**4. Use Pattern Matching for Consumption**
```typescript
// ✅ REQUIRED for consuming Option types
const message = matchOption({
  Some: (user) => `Found user: ${user.name}`,
  None: () => 'User not found'
})(findUser('123'));

// ✅ REQUIRED for consuming Result types
const result = await fetchUser('123');
const message = match({
  Ok: (user) => `Successfully loaded ${user.name}`,
  Err: (error) => `Failed to load user: ${error.message}`
})(result);
```

#### Strict Prohibitions

**Absolutely Forbidden in New Code:**
- ❌ `any` type - NEVER use under any circumstances
- ❌ `T | undefined` unions - Use `Option<T>` instead
- ❌ `T | null` unions - Use `Option<T>` instead
- ❌ Throwing exceptions - Use `Result<Error, T>` instead
- ❌ `@ts-ignore` - Fix types properly
- ❌ `as any` - Use proper type guards or Result/Option

**Allowed Only in Migration/Adapter Code:**
- `as any` only when wrapping external libraries that cannot be changed
- `T | undefined` only when implementing external API contracts

#### Migration Strategy for Existing Code

When refactoring existing code that uses `any` or `undefined`:

1. **Start with New Code**: Apply functional patterns to all new features
2. **Gradual Migration**: Refactor existing code during maintenance
3. **Adapter Pattern**: Wrap existing code to provide functional interfaces
4. **Test Coverage**: Ensure all error paths are tested with Result types

#### Available Helper Functions

```typescript
// Safe property access
const safeGet = <T, K extends keyof T>(obj: T, key: K): Option<T[K]>

// Safe array operations
const safeHead = <T>(array: readonly T[]): Option<T>
const safeFind = <T>(array: readonly T[], predicate: (item: T) => boolean): Option<T>

// Safe async operations
const safeAsync = <T>(operation: () => Promise<T>): Promise<AppResult<T>>
const asyncResult = <T>(operation: () => Promise<T>, onError: (error: unknown) => AppError): Promise<AppResult<T>>

// Type utilities
const fromNullable = <T>(value: T | null | undefined): Option<T>
const toUndefined = <T>(option: Option<T>): T | undefined
```

#### Documentation References

- **Complete API**: `lib/demo/types/functional.ts`
- **Usage Examples**: `lib/demo/examples/functional-integration.ts`
- **Migration Guide**: `lib/demo/migration-guide.md`
- **Refactored Example**: `lib/demo/refactored/sharedBackendClient.safe.ts`

#### Quality Enforcement

All code reviews must verify:
1. No `any` types in new code
2. No `T | undefined` unions in new interfaces
3. Proper use of Result types for async operations
4. Proper use of Option types for nullable values
5. Complete pattern matching for all Result/Option consumption

### Legacy TypeScript Support

For existing code that hasn't been migrated yet:
- Document the technical debt
- Create migration tickets
- Prioritize migration based on usage frequency and criticality
- Use adapter patterns to provide functional interfaces to legacy code