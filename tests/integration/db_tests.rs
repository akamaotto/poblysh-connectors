//! Integration tests for database functionality.
//!
//! These tests use testcontainers to spin up a real Postgres instance
//! and test migration execution and basic database operations.

use connectors::{config::AppConfig, db};
use migration::{Migrator, MigratorTrait};
use sea_orm::{DatabaseConnection, EntityTrait, Statement};
use testcontainers::{
    core::IntoContainerPort,
    runners::AsyncRunner,
    images::postgres::Postgres,
};
use uuid::Uuid;
use std::time::Duration;

/// Test database migration execution and verification
#[tokio::test]
async fn test_database_migrations() -> anyhow::Result<()> {
    // Start Postgres container
    let postgres = Postgres::default();
    let container = postgres.start().await?;
    let port = container.get_host_port_ipv4(5432).await?;
    
    // Wait for Postgres to be ready
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Create test configuration
    let database_url = format!(
        "postgresql://postgres:postgres@localhost:{}/test",
        port
    );
    
    let mut config = AppConfig::default();
    config.database_url = database_url;
    config.db_max_connections = 5;
    config.db_acquire_timeout_ms = 5000;
    
    // Initialize database connection
    let db = db::init_pool(&config).await?;
    
    // Run migrations
    Migrator::up(&db, None).await?;
    
    // Verify tenants table exists by checking applied migrations
    let applied_migrations = Migrator::get_applied_migrations(&db).await?;
    assert!(!applied_migrations.is_empty(), "No migrations were applied");
    
    // Verify we can query the database
    db::health_check(&db).await?;
    
    println!("✅ Database migrations test passed");
    Ok(())
}

/// Test tenant insertion and retrieval
#[tokio::test]
async fn test_tenant_operations() -> anyhow::Result<()> {
    // Start Postgres container
    let postgres = Postgres::default();
    let container = postgres.start().await?;
    let port = container.get_host_port_ipv4(5432).await?;
    
    // Wait for Postgres to be ready
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Create test configuration
    let database_url = format!(
        "postgresql://postgres:postgres@localhost:{}/test",
        port
    );
    
    let mut config = AppConfig::default();
    config.database_url = database_url;
    
    // Initialize database connection
    let db = db::init_pool(&config).await?;
    
    // Run migrations
    Migrator::up(&db, None).await?;
    
    // Insert a test tenant
    let tenant_id = Uuid::new_v4();
    let tenant_name = "Test Tenant";
    
    let stmt = Statement::from_string(
        db.get_database_backend(),
        format!(
            "INSERT INTO tenants (id, name) VALUES ('{}', '{}')",
            tenant_id, tenant_name
        ),
    );
    
    db.execute(stmt).await?;
    
    // Verify the tenant was inserted
    let stmt = Statement::from_string(
        db.get_database_backend(),
        format!("SELECT id, name FROM tenants WHERE id = '{}'", tenant_id),
    );
    
    let result = db.query_one(stmt).await?;
    
    assert!(result.is_some(), "Tenant was not inserted");
    
    let row = result.unwrap();
    let retrieved_id: Uuid = row.try_get("", "id")?;
    let retrieved_name: String = row.try_get("", "name")?;
    
    assert_eq!(retrieved_id, tenant_id);
    assert_eq!(retrieved_name, tenant_name);
    
    println!("✅ Tenant operations test passed");
    Ok(())
}

/// Test database connection pool configuration
#[tokio::test]
async fn test_connection_pool_configuration() -> anyhow::Result<()> {
    // Start Postgres container
    let postgres = Postgres::default();
    let container = postgres.start().await?;
    let port = container.get_host_port_ipv4(5432).await?;
    
    // Wait for Postgres to be ready
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Create test configuration with custom pool settings
    let database_url = format!(
        "postgresql://postgres:postgres@localhost:{}/test",
        port
    );
    
    let mut config = AppConfig::default();
    config.database_url = database_url;
    config.db_max_connections = 3;
    config.db_acquire_timeout_ms = 2000;
    
    // Initialize database connection
    let db = db::init_pool(&config).await?;
    
    // Verify database is working
    db::health_check(&db).await?;
    
    println!("✅ Connection pool configuration test passed");
    Ok(())
}

/// Test migration rollback functionality
#[tokio::test]
async fn test_migration_rollback() -> anyhow::Result<()> {
    // Start Postgres container
    let postgres = Postgres::default();
    let container = postgres.start().await?;
    let port = container.get_host_port_ipv4(5432).await?;
    
    // Wait for Postgres to be ready
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Create test configuration
    let database_url = format!(
        "postgresql://postgres:postgres@localhost:{}/test",
        port
    );
    
    let mut config = AppConfig::default();
    config.database_url = database_url;
    
    // Initialize database connection
    let db = db::init_pool(&config).await?;
    
    // Run migrations
    Migrator::up(&db, None).await?;
    
    // Verify migrations were applied
    let applied_before = Migrator::get_applied_migrations(&db).await?;
    assert!(!applied_before.is_empty());
    
    // Rollback one migration
    Migrator::down(&db, Some(1)).await?;
    
    // Verify one migration was rolled back
    let applied_after = Migrator::get_applied_migrations(&db).await?;
    assert_eq!(applied_after.len(), applied_before.len() - 1);
    
    println!("✅ Migration rollback test passed");
    Ok(())
}