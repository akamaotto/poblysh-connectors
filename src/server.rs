//! # Server Configuration
//!
//! This module contains the server setup and configuration for the Connectors API.

use std::sync::Arc;

use axum::{
    Router,
    extract::Request,
    http::HeaderValue,
    middleware,
    response::Response,
    routing::{get, post},
};
use sea_orm::DatabaseConnection;
use std::time::Duration;
use tower_http::trace::TraceLayer;
use utoipa::{Modify, OpenApi};
use utoipa_swagger_ui::SwaggerUi;

use crate::auth::auth_middleware;
use crate::config::AppConfig;
use crate::crypto::CryptoKey;
use crate::error::ApiError;
use crate::handlers;
use crate::telemetry::{self, TraceContext};
use uuid::Uuid;

/// Middleware to generate trace_id and store it in request extensions
async fn trace_middleware(mut request: Request, next: axum::middleware::Next) -> Response {
    let trace_context = TraceContext {
        trace_id: Uuid::new_v4().to_string(),
    };
    let header_trace_id = trace_context.trace_id.clone();

    // Store TraceContext in request extensions for handlers to access
    request.extensions_mut().insert(trace_context.clone());

    telemetry::with_trace_context(trace_context, async move {
        let response = next.run(request).await;

        // Add trace_id to response headers
        let mut response = response;
        if let Ok(header_value) = HeaderValue::from_str(&header_trace_id) {
            response.headers_mut().insert("x-trace-id", header_value);
        }

        response
    })
    .await
}

/// Application state containing shared resources
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub db: DatabaseConnection,
    pub crypto_key: CryptoKey,
}

/// Creates and configures the Axum application router
pub fn create_app(state: AppState) -> Router {
    // Public routes (no auth required)
    let public_routes = Router::new()
        .route("/", get(handlers::root))
        .route("/healthz", get(handlers::health))
        .route("/readyz", get(handlers::ready))
        .route("/providers", get(handlers::providers::list_providers))
        .route(
            "/connect/{provider}/callback",
            get(handlers::connect::oauth_callback),
        )
        .merge(SwaggerUi::new("/docs").url("/openapi.json", ApiDoc::openapi()));

    // Protected routes (auth required)
    let protected_routes = Router::new()
        .route("/protected/ping", get(handlers::protected_ping))
        .route("/connections", get(handlers::connections::list_connections))
        .route("/connect/{provider}", post(handlers::connect::start_oauth))
        .layer(middleware::from_fn_with_state(
            Arc::clone(&state.config),
            auth_middleware,
        ));

    // Combine all routes with tracing
    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .with_state(state)
        // Add HTTP request tracing
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<_>| {
                    let method = request.method().to_string();
                    let uri = request.uri().to_string();
                    let trace_ctx = request
                        .extensions()
                        .get::<TraceContext>()
                        .cloned()
                        .unwrap_or_else(|| TraceContext {
                            trace_id: Uuid::new_v4().to_string(),
                        });
                    let trace_id = trace_ctx.trace_id.clone();

                    let span = tracing::info_span!(
                        "http_request",
                        method = %method,
                        path = %uri,
                        status = tracing::field::Empty,
                        latency_ms = tracing::field::Empty,
                        trace_id = %trace_id,
                    );

                    span
                })
                .on_request(|request: &Request<_>, _span: &tracing::Span| {
                    // Count sensitive headers for redaction (avoid logging their values)
                    let headers = request.headers();
                    let _redacted_count = headers
                        .iter()
                        .filter(|(name, _value)| {
                            let name_str = name.as_str();
                            name_str.to_lowercase().contains("authorization")
                                || name_str.to_lowercase().contains("cookie")
                                || name_str.to_lowercase().contains("set-cookie")
                        })
                        .count();

                    tracing::info!("request started",);
                })
                .on_response(
                    |response: &Response<_>, latency: Duration, span: &tracing::Span| {
                        let status = response.status().to_string();
                        span.record("status", tracing::field::display(&status));
                        span.record("latency_ms", tracing::field::display(latency.as_millis()));
                        tracing::info!(
                            status = %status,
                            latency_ms = latency.as_millis(),
                            "request completed"
                        );
                    },
                ),
        )
        // Add trace ID generation middleware
        .layer(middleware::from_fn(trace_middleware))
}

/// Starts the server with the given configuration
pub async fn run_server(
    config: AppConfig,
    db: DatabaseConnection,
) -> Result<(), Box<dyn std::error::Error>> {
    let shared_config = Arc::new(config);

    // Create crypto key from config
    let crypto_key = CryptoKey::new(
        shared_config
            .crypto_key
            .as_ref()
            .ok_or("Crypto key is required")?
            .clone(),
    )
    .map_err(|e| format!("Failed to create crypto key: {}", e))?;

    let state = AppState {
        config: Arc::clone(&shared_config),
        db,
        crypto_key,
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
    info(
        title = "Poblysh Connectors API",
        version = "0.1.0",
        description = "API for managing connector integrations with secure token encryption using AES-256-GCM",
        license(
            name = "Security Notice",
            url = "https://docs.poblysh.com/security"
        )
    ),
    paths(
        crate::handlers::root,
        crate::handlers::health,
        crate::handlers::ready,
        crate::handlers::protected_ping,
        crate::handlers::providers::list_providers,
        crate::handlers::connections::list_connections,
        crate::handlers::connect::start_oauth,
        crate::handlers::connect::oauth_callback,
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
            crate::handlers::connect::ProviderPath,
            crate::handlers::connect::OAuthCallbackQuery,
            crate::handlers::connect::ConnectionResponse,
            crate::handlers::connect::ConnectionInfo,
            crate::handlers::connect::AuthorizeUrlResponse,
            crate::handlers::ReadinessResponse,
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
                    .description(Some("Operator bearer token. User tokens are stored at rest using AES-256-GCM encryption with tenant-context binding"))
                    .build(),
            ),
        );
        openapi.components = Some(components);
    }
}
