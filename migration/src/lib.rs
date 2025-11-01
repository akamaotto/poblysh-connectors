//! Database migrations for the Connectors API.
//!
//! This module contains all database migrations using SeaORM Migration.

pub use sea_orm_migration::prelude::*;

mod m2024_01_01_000001_create_tenants;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m2024_01_01_000001_create_tenants::Migration),
        ]
    }
}