//! # Server Configuration
//!
//! This module contains the server setup and configuration for the Connectors API.

use axum::{Router, routing::get};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::config::AppConfig;
use crate::handlers;

/// Creates and configures the Axum application router
pub fn create_app() -> Router {
    Router::new()
        .route("/", get(handlers::root))
        .merge(SwaggerUi::new("/docs").url("/openapi.json", ApiDoc::openapi()))
}

/// Starts the server with the given configuration
pub async fn run_server(config: AppConfig) -> Result<(), Box<dyn std::error::Error>> {
    let app = create_app();

    // Resolve the configured bind address
    let addr = config
        .bind_addr()
        .map_err(|e| format!("Invalid server address: {}", e))?;

    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("Server listening on: {}", addr);
    println!("Running in profile: {}", config.profile);

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
