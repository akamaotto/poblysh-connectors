//! # Server Configuration
//!
//! This module contains the server setup and configuration for the Connectors API.

use std::sync::Arc;
use tower_http::{
    request_id::{SetRequestIdLayer, PropagateRequestIdLayer, MakeRequestId},
    trace::{TraceLayer, MakeSpan},
};
use uuid::Uuid;
use axum::{Router, middleware, routing::get, http::Request};
use tracing::Span;
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
    pub db: Option<DatabaseConnection>,
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
        .layer(middleware::from_fn_with_state(
            Arc::clone(&state.config),
            auth_middleware,
        ));

    // Combine all routes
    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(CustomMakeSpan)
        )
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(SetRequestIdLayer::x_request_id(MyMakeRequestId))
        .with_state(state)
}

#[derive(Clone, Copy)]
struct MyMakeRequestId;

impl MakeRequestId for MyMakeRequestId {
    fn make_request_id<B>(&mut self, _request: &Request<B>) -> Option<tower_http::request_id::RequestId> {
        let request_id = Uuid::new_v4().to_string();
        Some(tower_http::request_id::RequestId::new(request_id.parse().unwrap()))
    }
}

#[derive(Clone)]
struct CustomMakeSpan;

impl<B> MakeSpan<B> for CustomMakeSpan {
    fn make_span(&mut self, request: &Request<B>) -> Span {
        let request_id = request
            .extensions()
            .get::<tower_http::request_id::RequestId>()
            .and_then(|id| id.header_value().to_str().ok())
            .unwrap_or("unknown");
        tracing::error_span!(
            "http-request",
            "http.method" = %request.method(),
            "http.uri" = %request.uri(),
            "http.version" = ?request.version(),
            "trace_id" = %request_id,
        )
    }
}

/// Starts the server with the given configuration
pub async fn run_server(
    config: AppConfig,
    db: Option<DatabaseConnection>,
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
    tracing::info!("Server listening on: {}", addr);
    tracing::info!("Running in profile: {}", shared_config.profile);

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
    ),
    components(
        schemas(
            crate::models::ServiceInfo,
            ApiError,
            crate::auth::TenantHeader,
            crate::handlers::ProtectedPingResponse,
            crate::handlers::providers::ProviderInfo,
            crate::handlers::providers::ProvidersResponse,
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
