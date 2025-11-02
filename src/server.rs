//! # Server Configuration
//!
//! This module contains the server setup and configuration for the Connectors API.

use std::sync::Arc;

use axum::{Router, middleware, routing::get};
use sea_orm::DatabaseConnection;
use utoipa::{Modify, OpenApi};
use utoipa_swagger_ui::SwaggerUi;

use crate::auth::auth_middleware;
use crate::config::AppConfig;
use crate::error::ApiError;
use crate::handlers;

/// Application state containing shared resources
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub db: DatabaseConnection,
}

/// Creates and configures the Axum application router
pub fn create_app(state: AppState) -> Router {
    // Public routes (no auth required)
    let public_routes = Router::new()
        .route("/", get(handlers::root))
        .route("/healthz", get(handlers::health))
        .route("/readyz", get(handlers::ready))
        .route("/providers", get(handlers::providers::list_providers))
        .merge(SwaggerUi::new("/docs").url("/openapi.json", ApiDoc::openapi()));

    // Protected routes (auth required)
    let protected_routes = Router::new()
        .route("/protected/ping", get(handlers::protected_ping))
        .route("/connections", get(handlers::connections::list_connections))
        .layer(middleware::from_fn_with_state(
            Arc::clone(&state.config),
            auth_middleware,
        ));

    // Combine all routes
    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .with_state(state)
}

/// Starts the server with the given configuration
pub async fn run_server(
    config: AppConfig,
    db: DatabaseConnection,
) -> Result<(), Box<dyn std::error::Error>> {
    let shared_config = Arc::new(config);
    let state = AppState {
        config: Arc::clone(&shared_config),
        db,
    };
    let app = create_app(state);

    // Resolve the configured bind address
    let addr = shared_config
        .bind_addr()
        .map_err(|e| format!("Invalid server address: {}", e))?;

    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("Server listening on: {}", addr);
    println!("Running in profile: {}", shared_config.profile);

    axum::serve(listener, app).await?;

    Ok(())
}

/// OpenAPI documentation
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::root,
        crate::handlers::health,
        crate::handlers::ready,
        crate::handlers::protected_ping,
        crate::handlers::providers::list_providers,
        crate::handlers::connections::list_connections,
    ),
    components(
        schemas(
            crate::models::ServiceInfo,
            ApiError,
            crate::auth::TenantHeader,
            crate::handlers::ProtectedPingResponse,
            crate::handlers::providers::ProviderInfo,
            crate::handlers::providers::ProvidersResponse,
            crate::handlers::connections::ConnectionInfo,
            crate::handlers::connections::ConnectionsResponse,
            crate::handlers::connections::ListConnectionsQuery,
        ),
    ),
    modifiers(&SecurityAddon),
    info(
        title = "Poblysh Connectors API",
        description = "API for managing connectors",
        version = env!("CARGO_PKG_VERSION"),
    ),
    servers(
        (url = "/", description = "Default server"),
    ),
    tags(
        (name = "root", description = "Root endpoint"),
        (name = "health", description = "Health check endpoints"),
        (name = "operators", description = "Operator-scoped endpoints"),
        (name = "providers", description = "Provider listing endpoints"),
    ),
    security(
        ("bearer_auth" = []),
    )
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        use utoipa::openapi::security::{Http, HttpAuthScheme, SecurityScheme};

        let mut components = openapi.components.take().unwrap_or_default();
        components.security_schemes.insert(
            "bearer_auth".into(),
            SecurityScheme::Http(
                Http::builder()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("Opaque token")
                    .description(Some("Operator bearer token"))
                    .build(),
            ),
        );
        openapi.components = Some(components);
    }
}
