//! # Connections API Handlers
//!
//! This module contains handlers for managing connection listings,
//! including tenant-scoped connection listing with optional provider filtering.

use crate::auth::{OperatorAuth, TenantExtension, TenantHeader};
use crate::cursor::decode_generic_cursor;
use crate::error::ApiError;
use crate::repositories::connection::ConnectionRepository;
use crate::repositories::provider::ProviderRepository;
use crate::server::AppState;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

/// Query parameters for connections listing
#[derive(Debug, Deserialize, Serialize, IntoParams, ToSchema)]
pub struct ListConnectionsQuery {
    /// Optional provider filter (snake_case slug, e.g., "github")
    pub provider: Option<String>,
    /// Maximum number of connections to return (default: 50, max: 100)
    pub limit: Option<i64>,
    /// Opaque cursor for pagination continuation
    pub cursor: Option<String>,
}

/// Connection information for API responses
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ConnectionInfo {
    /// Unique identifier for the connection
    #[schema(value_type = String)]
    pub id: Uuid,
    /// Provider slug (e.g., "github", "slack")
    pub provider: String,
    /// Optional expiration timestamp for the connection
    pub expires_at: Option<String>,
    /// Provider-specific metadata
    pub metadata: serde_json::Value,
    /// Indicates whether an encrypted access token is stored
    #[schema(default = false, example = true)]
    pub has_access_token: bool,
    /// Indicates whether an encrypted refresh token is stored
    #[schema(default = false, example = true)]
    pub has_refresh_token: bool,
    /// Version of encryption format used for stored tokens
    #[schema(default = 1, example = 1)]
    pub token_encryption_version: u8,
}

impl From<crate::models::connection::Model> for ConnectionInfo {
    fn from(model: crate::models::connection::Model) -> Self {
        Self {
            id: model.id,
            provider: model.provider_slug,
            expires_at: model.expires_at.map(|dt| {
                // Convert DateTimeWithTimeZone to RFC3339 string
                let utc_dt: DateTime<Utc> = dt.naive_utc().and_utc();
                utc_dt.to_rfc3339()
            }),
            metadata: model.metadata.unwrap_or_default(),
            // Check if encrypted tokens exist
            has_access_token: model.access_token_ciphertext.is_some(),
            has_refresh_token: model.refresh_token_ciphertext.is_some(),
            // Default to version 1 for current encrypted format
            token_encryption_version: 1,
        }
    }
}

/// Response wrapper for connections listing
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ConnectionsResponse {
    /// List of connections for the tenant
    pub connections: Vec<ConnectionInfo>,
    /// Opaque cursor for fetching the next page (null if this is the last page)
    pub next_cursor: Option<String>,
}

/// Lists connections for the authenticated tenant with optional provider filtering
#[utoipa::path(
    get,
    path = "/connections",
    security(("bearer_auth" = [])),
    params(TenantHeader, ListConnectionsQuery),
    responses(
        (status = 200, description = "List of tenant connections", body = ConnectionsResponse, example = json!({
            "connections": [
                {
                    "id": "550e8400-e29b-41d4-a716-446655440000",
                    "provider": "github",
                    "expires_at": "2024-12-31T23:59:59Z",
                    "metadata": {"login": "user123"},
                    "has_access_token": true,
                    "has_refresh_token": true,
                    "token_encryption_version": 1
                }
            ],
            "next_cursor": null
        })),
        (status = 400, description = "Validation error", body = ApiError),
        (status = 401, description = "Unauthorized", body = ApiError)
    ),
    tag = "operators"
)]
pub async fn list_connections(
    State(state): State<AppState>,
    _operator_auth: OperatorAuth,
    TenantExtension(tenant): TenantExtension,
    Query(query): Query<ListConnectionsQuery>,
) -> Result<Json<ConnectionsResponse>, ApiError> {
    // Validate and parse limit
    let limit = query.limit.unwrap_or(50);
    if !(1..=100).contains(&limit) {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            "VALIDATION_FAILED",
            "limit must be between 1 and 100",
        ));
    }

    // Validate cursor if provided
    if let Some(ref cursor_str) = query.cursor {
        // Try to decode the cursor to validate format
        decode_generic_cursor(cursor_str).map_err(|_| {
            ApiError::new(
                StatusCode::BAD_REQUEST,
                "VALIDATION_FAILED",
                "cursor is not valid base64-encoded JSON",
            )
        })?;
    }

    let connection_repo =
        ConnectionRepository::new(Arc::new(state.db.clone()), state.crypto_key.clone());
    let provider_repo = ProviderRepository::new(Arc::new(state.db.clone()));

    let (connections, next_cursor) = match query.provider {
        Some(provider_slug) => {
            // Validate provider exists in registry
            if provider_repo.find_by_slug(&provider_slug).await?.is_none() {
                return Err(ApiError::new(
                    StatusCode::BAD_REQUEST,
                    "VALIDATION_FAILED",
                    "unknown provider",
                ));
            }

            // Filter by tenant and provider with pagination
            connection_repo
                .list_by_tenant_provider(&tenant.0, &provider_slug, limit as u64, query.cursor)
                .await?
        }
        None => {
            // Get all connections for tenant with pagination
            connection_repo
                .list_by_tenant(&tenant.0, limit as u64, query.cursor)
                .await?
        }
    };

    let connection_infos: Vec<ConnectionInfo> =
        connections.into_iter().map(ConnectionInfo::from).collect();

    Ok(Json(ConnectionsResponse {
        connections: connection_infos,
        next_cursor,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::auth_middleware;
    use crate::config::AppConfig;
    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode},
        routing::get,
    };
    use std::sync::Arc;
    use tower::ServiceExt;

    async fn create_test_app() -> (Arc<AppConfig>, axum::Router) {
        let config = Arc::new(AppConfig {
            operator_tokens: vec!["test-token-123".to_string()],
            crypto_key: Some(vec![0u8; 32]), // Test key
            ..Default::default()
        });

        // Create a simple test state - we'll test the logic without requiring a full database
        let _crypto_key =
            crate::crypto::CryptoKey::new(vec![0u8; 32]).expect("Failed to create test crypto key");
        let state = crate::server::create_test_app_state(
            (*config).clone(),
            sea_orm::Database::connect("sqlite::memory:").await.unwrap(),
        );

        let app = Router::new()
            .route("/connections", get(list_connections))
            .layer(axum::middleware::from_fn_with_state(
                Arc::clone(&config),
                auth_middleware,
            ))
            .with_state(state);

        (config, app)
    }

    #[tokio::test]
    async fn list_connections_unauthorized_without_token() {
        let (_config, app) = create_test_app().await;

        let request = Request::builder()
            .uri("/connections")
            .header("X-Tenant-Id", uuid::Uuid::new_v4().to_string())
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn list_connections_missing_tenant_header() {
        let (_config, app) = create_test_app().await;

        let request = Request::builder()
            .uri("/connections")
            .header("Authorization", "Bearer test-token-123")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn list_connections_invalid_token() {
        let (_config, app) = create_test_app().await;

        let request = Request::builder()
            .uri("/connections")
            .header("Authorization", "Bearer invalid-token")
            .header("X-Tenant-Id", uuid::Uuid::new_v4().to_string())
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_connection_info_serialization() {
        let connection_info = ConnectionInfo {
            id: uuid::Uuid::new_v4(),
            provider: "github".to_string(),
            expires_at: Some("2024-12-31T23:59:59Z".to_string()),
            metadata: serde_json::json!({"user": "test"}),
            has_access_token: true,
            has_refresh_token: true,
            token_encryption_version: 1,
        };

        let json = serde_json::to_string(&connection_info).unwrap();
        let parsed: ConnectionInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.id, connection_info.id);
        assert_eq!(parsed.provider, connection_info.provider);
        assert_eq!(parsed.expires_at, connection_info.expires_at);
        assert_eq!(parsed.metadata, connection_info.metadata);
    }

    #[tokio::test]
    async fn test_connections_response_serialization() {
        let connections = vec![ConnectionInfo {
            id: uuid::Uuid::new_v4(),
            provider: "github".to_string(),
            expires_at: None,
            metadata: serde_json::json!({}),
            has_access_token: false,
            has_refresh_token: false,
            token_encryption_version: 1,
        }];

        let response = ConnectionsResponse {
            connections,
            next_cursor: None,
        };
        let json = serde_json::to_string(&response).unwrap();
        let parsed: ConnectionsResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.connections.len(), 1);
        assert_eq!(parsed.connections[0].provider, "github");
    }

    #[tokio::test]
    async fn test_list_connections_query_deserialization() {
        // Test with provider parameter
        let query = ListConnectionsQuery {
            provider: Some("github".to_string()),
            limit: None,
            cursor: None,
        };
        let json = serde_json::to_string(&query).unwrap();
        let parsed: ListConnectionsQuery = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.provider, Some("github".to_string()));

        // Test without provider parameter
        let query = ListConnectionsQuery {
            provider: None,
            limit: None,
            cursor: None,
        };
        let json = serde_json::to_string(&query).unwrap();
        let parsed: ListConnectionsQuery = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.provider, None);
    }

    #[tokio::test]
    async fn test_connections_response_includes_next_cursor_field() {
        // Test that the response JSON always includes the next_cursor field, even when null
        let connections = vec![ConnectionInfo {
            id: Uuid::new_v4(),
            provider: "github".to_string(),
            expires_at: None,
            metadata: serde_json::json!({}),
            has_access_token: false,
            has_refresh_token: false,
            token_encryption_version: 1,
        }];

        // Test response with null next_cursor (final page)
        let response = ConnectionsResponse {
            connections,
            next_cursor: None,
        };
        let json = serde_json::to_string(&response).unwrap();

        // Verify the JSON contains the next_cursor field
        assert!(json.contains("next_cursor"));
        assert!(json.contains("null"));

        // Verify we can parse it back and the field is present
        let parsed: ConnectionsResponse = serde_json::from_str(&json).unwrap();
        assert!(parsed.next_cursor.is_none());

        // Test response with non-null next_cursor (more pages available)
        let response_with_cursor = ConnectionsResponse {
            connections: vec![],
            next_cursor: Some("eyJ2ZXJzaW9uIjoxLCJrZXlzIjp7Im5hbWUiOiJnaXRodWIifX0=".to_string()),
        };
        let json_with_cursor = serde_json::to_string(&response_with_cursor).unwrap();

        // Verify the JSON contains the next_cursor field with a value
        assert!(json_with_cursor.contains("next_cursor"));
        assert!(!json_with_cursor.contains("null"));

        // Verify we can parse it back and the field is present with value
        let parsed_with_cursor: ConnectionsResponse =
            serde_json::from_str(&json_with_cursor).unwrap();
        assert!(parsed_with_cursor.next_cursor.is_some());
    }
}
