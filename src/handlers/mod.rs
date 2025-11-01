//! # API Handlers
//!
//! This module contains all the HTTP endpoint handlers for the Connectors API.

use crate::models::ServiceInfo;
use crate::server::AppState;
use axum::{response::Json, extract::State};

/// Root handler that returns basic service information
#[utoipa::path(
    get,
    path = "/",
    responses(
        (status = 200, description = "Service information", body = ServiceInfo)
    ),
    tag = "root"
)]
pub async fn root(State(_state): State<AppState>) -> Json<ServiceInfo> {
    // In the future, we could add database health check here
    // For now, just return basic service info
    Json(ServiceInfo::default())
}

#[cfg(test)]
mod tests;
