//! # Connectors API Main Entry Point
//!
//! This is the main entry point for the Connectors API service.

use clap::{Parser, Subcommand};
use connectors::{
    config::ConfigLoader, connectors::Registry, db, server::run_server,
    sync_executor::ExecutorConfig, telemetry,
};
use migration::{Migrator, MigratorTrait};
use sea_orm::DatabaseConnection;

#[derive(Parser)]
#[command(name = "connectors")]
#[command(about = "Connectors API service")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run database migrations
    Migrate {
        #[command(subcommand)]
        action: MigrateAction,
    },
    /// Run the sync executor service
    SyncExecutor,
    /// Run both API server and sync executor
    RunAll,
}

#[derive(Subcommand)]
enum MigrateAction {
    /// Apply all pending migrations
    Up,
    /// Rollback the last migration
    Down,
    /// Show migration status
    Status,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cli = Cli::parse();

    // Load configuration from layered env files and variables
    let config_loader = ConfigLoader::new();
    let config = config_loader.load()?;

    // Initialize tracing subscriber based on configuration
    telemetry::init_tracing(&config)?;

    // Initialize database connection
    let db = db::init_pool(&config).await?;

    // Handle CLI commands
    if let Some(command) = cli.command {
        match command {
            Commands::Migrate { action } => {
                handle_migrate_command(&db, action).await?;
                return Ok(());
            }
            Commands::SyncExecutor => {
                handle_sync_executor_command(config, db).await?;
                return Ok(());
            }
            Commands::RunAll => {
                println!("Starting both API server and sync executor...");

                // For now, run the server first
                println!("Starting API server...");
                run_server(config, db).await?;
                return Ok(());
            }
        }
    }

    // Run migrations automatically for local and test profiles
    if config.profile == "local" || config.profile == "test" {
        println!(
            "Running migrations automatically for profile: {}",
            config.profile
        );
        Migrator::up(&db, None).await?;
        println!("Migrations completed successfully");
    }

    // Initialize the connector registry
    Registry::initialize(&config);
    println!("Connector registry initialized with example provider");

    // Log the loaded configuration (no secrets in current schema)
    println!("Loaded configuration for profile: {}", config.profile);
    if let Ok(redacted_json) = config.redacted_json() {
        println!("Configuration: {}", redacted_json);
    }

    // Start the server with the loaded configuration
    run_server(config, db).await
}

async fn handle_migrate_command(
    db: &DatabaseConnection,
    action: MigrateAction,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match action {
        MigrateAction::Up => {
            println!("Applying migrations...");
            Migrator::up(db, None).await?;
            println!("All migrations applied successfully");
        }
        MigrateAction::Down => {
            println!("Rolling back last migration...");
            Migrator::down(db, Some(1)).await?;
            println!("Migration rolled back successfully");
        }
        MigrateAction::Status => {
            println!("Checking migration status...");
            let applied = Migrator::get_applied_migrations(db).await?;
            let pending = Migrator::get_pending_migrations(db).await?;

            if applied.is_empty() {
                println!("No migrations have been applied");
            } else {
                println!("Applied migrations: {} migration(s)", applied.len());
            }

            if pending.is_empty() {
                println!("No pending migrations");
            } else {
                println!("Pending migrations: {} migration(s)", pending.len());
            }
        }
    }
    Ok(())
}

async fn handle_sync_executor_command(
    config: connectors::config::AppConfig,
    db: DatabaseConnection,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Starting sync executor service...");

    // Run migrations automatically for local and test profiles
    if config.profile == "local" || config.profile == "test" {
        println!(
            "Running migrations for sync executor (profile: {})",
            config.profile
        );
        Migrator::up(&db, None).await?;
        println!("Migrations completed successfully");
    }

    // Initialize the connector registry
    Registry::initialize(&config);
    println!("Connector registry initialized");

    // Log rate limit policy configuration
    println!("Rate limit policy:");
    println!("  Base seconds: {}", config.rate_limit_policy.base_seconds);
    println!("  Max seconds: {}", config.rate_limit_policy.max_seconds);
    println!(
        "  Jitter factor: {}",
        config.rate_limit_policy.jitter_factor
    );
    if !config.rate_limit_policy.provider_overrides.is_empty() {
        println!("  Provider overrides:");
        for (provider, override_config) in &config.rate_limit_policy.provider_overrides {
            println!("    {}: {:?}", provider, override_config);
        }
    } else {
        println!("  No provider overrides configured");
    }

    // Create executor configuration
    let executor_config = ExecutorConfig::default();
    println!("Executor configuration:");
    println!("  Tick interval: {}ms", executor_config.tick_ms);
    println!("  Concurrency: {}", executor_config.concurrency);
    println!("  Claim batch: {}", executor_config.claim_batch);
    println!("  Max run time: {}s", executor_config.max_run_seconds);
    println!("  Max items per run: {}", executor_config.max_items_per_run);

    // Create crypto key and connection repository
    let crypto_key =
        connectors::crypto::CryptoKey::new(config.crypto_key.as_ref().unwrap().clone())
            .map_err(|e| format!("Failed to create crypto key: {}", e))?;
    // For now, create sync executor without token refresh service due to type issues
    // TODO: Reintegrate token refresh service once types are resolved
    let executor = connectors::sync_executor::SyncExecutor::new(
        db.clone(),
        Registry::global().read().unwrap().clone(),
        executor_config,
        config.rate_limit_policy.clone(),
        // Temporary placeholder - proper implementation needed
        std::sync::Arc::new(connectors::token_refresh::TokenRefreshService::new(
            std::sync::Arc::new(config.clone()),
            std::sync::Arc::new(db.clone()),
            std::sync::Arc::new(
                connectors::repositories::connection::ConnectionRepository::new(
                    std::sync::Arc::new(db.clone()),
                    crypto_key,
                ),
            ),
            Registry::global().read().unwrap().clone(),
        )),
    );

    println!("Sync executor started. Press Ctrl+C to stop.");

    // Run the executor loop (this will block until interrupted)
    executor.run().await
}
