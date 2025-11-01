//! # Server Configuration
//!
//! This module contains the server setup and configuration for the Connectors API.

use axum::{Router, routing::get};
use sea_orm::DatabaseConnection;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::config::AppConfig;
use crate::handlers;

/// Application state containing shared resources
#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
}

/// Creates and configures the Axum application router
pub fn create_app(state: AppState) -> Router {
    Router::new()
        .route("/", get(handlers::root))
        .with_state(state)
        .merge(SwaggerUi::new("/docs").url("/openapi.json", ApiDoc::openapi()))
}

/// Starts the server with the given configuration
pub async fn run_server(
    config: AppConfig,
    db: DatabaseConnection,
) -> Result<(), Box<dyn std::error::Error>> {
    let state = AppState { db };
    let app = create_app(state);

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
