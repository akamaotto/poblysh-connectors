//! # Connectors API Main Entry Point
//! 
//! This is the main entry point for the Connectors API service.

use connectors::server::{run_server, ServerConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the server configuration
    let config = ServerConfig::default();
    
    // Start the server
    run_server(config).await
}