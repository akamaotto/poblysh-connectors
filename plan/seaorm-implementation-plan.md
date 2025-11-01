# SeaORM Setup and Migrations Implementation Plan

## Overview
This document outlines the implementation plan for adding SeaORM setup and migrations to the Connectors API project, establishing a Postgres backbone for tenant isolation and future entity management.

## Architecture Decisions

### Database Layer
- **ORM**: SeaORM with async support and Postgres backend
- **Connection Pool**: Configurable max connections (default: 10) and acquire timeout (default: 5s)
- **Migration Strategy**: SeaORM Migrator with automatic execution for local/test profiles
- **UUID Generation**: Application-side UUID v4 for portability

### Migration Structure
- **Location**: Dedicated `migration/` directory following SeaORM conventions
- **Baseline Schema**: `tenants` table with UUID primary key and timestamps
- **CLI Integration**: Built into main binary using clap (`cargo run -- migrate <command>`)

## Implementation Steps

### 1. Dependencies and Configuration
- Add SeaORM dependencies with Postgres features
- Extend `AppConfig` with database configuration fields
- Update configuration loading to handle new `POBLYSH_*` environment variables

### 2. Database Connection Module
- Create `src/db.rs` with connection pool initialization
- Implement retry logic with exponential backoff
- Add comprehensive error handling for database operations

### 3. Migration System
- Set up SeaORM migration structure in `migration/` directory
- Create initial migration for `tenants` table
- Implement migrator with up/down/status operations

### 4. Application Integration
- Update `main.rs` to initialize database and handle CLI commands
- Modify `server.rs` to include database connection in application state
- Add automatic migration execution for local/test profiles

### 5. Testing and Documentation
- Create integration tests using testcontainers
- Update project documentation with setup instructions
- Add migration workflow documentation

## Database Schema

### Tenants Table
```sql
CREATE TABLE tenants (
    id UUID PRIMARY KEY NOT NULL,
    name TEXT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `POBLYSH_DATABASE_URL` | Postgres connection string | Required |
| `POBLYSH_DB_MAX_CONNECTIONS` | Maximum pool connections | 10 |
| `POBLYSH_DB_ACQUIRE_TIMEOUT_MS` | Connection acquire timeout | 5000 |

## CLI Commands

```bash
# Run migrations up
cargo run -- migrate up

# Rollback last migration
cargo run -- migrate down

# Check migration status
cargo run -- migrate status
```

## Testing Strategy

- Unit tests for database connection logic
- Integration tests with testcontainers for migration verification
- Schema validation tests to ensure correct table structure

## Future Considerations

- Migration to separate migration crate if complexity grows
- Per-tenant schema evaluation (currently using single schema with tenant_id FK)
- Connection pool tuning based on production workload
- Database health check endpoints