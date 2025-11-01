# Database Migrations

This document describes the database migration system for the Connectors API.

## Overview

The Connectors API uses SeaORM migrations to manage database schema changes. Migrations are versioned and applied in order to ensure consistent database state across environments.

## Migration Structure

Migrations are located in the `migration/` directory and follow SeaORM conventions:

```
migration/
├── Cargo.toml
└── src/
    ├── lib.rs
    └── m2024_01_01_000001_create_tenants.rs
```

## Available Migrations

### m2024_01_01_000001_create_tenants

Creates the baseline `tenants` table for tenant isolation.

**Schema:**
```sql
CREATE TABLE tenants (
    id UUID PRIMARY KEY NOT NULL,
    name TEXT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

## Running Migrations

### Automatic Migrations

For `local` and `test` profiles, migrations run automatically on service startup.

### Manual Migrations

For production and other environments, use the CLI commands:

```bash
# Apply all pending migrations
cargo run -- migrate up

# Rollback the last migration
cargo run -- migrate down

# Check migration status
cargo run -- migrate status
```

## Creating New Migrations

1. Create a new migration file in `migration/src/` with the naming convention:
   `mYYYY_MM_DD_HHMMSSS_description.rs`

2. Implement the `MigrationTrait` for your migration:

```rust
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Your migration logic here
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Your rollback logic here
    }
}
```

3. Add the migration to `migration/src/lib.rs`:

```rust
pub use sea_orm_migration::prelude::*;

mod m2024_01_01_000001_create_tenants;
mod m2024_01_02_000002_create_your_table;  // Your new migration

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m2024_01_01_000001_create_tenants::Migration),
            Box::new(m2024_01_02_000002_create_your_table::Migration),  // Add your migration
        ]
    }
}
```

## Best Practices

1. **Idempotent Migrations**: Ensure migrations can be run multiple times without side effects
2. **Rollback Support**: Always implement the `down` method for rollback capability
3. **Testing**: Test migrations in isolation before deploying
4. **Backwards Compatibility**: Consider data migration when changing existing schemas
5. **Documentation**: Document complex migrations with comments

## Troubleshooting

### Migration Conflicts

If migrations get out of sync, you can:

1. Check the current status: `cargo run -- migrate status`
2. Rollback to a known good state: `cargo run -- migrate down`
3. Apply migrations manually: `cargo run -- migrate up`

### Database Connection Issues

1. Verify `POBLYSH_DATABASE_URL` is correctly set
2. Check database accessibility with `psql $POBLYSH_DATABASE_URL`
3. Review connection pool settings in configuration