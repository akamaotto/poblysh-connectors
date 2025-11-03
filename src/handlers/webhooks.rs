//! # Webhook Handlers
//!
//! This module contains handlers for processing webhook callbacks from external providers.
//! For MVP, these endpoints are protected by operator authentication and tenant scoping.

use axum::{
    extract::{Path, Request, State},
    http::{HeaderMap, StatusCode},
    response::Json,
};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use tracing::{debug, error, info};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::auth::{OperatorAuth, TenantExtension};
use crate::error::ApiError;
use crate::handlers::TenantHeader;
use crate::repositories::{ConnectionRepository, ProviderRepository, SyncJobRepository};
use crate::server::AppState;

/// Path parameter for provider slug
#[derive(Debug, Deserialize, ToSchema)]
pub struct ProviderPath {
    /// Provider slug (e.g., "github", "jira")
    pub provider: String,
}

/// Optional connection ID header for targeting specific connections
#[derive(Debug, Deserialize)]
pub struct ConnectionIdHeader {
    /// Connection UUID to target (optional)
    #[serde(rename = "X-Connection-Id")]
    pub connection_id: Option<Uuid>,
}

/// Webhook accept response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct WebhookAcceptResponse {
    /// Acceptance status
    pub status: String,
}

/// Path parameter for provider slug with OpenAPI support
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct ProviderPathParam {
    /// Provider slug (e.g., "github", "jira")
    #[param(min_length = 1, example = "github")]
    pub provider: String,
}

/// Helper function to extract optional connection ID from headers
fn extract_connection_id(headers: &HeaderMap) -> Result<Option<Uuid>, ApiError> {
    match headers.get("X-Connection-Id") {
        Some(header_value) => {
            let header_str = header_value.to_str().map_err(|_| {
                ApiError::new(
                    axum::http::StatusCode::BAD_REQUEST,
                    "VALIDATION_FAILED",
                    "invalid X-Connection-Id header encoding",
                )
            })?;

            let uuid = Uuid::parse_str(header_str).map_err(|_| {
                ApiError::new(
                    axum::http::StatusCode::BAD_REQUEST,
                    "VALIDATION_FAILED",
                    "invalid X-Connection-Id header format",
                )
            })?;

            Ok(Some(uuid))
        }
        None => Ok(None),
    }
}

/// Helper function to extract optional JSON body from request
async fn extract_webhook_body(req: Request) -> Result<Option<JsonValue>, ApiError> {
    // Always try to read the body, regardless of Content-Length or Transfer-Encoding
    let (_parts, body) = req.into_parts();

    // Read the body once to handle both Content-Length and chunked Transfer-Encoding
    let bytes = axum::body::to_bytes(body, usize::MAX).await.map_err(|_| {
        ApiError::new(
            axum::http::StatusCode::BAD_REQUEST,
            "VALIDATION_FAILED",
            "Failed to read request body",
        )
    })?;

    // If body is empty, return None
    if bytes.is_empty() {
        return Ok(None);
    }

    // Try to parse as JSON - if it fails, we still want to capture the raw payload
    let json_value = serde_json::from_slice(&bytes).ok();

    Ok(json_value)
}

/// Accept webhook from external provider
///
/// This endpoint receives webhook callbacks from external providers. For MVP,
/// it requires operator authentication and tenant scoping. A valid connection
/// can be specified via the X-Connection-Id header to enqueue a sync job.
#[utoipa::path(
    post,
    path = "/webhooks/{provider}",
    security(("bearer_auth" = [])),
    params(
        TenantHeader,
        ("X-Connection-Id" = Option<String>, Header, description = "Optional connection ID to target"),
        ProviderPathParam
    ),
    request_body(content = Option<JsonValue>, description = "Webhook payload (opaque to API)", content_type = "application/json"),
    responses(
        (status = 202, description = "Webhook accepted", body = WebhookAcceptResponse),
        (status = 400, description = "Invalid connection ID header", body = ApiError),
        (status = 401, description = "Missing or invalid operator token", body = ApiError),
        (status = 404, description = "Provider not found or connection not found for tenant/provider", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    ),
    tag = "webhooks"
)]
pub async fn ingest_webhook(
    State(state): State<AppState>,
    _operator_auth: OperatorAuth,
    TenantExtension(tenant): TenantExtension,
    Path(provider_param): Path<ProviderPathParam>,
    req: Request,
) -> Result<(StatusCode, Json<WebhookAcceptResponse>), ApiError> {
    let provider_slug = provider_param.provider;
    let tenant_id = tenant.0;

    debug!(
        provider_slug = %provider_slug,
        tenant_id = %tenant_id,
        "Processing webhook ingestion"
    );

    // Extract headers before consuming the request
    let headers = req.headers().clone();

    // Filter out sensitive headers before persisting in job cursor
    let sensitive_headers = std::collections::HashSet::from([
        "authorization",
        "cookie",
        "set-cookie",
        "proxy-authorization",
        "www-authenticate",
        "authentication-info",
        "x-api-key",
        "x-auth-token",
        "x-csrf-token",
        "x-xsrf-token",
    ]);

    let webhook_headers: std::collections::HashMap<String, String> = headers
        .iter()
        .filter_map(|(name, value)| {
            let name_lower = name.as_str().to_lowercase();
            // Skip sensitive headers
            if sensitive_headers.contains(&name_lower.as_str()) {
                tracing::debug!("Filtering sensitive header: {}", name_lower);
                return None;
            }

            Some((
                name_lower, // Canonical lower-case headers
                value.to_str().unwrap_or("").to_string(),
            ))
        })
        .collect();

    // Validate provider exists
    let provider_repo = ProviderRepository::new(std::sync::Arc::new(state.db.clone()));
    let _provider = provider_repo
        .find_by_slug(&provider_slug)
        .await
        .map_err(|e| {
            error!(error = ?e, "Failed to lookup provider");
            ApiError::new(
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_SERVER_ERROR",
                "Failed to validate provider",
            )
        })?
        .ok_or_else(|| {
            info!(provider_slug = %provider_slug, "Provider not found");
            ApiError::new(
                axum::http::StatusCode::NOT_FOUND,
                "NOT_FOUND",
                &format!("provider '{}' not found", provider_slug),
            )
        })?;

    // Extract connection ID from headers
    let connection_id = extract_connection_id(req.headers())?;

    // Extract webhook body
    let body = extract_webhook_body(req).await?;

    // If connection ID is provided, validate it belongs to tenant and provider
    if let Some(conn_id) = connection_id {
        let connection_repo = ConnectionRepository::new(
            std::sync::Arc::new(state.db.clone()),
            state.crypto_key.clone(),
        );
        let _connection = connection_repo
            .find_by_tenant_and_provider(&tenant_id, &provider_slug)
            .await
            .map_err(|e| {
                error!(error = ?e, "Failed to validate connection");
                ApiError::new(
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_SERVER_ERROR",
                    "Failed to validate connection",
                )
            })?
            .into_iter()
            .find(|conn| conn.id == conn_id)
            .ok_or_else(|| {
                error!(
                    tenant_id = %tenant_id,
                    provider_slug = %provider_slug,
                    connection_id = %conn_id,
                    "Connection not found for tenant/provider"
                );
                ApiError::new(
                    axum::http::StatusCode::NOT_FOUND,
                    "NOT_FOUND",
                    "connection not found for tenant/provider",
                )
            })?;

        info!(
            tenant_id = %tenant_id,
            provider_slug = %provider_slug,
            connection_id = %conn_id,
            "Valid connection found, enqueuing webhook sync job"
        );

        // Create cursor with webhook context including headers and payload
        // body is Option<JsonValue> from the helper function
        let cursor = Some(serde_json::json!({
            "webhook_headers": webhook_headers,
            "webhook_payload": body,
            "received_at": chrono::Utc::now().to_rfc3339()
        }));

        // Enqueue webhook sync job
        let sync_job_repo = SyncJobRepository::new(state.db.clone());
        sync_job_repo
            .enqueue_webhook_job(tenant_id, &provider_slug, conn_id, cursor)
            .await
            .map_err(|e| {
                error!(error = ?e, "Failed to enqueue webhook sync job");
                ApiError::new(
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_SERVER_ERROR",
                    "Failed to enqueue webhook job",
                )
            })?;

        info!(
            tenant_id = %tenant_id,
            provider_slug = %provider_slug,
            connection_id = %conn_id,
            "Webhook sync job enqueued successfully"
        );
    } else {
        info!(
            tenant_id = %tenant_id,
            provider_slug = %provider_slug,
            "Webhook accepted without connection targeting"
        );
    }

    let response = WebhookAcceptResponse {
        status: "accepted".to_string(),
    };

    Ok((StatusCode::ACCEPTED, Json(response)))
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::config::AppConfig;
    use crate::db::init_pool;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use migration::{Migrator, MigratorTrait};
    use tower::ServiceExt;

    async fn setup_test_app() -> (AppState, axum::Router) {
        let config = AppConfig {
            profile: "test".to_string(),
            operator_tokens: vec!["test-token".to_string()],
            ..Default::default()
        };

        let db = init_pool(&config).await.expect("Failed to init test DB");

        // Apply migrations
        Migrator::up(&db, None).await.unwrap();

        let state = AppState {
            config: std::sync::Arc::new(config),
            db,
            crypto_key: crate::crypto::CryptoKey::new([0; 32].to_vec()).unwrap(),
        };

        let app = crate::server::create_app(state.clone());
        (state, app)
    }

    async fn create_test_provider(state: &AppState, slug: &str) {
        use sea_orm::{ActiveModelTrait, Set};
        let provider_repo = ProviderRepository::new(std::sync::Arc::new(state.db.clone()));

        // Check if provider already exists to avoid constraint violations
        if provider_repo.find_by_slug(slug).await.unwrap().is_some() {
            return; // Provider already exists, skip creation
        }

        let provider = crate::models::provider::ActiveModel {
            slug: Set(slug.to_string()),
            display_name: Set(format!("Test {}", slug)),
            auth_type: Set("oauth2".to_string()),
            created_at: Set(chrono::Utc::now().fixed_offset()),
            updated_at: Set(chrono::Utc::now().fixed_offset()),
        };
        provider.insert(&state.db).await.unwrap();
    }

    async fn create_test_tenant(state: &AppState, tenant_id: Uuid) {
        use sea_orm::{ActiveModelTrait, Set};

        // Check if tenant already exists to avoid constraint violations
        use sea_orm::EntityTrait;
        if let Ok(Some(_)) = crate::models::tenant::Entity::find_by_id(tenant_id)
            .one(&state.db)
            .await
        {
            return; // Tenant already exists
        }

        let tenant = crate::models::tenant::ActiveModel {
            id: Set(tenant_id),
            name: Set(Some("Test Tenant".to_string())),
            created_at: Set(chrono::Utc::now().fixed_offset()),
        };
        tenant.insert(&state.db).await.unwrap();
    }

    async fn create_test_connection(
        state: &AppState,
        tenant_id: Uuid,
        provider_slug: &str,
    ) -> Uuid {
        use sea_orm::{ActiveModelTrait, Set};

        // Ensure tenant exists first
        create_test_tenant(state, tenant_id).await;

        let connection = crate::models::connection::ActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            provider_slug: Set(provider_slug.to_string()),
            external_id: Set("test-external-id".to_string()),
            status: Set("active".to_string()),
            display_name: Set(Some("Test Connection".to_string())),
            access_token_ciphertext: Set(Some(vec![1, 2, 3, 4])),
            refresh_token_ciphertext: Set(None),
            expires_at: Set(None),
            scopes: Set(None),
            metadata: Set(None),
            created_at: Set(chrono::Utc::now().fixed_offset()),
            updated_at: Set(chrono::Utc::now().fixed_offset()),
        };
        let result = connection.insert(&state.db).await.unwrap();
        result.id
    }

    #[tokio::test]
    async fn test_webhook_ingest_accepts_known_provider() {
        let (state, app) = setup_test_app().await;
        create_test_provider(&state, "github").await;

        let tenant_id = Uuid::new_v4();
        let request = Request::builder()
            .method("POST")
            .uri("/webhooks/github")
            .header("Authorization", "Bearer test-token")
            .header("X-Tenant-Id", tenant_id.to_string())
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"event": "push"}"#))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        println!("Response status: {}", response.status());
        assert_eq!(response.status(), StatusCode::ACCEPTED);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let webhook_response: WebhookAcceptResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(webhook_response.status, "accepted");
    }

    #[tokio::test]
    async fn test_webhook_ingest_returns_404_for_unknown_provider() {
        let (_state, app) = setup_test_app().await;

        let tenant_id = Uuid::new_v4();
        let request = Request::builder()
            .method("POST")
            .uri("/webhooks/unknown")
            .header("Authorization", "Bearer test-token")
            .header("X-Tenant-Id", tenant_id.to_string())
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_webhook_ingest_returns_401_without_auth() {
        let (_state, app) = setup_test_app().await;

        let tenant_id = Uuid::new_v4();
        let request = Request::builder()
            .method("POST")
            .uri("/webhooks/github")
            .header("X-Tenant-Id", tenant_id.to_string())
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_webhook_ingest_returns_400_without_tenant() {
        let (_state, app) = setup_test_app().await;

        let request = Request::builder()
            .method("POST")
            .uri("/webhooks/github")
            .header("Authorization", "Bearer test-token")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_webhook_ingest_returns_400_for_invalid_connection_id() {
        let (state, app) = setup_test_app().await;
        create_test_provider(&state, "github").await;

        let tenant_id = Uuid::new_v4();
        let request = Request::builder()
            .method("POST")
            .uri("/webhooks/github")
            .header("Authorization", "Bearer test-token")
            .header("X-Tenant-Id", tenant_id.to_string())
            .header("X-Connection-Id", "not-a-uuid")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_webhook_ingest_returns_404_for_invalid_connection() {
        let (state, app) = setup_test_app().await;
        create_test_provider(&state, "github").await;

        let tenant_id = Uuid::new_v4();
        let connection_id = Uuid::new_v4();
        let request = Request::builder()
            .method("POST")
            .uri("/webhooks/github")
            .header("Authorization", "Bearer test-token")
            .header("X-Tenant-Id", tenant_id.to_string())
            .header("X-Connection-Id", connection_id.to_string())
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_webhook_ingest_enqueues_job_with_valid_connection() {
        let (state, app) = setup_test_app().await;
        create_test_provider(&state, "github").await;

        let tenant_id = Uuid::new_v4();
        let connection_id = create_test_connection(&state, tenant_id, "github").await;

        let request = Request::builder()
            .method("POST")
            .uri("/webhooks/github")
            .header("Authorization", "Bearer test-token")
            .header("X-Tenant-Id", tenant_id.to_string())
            .header("X-Connection-Id", connection_id.to_string())
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"event": "push", "repo": "test"}"#))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::ACCEPTED);

        // Verify sync job was created
        let sync_job_repo = SyncJobRepository::new(state.db.clone());
        let jobs = sync_job_repo
            .list_by_tenant(
                tenant_id,
                Some("github".to_string()),
                None,
                Some(10),
                Some(0),
            )
            .await
            .unwrap();

        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].job_type, "webhook");
        assert_eq!(jobs[0].connection_id, connection_id);
        assert_eq!(jobs[0].provider_slug, "github");
        assert!(jobs[0].cursor.is_some());

        // Verify that sensitive headers are filtered from the cursor
        let cursor: &serde_json::Value = jobs[0].cursor.as_ref().unwrap();
        let webhook_headers = cursor.get("webhook_headers").unwrap().as_object().unwrap();

        // Should contain webhook-specific headers
        assert!(webhook_headers.contains_key("content-type"));
        assert!(webhook_headers.contains_key("x-connection-id"));

        // Should NOT contain sensitive headers
        assert!(!webhook_headers.contains_key("authorization"));
        assert!(!webhook_headers.contains_key("cookie"));

        // Verify payload is captured
        assert!(cursor.get("webhook_payload").is_some());
        assert!(cursor.get("received_at").is_some());
    }
}
