//! # API Handlers
//!
//! This module contains all the HTTP endpoint handlers for the Connectors API.

pub mod providers;

use crate::auth::{OperatorAuth, TenantExtension, TenantHeader};
use crate::error::ApiError;
use crate::models::ServiceInfo;
use crate::server::AppState;
use axum::{extract::State, response::Json};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Root handler that returns basic service information
#[utoipa::path(
    get,
    path = "/",
    responses(
        (status = 200, description = "Service information", body = ServiceInfo),
        (status = 500, description = "Internal server error", body = ApiError)
    ),
    tag = "root"
)]
pub async fn root(State(_state): State<AppState>) -> Result<Json<ServiceInfo>, ApiError> {
    // In the future, we could add database health check here
    // For now, just return basic service info
    Ok(Json(ServiceInfo::default()))
}

/// Health check response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
    /// Service health status
    pub status: String,
    /// Current timestamp
    pub timestamp: String,
}

impl Default for HealthResponse {
    fn default() -> Self {
        Self {
            status: "healthy".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// Health check endpoint (public, no auth required)
#[utoipa::path(
    get,
    path = "/healthz",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse),
        (status = 503, description = "Service is unhealthy", body = ApiError)
    ),
    tag = "health"
)]
pub async fn health(State(_state): State<AppState>) -> Result<Json<HealthResponse>, ApiError> {
    // In the future, we could add database health checks here
    // For now, just return healthy status
    Ok(Json(HealthResponse::default()))
}

/// Readiness check endpoint (public, no auth required)
#[utoipa::path(
    get,
    path = "/readyz",
    responses(
        (status = 200, description = "Service is ready", body = HealthResponse),
        (status = 503, description = "Service is not ready", body = ApiError)
    ),
    tag = "health"
)]
pub async fn ready(State(_state): State<AppState>) -> Result<Json<HealthResponse>, ApiError> {
    // In the future, we could add dependency checks here
    // For now, just return ready status
    Ok(Json(HealthResponse::default()))
}

/// Response payload for protected ping endpoint
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ProtectedPingResponse {
    /// Static message confirming protected endpoint access
    pub message: String,
    /// Tenant identifier echoed back to the caller
    pub tenant_id: String,
}

/// Protected ping endpoint requiring operator auth and tenant header
#[utoipa::path(
    get,
    path = "/protected/ping",
    security(("bearer_auth" = [])),
    params(TenantHeader),
    responses(
        (status = 200, description = "Authenticated operator ping", body = ProtectedPingResponse),
        (status = 400, description = "Tenant header missing or invalid", body = ApiError),
        (status = 401, description = "Missing or invalid bearer token", body = ApiError)
    ),
    tag = "operators"
)]
pub async fn protected_ping(
    OperatorAuth: OperatorAuth,
    TenantExtension(tenant): TenantExtension,
) -> Result<Json<ProtectedPingResponse>, ApiError> {
    let response = ProtectedPingResponse {
        message: "pong".to_string(),
        tenant_id: tenant.0.to_string(),
    };

    Ok(Json(response))
}

#[cfg(test)]
mod tests;
