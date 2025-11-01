//! Lightweight integration tests verifying migrations apply on SQLite.

use anyhow::Result;
use connectors::db;
use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectionTrait, Database, Statement};

#[tokio::test]
async fn migrations_apply_and_tenants_table_exists() -> Result<()> {
    let db = Database::connect("sqlite::memory:").await?;
    Migrator::up(&db, None).await?;

    let stmt = Statement::from_string(
        db.get_database_backend(),
        "SELECT name FROM sqlite_master WHERE name = 'tenants'".to_string(),
    );
    let result = db.query_one(stmt).await?;
    assert!(result.is_some());
    Ok(())
}

#[tokio::test]
async fn health_check_passes_after_migrations() -> Result<()> {
    let db = Database::connect("sqlite::memory:").await?;
    Migrator::up(&db, None).await?;
    db::health_check(&db).await?;
    Ok(())
}
