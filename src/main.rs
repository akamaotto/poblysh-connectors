//! # Connectors API Main Entry Point
//!
//! This is the main entry point for the Connectors API service.

use clap::{Parser, Subcommand};
use connectors::{config::ConfigLoader, connectors::Registry, db, server::run_server};
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
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Load configuration from layered env files and variables
    let config_loader = ConfigLoader::new();
    let config = config_loader.load()?;

    // Initialize database connection
    let db = db::init_pool(&config).await?;

    // Handle CLI commands
    if let Some(command) = cli.command {
        match command {
            Commands::Migrate { action } => {
                handle_migrate_command(&db, action).await?;
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
    Registry::initialize();
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
) -> Result<(), Box<dyn std::error::Error>> {
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
