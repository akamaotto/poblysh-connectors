# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

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

Keep this managed block so 'openspec update' can refresh the instructions.

<!-- OPENSPEC:END -->

## Development Commands

### Core Commands
- **Run the service**: `cargo run`
- **Run tests**: `cargo test`
- **Check code**: `cargo check`
- **Format code**: `cargo fmt`
- **Run clippy**: `cargo clippy`
- **Build for release**: `cargo build --release`

### Database Operations
- **Run migrations manually**: `cargo run -- migrate up`
- **Rollback migration**: `cargo run -- migrate down`
- **Check migration status**: `cargo run -- migrate status`

### Running Specific Tests
- **Unit tests only**: `cargo test --lib`
- **Integration tests**: `cargo test --test '*'`
- **Specific test**: `cargo test test_name`

### Configuration
The service uses layered `.env` files with `POBLYSH_*` environment variables. The main configuration file is `src/config/mod.rs`. See `README.md` for the complete configuration precedence and available variables.

## Architecture Overview

This is the **Poblysh Connectors API v0.1**, a Rust-based service for managing integrations with collaboration tools (GitHub, Jira, Google Workspace, Slack, Zoho).

### Tech Stack
- **Language**: Rust (2024 edition) with Tokio async runtime
- **Web Framework**: Axum with utoipa for OpenAPI/Swagger documentation
- **Database**: PostgreSQL with SeaORM for data access
- **Configuration**: Layered `.env` files with `POBLYSH_*` environment variables
- **Testing**: Cargo test with testcontainers for database integration tests

### Directory Structure
```
src/
├── main.rs              # CLI entry point (server + migrations)
├── lib.rs               # Library exports
├── server.rs            # Axum server setup and routing
├── config/              # Configuration loading
├── models/              # SeaORM entities (Provider, Connection)
├── repositories/        # Database access layer
├── handlers/           # HTTP request handlers
├── seeds/              # Database seeding utilities
└── db.rs               # Database connection pool
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
This project uses a specification-driven development workflow. Before implementing new features:
1. Check `openspec/specs/` for existing specifications
2. Review `openspec/changes/` for pending proposals
3. Follow the workflow in `openspec/AGENTS.md`

### Development Patterns
- **Layered Architecture**: API → Services → Repositories → Database
- **Error Handling**: `thiserror` for libraries, `anyhow` for application
- **Problem Details Codes**: Use screaming snake case (e.g., `INTERNAL_SERVER_ERROR`) for the `code` field in problem+json responses and specs
- **Testing**: Unit tests alongside code, integration tests in `tests/`
- **Database**: Automatic migrations for `local` and `test` profiles

### Important Files for Context
- `openspec/AGENTS.md` - Development workflow and spec-driven development
- `openspec/project.md` - Project conventions
- `README.md` - Setup and usage instructions
- `src/main.rs` - CLI commands and entry point
- `src/config/mod.rs` - Configuration system
- `src/models/` - SeaORM entity definitions
- `plan/nextjs-demo/` - PRD, tech specs, API plan, and UI plan for the Next.js mock UX demo
- `examples/nextjs-demo/` - Next.js App Router mock UX sandbox that demonstrates Poblysh ↔ Connectors flows (Mode A: pure mock, no real API calls)

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