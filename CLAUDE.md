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