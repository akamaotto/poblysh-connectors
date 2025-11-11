//! # Tenants API Handlers
//!
//! This module contains handlers for tenant creation and management endpoints.

use crate::auth::{OperatorAuth, TenantExtension};
use crate::error::ApiError;
use crate::repositories::{CreateTenantRequest, TenantRepository};
use crate::server::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Request payload for creating a new tenant
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateTenantRequestDto {
    /// Display name for the tenant (required, max 255 characters)
    #[schema(example = "Acme Corp")]
    pub name: String,
    /// Optional metadata for the tenant
    pub metadata: Option<serde_json::Value>,
}

/// Response payload for tenant creation
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateTenantResponseDto {
    /// Unique identifier for the tenant (UUID)
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub id: String,
    /// Display name of the tenant
    #[schema(example = "Acme Corp")]
    pub name: String,
    /// Timestamp when the tenant was created (ISO 8601)
    #[schema(example = "2024-01-15T10:30:00Z")]
    pub created_at: String,
    /// Optional metadata for the tenant
    pub metadata: Option<serde_json::Value>,
}

/// Standard API response wrapper for tenant operations
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TenantApiResponse<T> {
    /// Response data
    pub data: T,
    /// Response metadata
    pub meta: TenantResponseMeta,
}

/// Response metadata
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TenantResponseMeta {
    /// Unique request identifier for tracing
    #[schema(example = "req-1705319400-abc123def")]
    pub request_id: String,
    /// Response timestamp (ISO 8601)
    #[schema(example = "2024-01-15T10:30:00Z")]
    pub timestamp: String,
}

/// Create a new tenant
#[utoipa::path(
    post,
    path = "/api/v1/tenants",
    security(("bearer_auth" = [])),
    request_body = CreateTenantRequestDto,
    responses(
        (status = 201, description = "Tenant created successfully", body = TenantApiResponse<CreateTenantResponseDto>, headers(
            ("Location", description = "URL of the created tenant"),
            ("X-Trace-Id", description = "Trace identifier for request correlation")
        )),
        (status = 400, description = "Validation failed", body = ApiError),
        (status = 401, description = "Missing or invalid bearer token", body = ApiError),
        (status = 403, description = "Insufficient permissions", body = ApiError),
        (status = 409, description = "Conflict - tenant already exists", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    ),
    tag = "tenants"
)]
pub async fn create_tenant(
    State(state): State<AppState>,
    _operator_auth: OperatorAuth,
    TenantExtension(_tenant): TenantExtension,
    Json(request): Json<CreateTenantRequestDto>,
) -> Result<
    (
        StatusCode,
        [(&'static str, String); 2],
        Json<TenantApiResponse<CreateTenantResponseDto>>,
    ),
    ApiError,
> {
    let trace_id = Uuid::new_v4().to_string();

    // Validate request
    if request.name.trim().is_empty() {
        let mut api_err = ApiError::new(
            StatusCode::BAD_REQUEST,
            "VALIDATION_FAILED",
            "Tenant name is required and cannot be empty",
        );
        api_err.details = Some(Box::new(serde_json::json!({
            "field": "name",
            "message": "Tenant name must be provided and cannot be empty"
        })));
        return Err(api_err);
    }

    if request.name.len() > 255 {
        let mut api_err = ApiError::new(
            StatusCode::BAD_REQUEST,
            "VALIDATION_FAILED",
            "Tenant name exceeds maximum length",
        );
        api_err.details = Some(Box::new(serde_json::json!({
            "field": "name",
            "message": "Tenant name cannot exceed 255 characters",
            "max_length": 255,
            "actual_length": request.name.len()
        })));
        return Err(api_err);
    }

    // Create repository and tenant
    let repo = TenantRepository::new(&state.db);
    let create_request = CreateTenantRequest {
        name: request.name.trim().to_string(),
        metadata: request.metadata,
    };

    let tenant = repo.create_tenant(create_request).await.map_err(|e| {
        let mut api_err = ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "INTERNAL_SERVER_ERROR",
            "Failed to create tenant",
        );
        api_err.details = Some(Box::new(serde_json::json!({
            "repository_error": e.to_string()
        })));
        api_err
    })?;

    let response_data = CreateTenantResponseDto {
        id: tenant.id.to_string(),
        name: tenant.name.unwrap_or_default(),
        created_at: tenant.created_at.to_rfc3339(),
        metadata: None, // Currently not stored in tenant model
    };

    let response = TenantApiResponse {
        data: response_data,
        meta: TenantResponseMeta {
            request_id: trace_id.clone(),
            timestamp: Utc::now().to_rfc3339(),
        },
    };

    let location_header = format!("/api/v1/tenants/{}", tenant.id);

    Ok((
        StatusCode::CREATED,
        [("Location", location_header), ("X-Trace-Id", trace_id)],
        Json(response),
    ))
}

/// Get a tenant by ID
#[utoipa::path(
    get,
    path = "/api/v1/tenants/{id}",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "Tenant UUID")
    ),
    responses(
        (status = 200, description = "Tenant retrieved successfully", body = TenantApiResponse<CreateTenantResponseDto>),
        (status = 401, description = "Missing or invalid bearer token", body = ApiError),
        (status = 403, description = "Insufficient permissions", body = ApiError),
        (status = 404, description = "Tenant not found", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    ),
    tag = "tenants"
)]
pub async fn get_tenant(
    State(state): State<AppState>,
    _operator_auth: OperatorAuth,
    TenantExtension(_tenant): TenantExtension,
    Path(tenant_id): Path<Uuid>,
) -> Result<Json<TenantApiResponse<CreateTenantResponseDto>>, ApiError> {
    let trace_id = Uuid::new_v4().to_string();

    // Create repository and fetch tenant
    let repo = TenantRepository::new(&state.db);

    let tenant = repo
        .get_tenant_by_id(tenant_id)
        .await
        .map_err(|e| {
            let mut api_err = ApiError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_SERVER_ERROR",
                "Failed to retrieve tenant",
            );
            api_err.details = Some(Box::new(serde_json::json!({
                "repository_error": e.to_string()
            })));
            api_err
        })?
        .ok_or_else(|| {
            let mut api_err = ApiError::new(
                StatusCode::NOT_FOUND,
                "TENANT_NOT_FOUND",
                "Tenant not found",
            );
            api_err.details = Some(Box::new(serde_json::json!({
                "tenant_id": tenant_id.to_string()
            })));
            api_err
        })?;

    let response_data = CreateTenantResponseDto {
        id: tenant.id.to_string(),
        name: tenant.name.unwrap_or_default(),
        created_at: tenant.created_at.to_rfc3339(),
        metadata: None, // Currently not stored in tenant model
    };

    let response = TenantApiResponse {
        data: response_data,
        meta: TenantResponseMeta {
            request_id: trace_id,
            timestamp: Utc::now().to_rfc3339(),
        },
    };

    Ok(Json(response))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;
    use crate::db::init_pool;
    use crate::repositories::TenantRepository;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use serde_json::json;
    use tower::ServiceExt;
    use uuid::Uuid;

    async fn setup_test_app() -> (AppState, axum::Router) {
        let config = AppConfig {
            profile: "test".to_string(),
            operator_tokens: vec!["test-token".to_string()],
            ..Default::default()
        };

        let db = init_pool(&config).await.expect("Failed to init test DB");
        let state = crate::server::create_test_app_state(config, db.clone());

        let app = crate::server::create_app(state.clone());
        (state, app)
    }

    fn create_auth_headers() -> Vec<(&'static str, &'static str)> {
        vec![
            ("Authorization", "Bearer test-token"),
            ("X-Tenant-Id", "550e8400-e29b-41d4-a716-446655440000"),
            ("Content-Type", "application/json"),
        ]
    }

    #[tokio::test]
    async fn test_create_tenant_success() {
        let (_state, app) = setup_test_app().await;

        let request_body = json!({
            "name": "Test Tenant",
            "metadata": {
                "environment": "test"
            }
        });

        let mut builder = Request::builder().method("POST").uri("/api/v1/tenants");

        for (name, value) in create_auth_headers() {
            builder = builder.header(name, value);
        }

        let request = builder.body(Body::from(request_body.to_string())).unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);

        // Check Location header
        let location = response.headers().get("Location").unwrap();
        assert!(location.to_str().unwrap().starts_with("/api/v1/tenants/"));

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let response_json: TenantApiResponse<CreateTenantResponseDto> =
            serde_json::from_slice(&body).unwrap();

        assert!(!response_json.data.id.is_empty());
        assert_eq!(response_json.data.name, "Test Tenant");
        assert!(!response_json.data.created_at.is_empty());
        assert_eq!(response_json.meta.request_id.len(), 36); // UUID length
    }

    #[tokio::test]
    async fn test_create_tenant_validation_error() {
        let (_state, app) = setup_test_app().await;

        let request_body = json!({
            "name": "",  // Empty name
            "metadata": {}
        });

        let mut builder = Request::builder().method("POST").uri("/api/v1/tenants");

        for (name, value) in create_auth_headers() {
            builder = builder.header(name, value);
        }

        let request = builder.body(Body::from(request_body.to_string())).unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let error_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(error_json["code"], "VALIDATION_FAILED");
    }

    #[tokio::test]
    async fn test_get_tenant_success() {
        let (state, app) = setup_test_app().await;

        // First create a tenant
        let repo = TenantRepository::new(&state.db);
        let create_request = CreateTenantRequest {
            name: "Test Tenant for Get".to_string(),
            metadata: None,
        };
        let tenant = repo.create_tenant(create_request).await.unwrap();

        let mut builder = Request::builder()
            .method("GET")
            .uri(&format!("/api/v1/tenants/{}", tenant.id));

        for (name, value) in create_auth_headers() {
            builder = builder.header(name, value);
        }

        let request = builder.body(Body::empty()).unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let response_json: TenantApiResponse<CreateTenantResponseDto> =
            serde_json::from_slice(&body).unwrap();

        assert_eq!(response_json.data.id, tenant.id.to_string());
        assert_eq!(response_json.data.name, "Test Tenant for Get");
    }

    #[tokio::test]
    async fn test_get_tenant_not_found() {
        let (_state, app) = setup_test_app().await;

        let non_existent_id = Uuid::new_v4();
        let mut builder = Request::builder()
            .method("GET")
            .uri(&format!("/api/v1/tenants/{}", non_existent_id));

        for (name, value) in create_auth_headers() {
            builder = builder.header(name, value);
        }

        let request = builder.body(Body::empty()).unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let error_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(error_json["code"], "TENANT_NOT_FOUND");
        assert_eq!(
            error_json["details"]["tenant_id"],
            non_existent_id.to_string()
        );
    }
}
