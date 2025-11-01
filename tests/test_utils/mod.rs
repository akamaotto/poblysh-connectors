//! Test utilities for database testing.
//!
//! This module provides utilities for setting up in-memory SQLite databases
//! with migrations for testing purposes.

use anyhow::Result;
use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, Statement};
use std::sync::Arc;
use uuid::Uuid;

/// Sets up an in-memory SQLite database with all migrations applied.
///
/// # Returns
///
/// Returns a Result containing the database connection
pub async fn setup_test_db() -> Result<DatabaseConnection> {
    // Create in-memory SQLite database
    let db = Database::connect("sqlite::memory:").await?;

    // Run all migrations
    Migrator::up(&db, None).await?;

    // SQLite does not enforce our Postgres foreign key semantics; disable FK checks to
    // allow inserting fixture data that may not satisfy cross-table relations in tests.
    use sea_orm::Statement;
    db.execute(Statement::from_string(
        db.get_database_backend(),
        "PRAGMA foreign_keys = OFF".to_string(),
    ))
    .await?;

    Ok(db)
}

/// Sets up an in-memory SQLite database with all migrations applied and returns an Arc.
///
/// # Returns
///
/// Returns a Result containing an Arc-wrapped database connection
#[allow(dead_code)]
pub async fn setup_test_db_arc() -> Result<Arc<DatabaseConnection>> {
    let db = setup_test_db().await?;
    Ok(Arc::new(db))
}

/// Creates a test tenant in the database.
///
/// # Arguments
///
/// * `db` - Database connection
/// * `tenant_id` - Optional tenant ID (will generate a new UUID if None)
///
/// # Returns
///
/// Returns a Result containing the tenant ID
#[allow(dead_code)]
pub async fn create_test_tenant(
    db: &DatabaseConnection,
    tenant_id: Option<uuid::Uuid>,
) -> Result<uuid::Uuid> {
    let id = tenant_id.unwrap_or_else(Uuid::new_v4);

    let stmt = Statement::from_string(
        db.get_database_backend(),
        format!(
            "INSERT INTO tenants (id, name) VALUES ('{}', 'Test Tenant')",
            id
        ),
    );

    db.execute(stmt).await?;

    Ok(id)
}

/// Inserts a provider row directly for testing.
#[allow(dead_code)]
pub async fn insert_provider(
    db: &DatabaseConnection,
    slug: &str,
    display_name: &str,
    auth_type: &str,
) -> Result<()> {
    let stmt = Statement::from_string(
        db.get_database_backend(),
        format!(
            "INSERT INTO providers (slug, display_name, auth_type, created_at, updated_at) VALUES ('{}', '{}', '{}', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
            slug, display_name, auth_type
        ),
    );
    db.execute(stmt).await?;
    Ok(())
}

/// Inserts a connection row directly for testing.
#[allow(dead_code)]
pub async fn insert_connection(
    db: &DatabaseConnection,
    id: Uuid,
    tenant_id: Uuid,
    provider_slug: &str,
    external_id: &str,
) -> Result<()> {
    let stmt = Statement::from_string(
        db.get_database_backend(),
        format!(
            "INSERT INTO connections (id, tenant_id, provider_slug, external_id, status, display_name, access_token_ciphertext, refresh_token_ciphertext, expires_at, scopes, metadata, created_at, updated_at) VALUES ('{}', '{}', '{}', '{}', 'active', '{} connection', NULL, NULL, NULL, '[]', '{{}}', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
            id, tenant_id, provider_slug, external_id, provider_slug
        ),
    );
    db.execute(stmt).await?;
    Ok(())
}
