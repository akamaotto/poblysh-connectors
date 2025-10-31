//! # Server Configuration
//! 
//! This module contains the server setup and configuration for the Connectors API.

use axum::{
    routing::get,
    Router,
};
use std::net::SocketAddr;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::handlers;

/// Creates and configures the Axum application router
pub fn create_app() -> Router {
    Router::new()
        .route("/", get(handlers::root))
        .merge(SwaggerUi::new("/docs")
            .url("/openapi.json", ApiDoc::openapi()))
}

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// The address to bind the server to
    pub bind_addr: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_addr: std::env::var("POBLYSH_API_BIND_ADDR")
                .unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
        }
    }
}

/// Starts the server with the given configuration
pub async fn run_server(config: ServerConfig) -> Result<(), Box<dyn std::error::Error>> {
    let app = create_app();
    
    let addr: SocketAddr = config.bind_addr.parse()?;
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("Server listening on: {}", addr);
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

/// OpenAPI documentation
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::root,
    ),
    components(
        schemas(
            crate::models::ServiceInfo,
        )
    ),
    info(
        title = "Poblysh Connectors API",
        description = "API for managing connectors",
        version = env!("CARGO_PKG_VERSION"),
    )
)]
pub struct ApiDoc;