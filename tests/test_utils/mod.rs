//! Test utilities for database testing.
//!
//! This module provides utilities for setting up in-memory SQLite databases
//! with migrations for testing purposes.

use anyhow::Result;
use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, EntityTrait, Statement};
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
    use sea_orm::Value;
    let backend = db.get_database_backend();
    let stmt = Statement::from_sql_and_values(
        backend,
        "INSERT INTO connections (
            id, tenant_id, provider_slug, external_id, status, display_name,
            access_token_ciphertext, refresh_token_ciphertext, expires_at, scopes, metadata,
            created_at, updated_at
        ) VALUES (?, ?, ?, ?, 'active', ?, NULL, NULL, NULL, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
        vec![
            Value::Uuid(Some(Box::new(id))),
            Value::Uuid(Some(Box::new(tenant_id))),
            Value::String(Some(Box::new(provider_slug.to_string()))),
            Value::String(Some(Box::new(external_id.to_string()))),
            Value::String(Some(Box::new(format!("{} connection", provider_slug)))),
            Value::Json(Some(Box::new(serde_json::json!([])))),
            Value::Json(Some(Box::new(serde_json::json!({})))),
        ],
    );
    db.execute(stmt).await?;
    Ok(())
}

/// Creates a test connection model for testing.
pub async fn create_test_connection() -> Result<connectors::models::connection::Model> {
    let db = setup_test_db().await?;
    let tenant_id = create_test_tenant(&db, None).await?;
    let connection_id = Uuid::new_v4();
    insert_provider(&db, "test", "Test Provider", "oauth2").await?;
    insert_connection(&db, connection_id, tenant_id, "test", "test-external-id").await?;

    let connection = connectors::models::connection::Entity::find_by_id(connection_id)
        .one(&db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Failed to find created connection"))?;

    Ok(connection)
}

/// Creates a test connection with specific ID for testing.
pub async fn create_test_connection_with_id(
    db: &DatabaseConnection,
    tenant_id: Uuid,
    connection_id: Uuid,
) -> Result<connectors::models::connection::Model> {
    insert_provider(db, "test", "Test Provider", "oauth2").await?;
    insert_connection(db, connection_id, tenant_id, "test", "test-external-id").await?;

    let connection = connectors::models::connection::Entity::find_by_id(connection_id)
        .one(db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Failed to find created connection"))?;

    Ok(connection)
}

/// Creates a test connection with specific provider for testing.
pub async fn create_test_connection_with_provider(
    db: &DatabaseConnection,
    tenant_id: Uuid,
    connection_id: Uuid,
    provider_slug: &str,
) -> Result<connectors::models::connection::Model> {
    insert_provider(
        db,
        provider_slug,
        &format!("{} Provider", provider_slug),
        "oauth2",
    )
    .await?;
    insert_connection(
        db,
        connection_id,
        tenant_id,
        provider_slug,
        "test-external-id",
    )
    .await?;

    let connection = connectors::models::connection::Entity::find_by_id(connection_id)
        .one(db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Failed to find created connection"))?;

    Ok(connection)
}

/// Creates a test sync job in the database.
pub async fn create_test_sync_job(
    db: &DatabaseConnection,
    tenant_id: Uuid,
    connection_id: Uuid,
    provider_slug: &str,
    status: &str,
    retry_after: Option<chrono::DateTime<chrono::Utc>>,
    priority: i16,
) -> Result<connectors::models::sync_job::Model> {
    let job_id = Uuid::new_v4();
    let now = chrono::Utc::now();

    let retry_after_str = retry_after
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or("NULL".to_string());

    let stmt = Statement::from_string(
        db.get_database_backend(),
        format!(
            r#"
            INSERT INTO sync_jobs (
                id, tenant_id, provider_slug, connection_id, job_type, status,
                priority, attempts, scheduled_at, retry_after, started_at, finished_at,
                cursor, error, created_at, updated_at
            ) VALUES (
                '{}', '{}', '{}', '{}', 'full', '{}', {}, 1, '{}', {},
                NULL, NULL, NULL, NULL, '{}', '{}'
            )
            "#,
            job_id,
            tenant_id,
            provider_slug,
            connection_id,
            status,
            priority,
            now.format("%Y-%m-%d %H:%M:%S"),
            if retry_after.is_some() {
                format!("'{}'", retry_after_str)
            } else {
                "NULL".to_string()
            },
            now.format("%Y-%m-%d %H:%M:%S"),
            now.format("%Y-%m-%d %H:%M:%S")
        ),
    );

    db.execute(stmt).await?;

    let job = connectors::models::sync_job::Entity::find_by_id(job_id)
        .one(db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Failed to find created job"))?;

    Ok(job)
}

/// Creates a test sync job with specific attempt count in the database.
pub async fn create_test_sync_job_with_attempts(
    db: &DatabaseConnection,
    tenant_id: Uuid,
    connection_id: Uuid,
    provider_slug: &str,
    status: &str,
    retry_after: Option<chrono::DateTime<chrono::Utc>>,
    priority: i16,
    attempts: i32,
) -> Result<connectors::models::sync_job::Model> {
    let job_id = Uuid::new_v4();
    let now = chrono::Utc::now();

    let retry_after_str = retry_after
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or("NULL".to_string());

    let stmt = Statement::from_string(
        db.get_database_backend(),
        format!(
            r#"
            INSERT INTO sync_jobs (
                id, tenant_id, provider_slug, connection_id, job_type, status,
                priority, attempts, scheduled_at, retry_after, started_at, finished_at,
                cursor, error, created_at, updated_at
            ) VALUES (
                '{}', '{}', '{}', '{}', 'full', '{}', {}, {}, '{}', {},
                NULL, NULL, NULL, NULL, '{}', '{}'
            )
            "#,
            job_id,
            tenant_id,
            provider_slug,
            connection_id,
            status,
            priority,
            attempts,
            now.format("%Y-%m-%d %H:%M:%S"),
            if retry_after.is_some() {
                format!("'{}'", retry_after_str)
            } else {
                "NULL".to_string()
            },
            now.format("%Y-%m-%d %H:%M:%S"),
            now.format("%Y-%m-%d %H:%M:%S")
        ),
    );

    db.execute(stmt).await?;

    let job = connectors::models::sync_job::Entity::find_by_id(job_id)
        .one(db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Failed to find created job"))?;

    Ok(job)
}
