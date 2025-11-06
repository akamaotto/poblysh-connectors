//! # Authentication and Authorization
//!
//! This module provides operator bearer authentication and tenant header validation
//! for protected API endpoints.

use std::sync::Arc;

use axum::{
    extract::{FromRef, FromRequestParts, Request, State},
    http::{HeaderMap, header::AUTHORIZATION, request::Parts},
    middleware::Next,
    response::Response,
};
use serde::{Deserialize, Serialize};
use subtle::ConstantTimeEq;
use utoipa::IntoParams;
use uuid::Uuid;

use crate::config::AppConfig;
use crate::error::{ApiError, unauthorized, unauthorized_with_trace_id, validation_error};
use crate::server::AppState;
use crate::telemetry::TraceContext;

/// Tenant ID wrapper for type safety
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TenantId(pub Uuid);

/// Marker type for authenticated operator requests
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OperatorAuth;

/// Extractor for tenant ID from request extensions
#[derive(Debug, Clone)]
pub struct TenantExtension(pub TenantId);

impl FromRef<AppState> for Arc<AppConfig> {
    fn from_ref(app_state: &AppState) -> Self {
        Arc::clone(&app_state.config)
    }
}

/// Authentication middleware that validates bearer tokens and tenant headers
pub async fn auth_middleware(
    State(config): State<Arc<AppConfig>>,
    request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let headers = request.headers().clone();

    // Extract trace_id from request context for consistent error responses
    let trace_id = request
        .extensions()
        .get::<TraceContext>()
        .map(|ctx| ctx.trace_id.clone());

    let token = extract_bearer_token_with_trace_id(&headers, trace_id.clone())?;
    validate_token(&config, token)?;

    let tenant = extract_tenant_id_with_trace_id(&headers, trace_id)?;
    tracing::info!(tenant_id = %tenant.0, "Authenticated operator request");

    let mut request = request;
    request.extensions_mut().insert(TenantExtension(tenant));
    request.extensions_mut().insert(OperatorAuth);

    Ok(next.run(request).await)
}

fn extract_bearer_token_with_trace_id(
    headers: &HeaderMap,
    trace_id: Option<String>,
) -> Result<&str, ApiError> {
    let trace_id_clone = trace_id.clone();

    headers
        .get(AUTHORIZATION)
        .ok_or_else(|| {
            if let Some(trace_id_val) = trace_id_clone {
                unauthorized_with_trace_id(Some("Missing Authorization header"), trace_id_val)
            } else {
                unauthorized(Some("Missing Authorization header"))
            }
        })
        .and_then(|value| {
            let trace_id_clone2 = trace_id.clone();
            value.to_str().map_err(|_| {
                if let Some(trace_id_val) = trace_id_clone2 {
                    unauthorized_with_trace_id(Some("Invalid Authorization header"), trace_id_val)
                } else {
                    unauthorized(Some("Invalid Authorization header"))
                }
            })
        })
        .and_then(|header| {
            header.strip_prefix("Bearer ").ok_or_else(|| {
                if let Some(trace_id_val) = trace_id {
                    unauthorized_with_trace_id(
                        Some("Authorization header must use Bearer scheme"),
                        trace_id_val,
                    )
                } else {
                    unauthorized(Some("Authorization header must use Bearer scheme"))
                }
            })
        })
}

fn validate_token(config: &AppConfig, token: &str) -> Result<(), ApiError> {
    let is_valid = config
        .operator_tokens
        .iter()
        .any(|configured| ConstantTimeEq::ct_eq(token.as_bytes(), configured.as_bytes()).into());

    if is_valid {
        Ok(())
    } else {
        Err(unauthorized(Some("Invalid bearer token")))
    }
}

fn extract_tenant_id_with_trace_id(
    headers: &HeaderMap,
    _trace_id: Option<String>,
) -> Result<TenantId, ApiError> {
    let header_value = headers
        .get("X-Tenant-Id")
        .ok_or_else(|| {
            validation_error(
                "Missing required header",
                serde_json::json!({ "X-Tenant-Id": "Required header is missing" }),
            )
        })?
        .to_str()
        .map_err(|_| {
            validation_error(
                "Invalid tenant header",
                serde_json::json!({ "X-Tenant-Id": "Header must be valid UTF-8" }),
            )
        })?;

    header_value.parse::<Uuid>().map(TenantId).map_err(|_| {
        validation_error(
            "Invalid tenant ID",
            serde_json::json!({ "X-Tenant-Id": "Must be a valid UUID" }),
        )
    })
}

/// Extractor for tenant ID from request extensions
pub fn extract_tenant(ext: &TenantExtension) -> &TenantId {
    &ext.0
}

/// OpenAPI header parameter for X-Tenant-Id
#[derive(Debug, Serialize, Deserialize, IntoParams, utoipa::ToSchema)]
#[into_params(parameter_in = Header)]
pub struct TenantHeader {
    /// Tenant identifier (UUID) that scopes the request to a specific tenant
    #[serde(rename = "X-Tenant-Id")]
    #[param(rename = "X-Tenant-Id", value_type = String)]
    pub tenant_id: String,
}

impl<S> FromRequestParts<S> for TenantExtension
where
    Arc<AppConfig>: FromRef<S>,
    S: Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<TenantExtension>()
            .cloned()
            .ok_or_else(|| {
                validation_error(
                    "Tenant context missing",
                    serde_json::json!({ "X-Tenant-Id": "Tenant context not present" }),
                )
            })
    }
}

impl<S> FromRequestParts<S> for OperatorAuth
where
    Arc<AppConfig>: FromRef<S>,
    S: Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<OperatorAuth>()
            .copied()
            .ok_or_else(|| unauthorized(Some("Operator authentication required")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode},
        routing::get,
    };
    use tower::ServiceExt;

    fn create_test_config() -> Arc<AppConfig> {
        Arc::new(AppConfig {
            operator_tokens: vec!["test-token-123".to_string()],
            ..Default::default()
        })
    }

    async fn run_middleware(config: Arc<AppConfig>, request: Request<Body>) -> Response {
        async fn handler() -> &'static str {
            "OK"
        }

        Router::new()
            .route("/test", get(handler))
            .layer(axum::middleware::from_fn_with_state(
                Arc::clone(&config),
                auth_middleware,
            ))
            .with_state({
                let db = sea_orm::DatabaseConnection::default();
                let crypto_key =
                    crate::crypto::CryptoKey::new(vec![0u8; 32]).expect("Failed to create test crypto key");

                // Create required dependencies for TokenRefreshService
                let connection_repo = crate::repositories::ConnectionRepository::new(
                    std::sync::Arc::new(db.clone()),
                    crypto_key.clone(),
                );

                // Create TokenRefreshService
                let token_refresh_service = std::sync::Arc::new(crate::token_refresh::TokenRefreshService::new(
                    config.clone(),
                    std::sync::Arc::new(db.clone()),
                    std::sync::Arc::new(connection_repo),
                    crate::connectors::registry::Registry::new(),
                ));

                AppState { config, db, crypto_key, token_refresh_service }
            })
            .oneshot(request)
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn missing_auth_header_returns_401() {
        let config = create_test_config();
        let request = Request::builder()
            .uri("/test")
            .header("X-Tenant-Id", Uuid::new_v4().to_string())
            .body(Body::empty())
            .unwrap();

        let response = run_middleware(config, request).await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn invalid_auth_scheme_returns_401() {
        let config = create_test_config();
        let request = Request::builder()
            .uri("/test")
            .header("Authorization", "Basic dGVzdDoxMjM=")
            .header("X-Tenant-Id", Uuid::new_v4().to_string())
            .body(Body::empty())
            .unwrap();

        let response = run_middleware(config, request).await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn invalid_token_returns_401() {
        let config = create_test_config();
        let request = Request::builder()
            .uri("/test")
            .header("Authorization", "Bearer wrong-token")
            .header("X-Tenant-Id", Uuid::new_v4().to_string())
            .body(Body::empty())
            .unwrap();

        let response = run_middleware(config, request).await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn missing_tenant_header_returns_400() {
        let config = create_test_config();
        let request = Request::builder()
            .uri("/test")
            .header("Authorization", "Bearer test-token-123")
            .body(Body::empty())
            .unwrap();

        let response = run_middleware(config, request).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn invalid_tenant_uuid_returns_400() {
        let config = create_test_config();
        let request = Request::builder()
            .uri("/test")
            .header("Authorization", "Bearer test-token-123")
            .header("X-Tenant-Id", "not-a-uuid")
            .body(Body::empty())
            .unwrap();

        let response = run_middleware(config, request).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn valid_request_passes_through() {
        let config = create_test_config();
        let request = Request::builder()
            .uri("/test")
            .header("Authorization", "Bearer test-token-123")
            .header("X-Tenant-Id", Uuid::new_v4().to_string())
            .body(Body::empty())
            .unwrap();

        let response = run_middleware(config, request).await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn multiple_tokens_supported() {
        let config = Arc::new(AppConfig {
            operator_tokens: vec![
                "token-one".to_string(),
                "token-two".to_string(),
                "token-three".to_string(),
            ],
            ..Default::default()
        });

        for candidate in ["token-one", "token-two", "token-three"] {
            let request = Request::builder()
                .uri("/test")
                .header("Authorization", format!("Bearer {}", candidate))
                .header("X-Tenant-Id", Uuid::new_v4().to_string())
                .body(Body::empty())
                .unwrap();

            let response = run_middleware(Arc::clone(&config), request).await;
            assert_eq!(response.status(), StatusCode::OK);
        }
    }
}
