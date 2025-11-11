//! # Server Configuration
//!
//! This module contains the server setup and configuration for the Connectors API.

use std::sync::Arc;

use axum::http::{HeaderName, Method};
use axum::{
    Router,
    extract::Request,
    http::HeaderValue,
    middleware,
    response::Response,
    routing::{delete, get, patch, post},
};
use sea_orm::DatabaseConnection;
use std::time::Duration;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use utoipa::{Modify, OpenApi};
use utoipa_swagger_ui::SwaggerUi;

use crate::auth::auth_middleware;
use crate::config::AppConfig;
use crate::connectors::Registry;
use crate::crypto::CryptoKey;
use crate::error::ApiError;
use crate::handlers;
use crate::repositories::connection::ConnectionRepository;
use crate::telemetry::{self, TraceContext};
use crate::token_refresh::TokenRefreshService;
use crate::webhook_verification::webhook_verification_middleware;
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
    pub token_refresh_service: Arc<TokenRefreshService>,
}

/// Creates and configures the Axum application router
pub fn create_app(state: AppState) -> Router {
    // Public routes (no auth required)
    let public_routes = Router::new()
        .route("/", get(handlers::root))
        .route("/healthz", get(handlers::health))
        .route("/readyz", get(handlers::ready))
        .route(
            "/config/rate-limit-policy",
            get(handlers::config::get_rate_limit_policy_config),
        )
        .route("/config/summary", get(handlers::config::get_config_summary))
        .route("/providers", get(handlers::providers::list_providers))
        .route(
            "/connect/{provider}/callback",
            get(handlers::connect::oauth_callback),
        )
        // Public webhook routes with signature verification
        .route(
            "/webhooks/{provider}/{tenant_id}",
            post(handlers::webhooks::ingest_public_webhook),
        )
        .layer(middleware::from_fn_with_state(
            Arc::clone(&state.config),
            webhook_verification_middleware,
        ))
        .merge(SwaggerUi::new("/docs").url("/openapi.json", ApiDoc::openapi()));

    // Protected routes (auth required)
    let protected_routes = Router::new()
        .route("/protected/ping", get(handlers::protected_ping))
        .route("/connections", get(handlers::connections::list_connections))
        .route("/jobs", get(handlers::jobs::list_jobs))
        .route("/signals", get(handlers::signals::list_signals))
        .route(
            "/grounded-signals",
            get(handlers::grounded_signals::list_grounded_signals),
        )
        .route(
            "/grounded-signals/{id}",
            get(handlers::grounded_signals::get_grounded_signal),
        )
        .route(
            "/grounded-signals/{id}",
            patch(handlers::grounded_signals::update_grounded_signal),
        )
        .route(
            "/grounded-signals/{id}",
            delete(handlers::grounded_signals::delete_grounded_signal),
        )
        .route("/api/v1/tenants", post(handlers::tenants::create_tenant))
        .route("/api/v1/tenants/{id}", get(handlers::tenants::get_tenant))
        .route("/connect/{provider}", post(handlers::connect::start_oauth))
        .route(
            "/webhooks/{provider}",
            post(handlers::webhooks::ingest_webhook),
        )
        .layer(middleware::from_fn_with_state(
            Arc::clone(&state.config),
            auth_middleware,
        ));

    // Combine all routes with CORS, tracing, and trace ID middleware
    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .with_state(state)
        // CORS: allow frontend dev origin to call backend.
        // For local development we allow:
        // - http://localhost:3000
        // - http://127.0.0.1:3000
        // You can tighten this as needed.
        .layer(
            CorsLayer::new()
                .allow_origin(Any) // Simplified for local dev; restrict in production.
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::PATCH,
                    Method::DELETE,
                    Method::OPTIONS,
                ])
                .allow_headers([
                    axum::http::header::CONTENT_TYPE,
                    axum::http::header::AUTHORIZATION,
                    HeaderName::from_static("x-tenant-id"),
                ]),
        )
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

/// Creates a test AppState with TokenRefreshService for testing purposes
pub fn create_test_app_state(config: AppConfig, db: DatabaseConnection) -> AppState {
    let crypto_key =
        crate::crypto::CryptoKey::new(vec![0u8; 32]).expect("Failed to create crypto key for test");

    // Create required dependencies for TokenRefreshService
    let connection_repo = crate::repositories::ConnectionRepository::new(
        std::sync::Arc::new(db.clone()),
        crypto_key.clone(),
    );

    // Create TokenRefreshService
    let token_refresh_service =
        std::sync::Arc::new(crate::token_refresh::TokenRefreshService::new(
            std::sync::Arc::new(config.clone()),
            std::sync::Arc::new(db.clone()),
            std::sync::Arc::new(connection_repo),
            crate::connectors::registry::Registry::new(),
        ));

    AppState {
        config: std::sync::Arc::new(config),
        db,
        crypto_key,
        token_refresh_service,
    }
}

/// Starts the server with the given configuration
pub async fn run_server(
    config: AppConfig,
    db: DatabaseConnection,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let shared_config = Arc::new(config);
    let shared_db = Arc::new(db);

    // Initialize the connector registry
    Registry::initialize(shared_config.as_ref());
    println!("Connector registry initialized");

    // Create crypto key from config
    let crypto_key = CryptoKey::new(
        shared_config
            .crypto_key
            .as_ref()
            .ok_or("Crypto key is required")?
            .clone(),
    )
    .map_err(|e| format!("Failed to create crypto key: {}", e))?;

    // Create connection repository for token refresh service
    let connection_repo = Arc::new(ConnectionRepository::new(
        shared_db.clone(),
        crypto_key.clone(),
    ));

    // Create and start token refresh service
    let token_refresh_service = Arc::new(TokenRefreshService::new(
        shared_config.clone(),
        shared_db.clone(),
        connection_repo,
        Registry::global().read().unwrap().clone(),
    ));

    let shutdown_token = tokio_util::sync::CancellationToken::new();
    let shutdown_token_for_server = shutdown_token.clone();
    let shutdown_token_for_refresh = shutdown_token.clone();

    let state = AppState {
        config: Arc::clone(&shared_config),
        db: (*shared_db).clone(),
        crypto_key,
        token_refresh_service: Arc::clone(&token_refresh_service),
    };
    let app = create_app(state);

    // Resolve the configured bind address
    let addr = shared_config
        .bind_addr()
        .map_err(|e| format!("Invalid server address: {}", e))?;

    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("Server listening on: {}", addr);
    println!("Running in profile: {}", shared_config.profile);
    println!("Token refresh service started");

    // Start token refresh service in background
    let token_refresh_service_clone = token_refresh_service.clone();
    let token_refresh_handle = tokio::spawn(async move {
        if let Err(e) = token_refresh_service_clone
            .run(shutdown_token_for_refresh)
            .await
        {
            eprintln!("Token refresh service error: {:?}", e);
        }
    });

    // Start the server with graceful shutdown
    let server_handle = tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                tokio::signal::ctrl_c()
                    .await
                    .expect("Failed to install Ctrl+C handler");
                println!("Received shutdown signal");
                shutdown_token_for_server.cancel();
            })
            .await
    });

    // Wait for either the server or token refresh service to complete
    tokio::select! {
        result = server_handle => {
            if let Err(e) = result {
                eprintln!("Server error: {:?}", e);
            }
        }
        result = token_refresh_handle => {
            if let Err(e) = result {
                eprintln!("Token refresh service error: {:?}", e);
            }
        }
    }

    println!("Server shutdown complete");
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
        crate::handlers::config::get_rate_limit_policy_config,
        crate::handlers::config::get_config_summary,
        crate::handlers::providers::list_providers,
        crate::handlers::connections::list_connections,
        crate::handlers::jobs::list_jobs,
        crate::handlers::signals::list_signals,
        crate::handlers::grounded_signals::list_grounded_signals,
        crate::handlers::grounded_signals::get_grounded_signal,
        crate::handlers::grounded_signals::update_grounded_signal,
        crate::handlers::grounded_signals::delete_grounded_signal,
        crate::handlers::tenants::create_tenant,
        crate::handlers::tenants::get_tenant,
        crate::handlers::connect::start_oauth,
        crate::handlers::connect::oauth_callback,
        crate::handlers::webhooks::ingest_webhook,
        crate::handlers::webhooks::ingest_public_webhook,
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
            crate::handlers::jobs::JobInfo,
            crate::handlers::jobs::JobsResponse,
            crate::handlers::jobs::JobStatusParam,
            crate::handlers::jobs::JobTypeParam,
            crate::handlers::signals::SignalInfo,
            crate::handlers::signals::SignalsResponse,
            crate::handlers::signals::ListSignalsQuery,
            crate::handlers::tenants::CreateTenantRequestDto,
            crate::handlers::tenants::CreateTenantResponseDto,
            crate::handlers::tenants::TenantResponseMeta,
            crate::handlers::connect::ProviderPath,
            crate::handlers::connect::OAuthCallbackQuery,
            crate::handlers::connect::ConnectionResponse,
            crate::handlers::connect::ConnectionInfo,
            crate::handlers::connect::AuthorizeUrlResponse,
            crate::handlers::ReadinessResponse,
            crate::handlers::webhooks::WebhookAcceptResponse,
            crate::handlers::webhooks::ProviderPath,
            crate::handlers::webhooks::GitHubSignatureHeader,
            crate::handlers::webhooks::SlackSignatureHeaders,
        crate::config::RateLimitPolicyConfig,
        crate::config::RateLimitProviderOverride,
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
        (name = "configuration", description = "Configuration endpoints"),
        (name = "operators", description = "Operator-scoped endpoints"),
        (name = "providers", description = "Provider listing endpoints"),
        (name = "webhooks", description = "Webhook ingest endpoints"),
        (name = "jobs", description = "Jobs listing and management endpoints"),
        (name = "signals", description = "Signals listing and querying endpoints"),
        (name = "grounded-signals", description = "Grounded signals management endpoints"),
        (name = "tenants", description = "Tenant management endpoints"),
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
