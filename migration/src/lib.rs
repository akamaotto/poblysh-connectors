//! Database migrations for the Connectors API.
//!
//! This module contains all database migrations using SeaORM Migration.

pub use sea_orm_migration::prelude::*;

mod m2024_01_01_000001_create_tenants;
mod m2025_11_01_102700_create_providers;
mod m2025_11_01_102800_create_connections;
mod m2025_11_01_103000_create_signals;
mod m2025_11_01_103100_create_sync_jobs;
mod m2025_11_02_120000_create_oauth_states;
mod m2025_11_03_000100_add_sync_job_unique_interval_guard;
mod m2025_11_07_120000_create_grounded_signals;
mod m2025_11_07_120100_create_tenant_signal_configs;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m2024_01_01_000001_create_tenants::Migration),
            Box::new(m2025_11_01_102700_create_providers::Migration),
            Box::new(m2025_11_01_102800_create_connections::Migration),
            Box::new(m2025_11_01_103000_create_signals::Migration),
            Box::new(m2025_11_01_103100_create_sync_jobs::Migration),
            Box::new(m2025_11_02_120000_create_oauth_states::Migration),
            Box::new(m2025_11_03_000100_add_sync_job_unique_interval_guard::Migration),
            Box::new(m2025_11_07_120000_create_grounded_signals::Migration),
            Box::new(m2025_11_07_120100_create_tenant_signal_configs::Migration),
        ]
    }
}
