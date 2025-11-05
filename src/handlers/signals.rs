//! # Signals Endpoint Handler
//!
//! This module contains the handler for the GET /signals endpoint,
//! which lists normalized signals with filters and cursor pagination.

use crate::auth::{OperatorAuth, TenantExtension};
use crate::cursor::{decode_cursor, encode_cursor};
use crate::error::ApiError;
use crate::repositories::SignalRepository;
use crate::server::AppState;
use axum::{extract::Query, extract::State, http::StatusCode, response::Json};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

/// Query parameters for listing signals
#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct ListSignalsQuery {
    /// Filter by provider slug
    pub provider: Option<String>,
    /// Filter by connection ID (UUID)
    pub connection_id: Option<String>,
    /// Filter by signal kind
    pub kind: Option<String>,
    /// Filter for signals that occurred after this timestamp (RFC3339)
    pub occurred_after: Option<String>,
    /// Filter for signals that occurred before this timestamp (RFC3339)
    pub occurred_before: Option<String>,
    /// Maximum number of signals to return (default: 50, max: 100)
    pub limit: Option<i64>,
    /// Opaque cursor for pagination continuation
    pub cursor: Option<String>,
    /// Whether to include the full payload (default: false)
    pub include_payload: Option<bool>,
}

/// Signal information for API responses
#[derive(Debug, Serialize, ToSchema)]
pub struct SignalInfo {
    /// Unique identifier for the signal
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub id: String,
    /// Slug of the provider that emitted this signal
    #[schema(example = "github")]
    pub provider_slug: String,
    /// Connection identifier that this signal originated from
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub connection_id: String,
    /// Normalized event kind
    #[schema(example = "issue_created")]
    pub kind: String,
    /// Timestamp when the event occurred in the provider system
    #[schema(example = "2024-01-15T10:30:00Z")]
    pub occurred_at: String,
    /// Timestamp when the signal was processed by the system
    #[schema(example = "2024-01-15T10:30:05Z")]
    pub received_at: String,
    /// Normalized event payload (only included when include_payload=true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<serde_json::Value>,
}

/// Response payload for signals endpoint
#[derive(Debug, Serialize, ToSchema)]
pub struct SignalsResponse {
    /// List of signals matching the query
    pub signals: Vec<SignalInfo>,
    /// Opaque cursor for fetching the next page (null if this is the last page)
    pub next_cursor: Option<String>,
}

/// List signals with filters and cursor pagination
#[utoipa::path(
    get,
    path = "/signals",
    security(("bearer_auth" = [])),
    params(ListSignalsQuery),
    responses(
        (status = 200, description = "Signals listed successfully", body = SignalsResponse),
        (status = 400, description = "Invalid query parameters", body = ApiError, example = json!({
            "status": 400,
            "code": "VALIDATION_FAILED",
            "message": "connection_id must be a valid UUID",
            "trace_id": "corr-12345678"
        })),
        (status = 401, description = "Missing or invalid bearer token", body = ApiError, example = json!({
            "status": 401,
            "code": "UNAUTHORIZED",
            "message": "Authentication required",
            "trace_id": "corr-87654321"
        })),
        (status = 403, description = "Forbidden access", body = ApiError, example = json!({
            "status": 403,
            "code": "FORBIDDEN",
            "message": "Insufficient permissions to access signals",
            "trace_id": "corr-11111111"
        })),
        (status = 429, description = "Rate limit exceeded", body = ApiError, example = json!({
            "status": 429,
            "code": "RATE_LIMITED",
            "message": "Too many requests",
            "retry_after": 60,
            "trace_id": "corr-22222222"
        })),
        (status = 500, description = "Internal server error", body = ApiError, example = json!({
            "status": 500,
            "code": "INTERNAL_SERVER_ERROR",
            "message": "An unexpected error occurred",
            "trace_id": "corr-33333333"
        })),
        (status = 502, description = "Provider service error", body = ApiError, example = json!({
            "status": 502,
            "code": "PROVIDER_ERROR",
            "message": "Provider github returned error status 503",
            "details": {
                "provider": "github",
                "status": 503,
                "body_snippet": "Service temporarily unavailable"
            },
            "trace_id": "corr-44444444"
        })),
        (status = 503, description = "Database unavailable", body = ApiError, example = json!({
            "status": 503,
            "code": "DATABASE_UNAVAILABLE",
            "message": "Database service temporarily unavailable",
            "trace_id": "corr-55555555"
        }))
    ),
    tag = "signals"
)]
pub async fn list_signals(
    State(state): State<AppState>,
    _operator_auth: OperatorAuth,
    TenantExtension(tenant): TenantExtension,
    Query(query): Query<ListSignalsQuery>,
) -> Result<Json<SignalsResponse>, ApiError> {
    // Validate and parse limit
    let limit = query.limit.unwrap_or(50);
    if limit < 1 || limit > 100 {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            "VALIDATION_FAILED",
            "limit must be between 1 and 100",
        ));
    }

    // Validate connection_id if provided
    let connection_id = if let Some(conn_id_str) = query.connection_id {
        match Uuid::from_str(&conn_id_str) {
            Ok(uuid) => Some(uuid),
            Err(_) => {
                return Err(ApiError::new(
                    StatusCode::BAD_REQUEST,
                    "VALIDATION_FAILED",
                    "connection_id must be a valid UUID",
                ));
            }
        }
    } else {
        None
    };

    // Validate timestamps if provided
    let occurred_after = if let Some(timestamp_str) = query.occurred_after {
        match DateTime::parse_from_rfc3339(&timestamp_str) {
            Ok(dt) => Some(dt.with_timezone(&Utc)),
            Err(_) => {
                return Err(ApiError::new(
                    StatusCode::BAD_REQUEST,
                    "VALIDATION_FAILED",
                    "occurred_after must be a valid RFC3339 timestamp",
                ));
            }
        }
    } else {
        None
    };

    let occurred_before = if let Some(timestamp_str) = query.occurred_before {
        match DateTime::parse_from_rfc3339(&timestamp_str) {
            Ok(dt) => Some(dt.with_timezone(&Utc)),
            Err(_) => {
                return Err(ApiError::new(
                    StatusCode::BAD_REQUEST,
                    "VALIDATION_FAILED",
                    "occurred_before must be a valid RFC3339 timestamp",
                ));
            }
        }
    } else {
        None
    };

    // Validate cursor if provided
    let cursor_data = if let Some(cursor_str) = query.cursor {
        match decode_cursor(&cursor_str) {
            Ok(data) => Some(data),
            Err(e) => {
                return Err(e);
            }
        }
    } else {
        None
    };

    let include_payload = query.include_payload.unwrap_or(false);

    // Use repository to list signals
    let signal_repo = SignalRepository::new(&state.db);
    let result = signal_repo
        .list_signals(
            tenant.0,
            query.provider,
            connection_id,
            query.kind,
            occurred_after,
            occurred_before,
            cursor_data,
            limit + 1, // Fetch one extra to determine if there are more results
            include_payload,
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to list signals: {}", e);
            match e {
                crate::error::RepositoryError::Database(db_err) => {
                    // Convert specific database errors to appropriate API errors
                    match db_err {
                        sea_orm::DbErr::Conn(_) => ApiError::new(
                            StatusCode::SERVICE_UNAVAILABLE,
                            "DATABASE_UNAVAILABLE",
                            "Database service temporarily unavailable",
                        ),
                        sea_orm::DbErr::Query(_) | sea_orm::DbErr::Exec(_) => ApiError::new(
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "QUERY_ERROR",
                            "Failed to process signal query",
                        ),
                        _ => ApiError::new(
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "INTERNAL_SERVER_ERROR",
                            "An unexpected database error occurred",
                        ),
                    }
                }
                crate::error::RepositoryError::NotFound(msg) => {
                    // This shouldn't happen for list operations, but handle it gracefully
                    ApiError::new(
                        StatusCode::NOT_FOUND,
                        "NOT_FOUND",
                        &format!("Requested resource not found: {}", msg),
                    )
                }
                crate::error::RepositoryError::Validation(msg) => {
                    // This shouldn't happen with our current validation, but handle it
                    ApiError::new(
                        StatusCode::BAD_REQUEST,
                        "VALIDATION_FAILED",
                        &format!("Query validation failed: {}", msg),
                    )
                }
            }
        })?;

    // Determine if there are more results and extract the signals to return
    let has_more = result.len() > limit as usize;
    let signals_to_return = if has_more {
        result.into_iter().take(limit as usize).collect()
    } else {
        result
    };

    // Generate next cursor if there are more results
    let next_cursor = if has_more {
        if let Some(last_signal) = signals_to_return.last() {
            Some(encode_cursor(
                &last_signal.occurred_at.with_timezone(&Utc),
                &last_signal.id,
            ))
        } else {
            None
        }
    } else {
        None
    };

    // Convert to API response format
    let signals: Vec<SignalInfo> = signals_to_return
        .into_iter()
        .map(|signal| SignalInfo {
            id: signal.id.to_string(),
            provider_slug: signal.provider_slug,
            connection_id: signal.connection_id.to_string(),
            kind: signal.kind,
            occurred_at: signal.occurred_at.to_rfc3339(),
            received_at: signal.received_at.to_rfc3339(),
            payload: if include_payload {
                Some(signal.payload)
            } else {
                None
            },
        })
        .collect();

    Ok(Json(SignalsResponse {
        signals,
        next_cursor,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;
    use crate::db::init_pool;
    use axum::{
        body::Body,
        http::{Request, StatusCode, header::AUTHORIZATION, header::HeaderValue},
    };
    use tower::ServiceExt;
    use uuid::Uuid;

    async fn setup_test_app() -> (AppState, axum::Router) {
        let config = AppConfig {
            profile: "test".to_string(),
            operator_tokens: vec!["test-token".to_string()],
            ..Default::default()
        };

        let db = init_pool(&config).await.expect("Failed to init test DB");
        let state = AppState {
            config: std::sync::Arc::new(config),
            db,
            crypto_key: crate::crypto::CryptoKey::new([0; 32].to_vec()).unwrap(),
        };

        let app = crate::server::create_app(state.clone());
        (state, app)
    }

    #[tokio::test]
    async fn test_list_signals_requires_auth() {
        let (_state, app) = setup_test_app().await;

        let request = Request::builder()
            .method("GET")
            .uri("/signals")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_list_signals_requires_tenant_header() {
        let (_state, app) = setup_test_app().await;

        let request = Request::builder()
            .method("GET")
            .uri("/signals")
            .header(AUTHORIZATION, HeaderValue::from_static("Bearer test-token"))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_cursor_encoding_decoding() {
        let occurred_at = Utc::now();
        let id = Uuid::new_v4();

        let cursor_str = encode_cursor(&occurred_at, &id);
        let decoded = decode_cursor(&cursor_str).unwrap();

        assert_eq!(decoded.occurred_at, occurred_at);
        assert_eq!(decoded.id, id);
    }

    #[tokio::test]
    async fn test_invalid_cursor_decoding() {
        let invalid_cursor = "invalid-base64!";
        let result = decode_cursor(invalid_cursor);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_invalid_connection_id_validation() {
        let config = AppConfig {
            profile: "test".to_string(),
            operator_tokens: vec!["test-token".to_string()],
            ..Default::default()
        };

        let db = init_pool(&config).await.expect("Failed to init test DB");
        let state = AppState {
            config: std::sync::Arc::new(config),
            db,
            crypto_key: crate::crypto::CryptoKey::new([0; 32].to_vec()).unwrap(),
        };

        let query = ListSignalsQuery {
            connection_id: Some("invalid-uuid".to_string()),
            provider: None,
            kind: None,
            occurred_after: None,
            occurred_before: None,
            limit: None,
            cursor: None,
            include_payload: None,
        };

        let result = list_signals(
            State(state),
            OperatorAuth,
            TenantExtension(crate::auth::TenantId(Uuid::new_v4())),
            Query(query),
        )
        .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.status, StatusCode::BAD_REQUEST);
        assert_eq!(err.code, "VALIDATION_FAILED".into());
    }

    #[tokio::test]
    async fn test_invalid_timestamp_validation() {
        let config = AppConfig {
            profile: "test".to_string(),
            operator_tokens: vec!["test-token".to_string()],
            ..Default::default()
        };

        let db = init_pool(&config).await.expect("Failed to init test DB");
        let state = AppState {
            config: std::sync::Arc::new(config),
            db,
            crypto_key: crate::crypto::CryptoKey::new([0; 32].to_vec()).unwrap(),
        };

        let query = ListSignalsQuery {
            connection_id: None,
            provider: None,
            kind: None,
            occurred_after: Some("not-a-timestamp".to_string()),
            occurred_before: None,
            limit: None,
            cursor: None,
            include_payload: None,
        };

        let result = list_signals(
            State(state),
            OperatorAuth,
            TenantExtension(crate::auth::TenantId(Uuid::new_v4())),
            Query(query),
        )
        .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.status, StatusCode::BAD_REQUEST);
        assert_eq!(err.code, "VALIDATION_FAILED".into());
    }

    #[tokio::test]
    async fn test_limit_validation() {
        let config = AppConfig {
            profile: "test".to_string(),
            operator_tokens: vec!["test-token".to_string()],
            ..Default::default()
        };

        let db = init_pool(&config).await.expect("Failed to init test DB");
        let state = AppState {
            config: std::sync::Arc::new(config),
            db,
            crypto_key: crate::crypto::CryptoKey::new([0; 32].to_vec()).unwrap(),
        };

        // Test limit too high
        let query = ListSignalsQuery {
            connection_id: None,
            provider: None,
            kind: None,
            occurred_after: None,
            occurred_before: None,
            limit: Some(101), // Over max
            cursor: None,
            include_payload: None,
        };

        let result = list_signals(
            State(state.clone()),
            OperatorAuth,
            TenantExtension(crate::auth::TenantId(Uuid::new_v4())),
            Query(query),
        )
        .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.status, StatusCode::BAD_REQUEST);
        assert_eq!(err.code, "VALIDATION_FAILED".into());

        // Test limit too low
        let query = ListSignalsQuery {
            connection_id: None,
            provider: None,
            kind: None,
            occurred_after: None,
            occurred_before: None,
            limit: Some(0), // Under min
            cursor: None,
            include_payload: None,
        };

        let result = list_signals(
            State(state),
            OperatorAuth,
            TenantExtension(crate::auth::TenantId(Uuid::new_v4())),
            Query(query),
        )
        .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.status, StatusCode::BAD_REQUEST);
        assert_eq!(err.code, "VALIDATION_FAILED".into());
    }
}
