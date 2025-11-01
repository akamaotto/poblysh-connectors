//! Database connection and pool management for the Connectors API.
//!
//! This module provides functionality to initialize and manage a SeaORM
//! connection pool to Postgres with configurable parameters.

use anyhow::{Context, Result};
use sea_orm::{Database, DatabaseConnection, ConnectOptions, ConnectionTrait};
use std::time::Duration;
use tokio::time::sleep;

use crate::config::AppConfig;

/// Errors that can occur during database operations.
#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("Failed to connect to database: {source}")]
    ConnectionFailed {
        #[from]
        source: sea_orm::DbErr,
    },
    #[error("Database connection timeout after {timeout_ms}ms")]
    ConnectionTimeout { timeout_ms: u64 },
    #[error("Invalid database configuration: {message}")]
    InvalidConfiguration { message: String },
}

/// Initializes a database connection pool with the given configuration.
///
/// This function creates a connection pool to Postgres using SeaORM with
/// configurable maximum connections and acquire timeout. It implements
/// retry logic with exponential backoff for transient errors.
///
/// # Arguments
///
/// * `cfg` - Application configuration containing database settings
///
/// # Returns
///
/// Returns a `DatabaseConnection` pool on success, or an error on failure.
///
/// # Examples
///
/// ```no_run
/// use connectors::{config::AppConfig, db::init_pool};
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let config = AppConfig::default();
///     let db = init_pool(&config).await?;
///     // Use the database connection...
///     Ok(())
/// }
/// ```
pub async fn init_pool(cfg: &AppConfig) -> Result<DatabaseConnection> {
    // Validate database URL
    if cfg.database_url.is_empty() {
        return Err(DatabaseError::InvalidConfiguration {
            message: "Database URL cannot be empty".to_string(),
        }
        .into());
    }

    // Configure connection options
    let mut opt = ConnectOptions::new(&cfg.database_url);
    opt.max_connections(cfg.db_max_connections)
        .acquire_timeout(Duration::from_millis(cfg.db_acquire_timeout_ms))
        .idle_timeout(Duration::from_secs(600)) // 10 minutes
        .max_lifetime(Duration::from_secs(1800)) // 30 minutes
        .sqlx_logging(true)
        .sqlx_logging_level(log::LevelFilter::Debug);

    // Implement retry logic with exponential backoff
    let max_retries = 5;
    let mut retry_delay = Duration::from_millis(100);

    for attempt in 1..=max_retries {
        match Database::connect(opt.clone()).await {
            Ok(conn) => {
                log::info!(
                    "Successfully connected to database (attempt {})",
                    attempt
                );
                return Ok(conn);
            }
            Err(e) => {
                if attempt == max_retries {
                    log::error!(
                        "Failed to connect to database after {} attempts: {}",
                        max_retries,
                        e
                    );
                    return Err(DatabaseError::ConnectionFailed { source: e }.into());
                }

                log::warn!(
                    "Database connection attempt {} failed: {}, retrying in {:?}",
                    attempt,
                    e,
                    retry_delay
                );

                sleep(retry_delay).await;
                retry_delay *= 2; // Exponential backoff
            }
        }
    }

    Err(DatabaseError::ConnectionTimeout {
        timeout_ms: cfg.db_acquire_timeout_ms,
    }
    .into())
}

/// Health check for the database connection.
///
/// This function verifies that the database connection is still active
/// by executing a simple query.
///
/// # Arguments
///
/// * `db` - Database connection to check
///
/// # Returns
///
/// Returns `Ok(())` if the connection is healthy, or an error otherwise.
pub async fn health_check(db: &DatabaseConnection) -> Result<()> {
    use sea_orm::Statement;
    
    let stmt = Statement::from_string(
        db.get_database_backend(),
        "SELECT 1".to_string(),
    );
    
    db.query_one(stmt).await
        .context("Database health check failed")?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_database_url() {
        let mut config = AppConfig::default();
        config.database_url = "".to_string();

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(init_pool(&config));
        
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err().downcast::<DatabaseError>(),
            Ok(DatabaseError::InvalidConfiguration { .. })
        ));
    }
}