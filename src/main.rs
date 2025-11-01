//! # Connectors API Main Entry Point
//!
//! This is the main entry point for the Connectors API service.

use connectors::{config::ConfigLoader, server::run_server};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration from layered env files and variables
    let config_loader = ConfigLoader::new();
    let config = config_loader.load()?;

    // Log the loaded configuration (no secrets in current schema)
    println!("Loaded configuration for profile: {}", config.profile);
    if let Ok(redacted_json) = config.redacted_json() {
        println!("Configuration: {}", redacted_json);
    }

    // Start the server with the loaded configuration
    run_server(config).await
}
