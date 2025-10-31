//! # API Handlers
//!
//! This module contains all the HTTP endpoint handlers for the Connectors API.

use axum::response::Json;
use crate::models::ServiceInfo;

/// Root handler that returns basic service information
#[utoipa::path(
    get,
    path = "/",
    responses(
        (status = 200, description = "Service information", body = ServiceInfo)
    ),
    tag = "root"
)]
pub async fn root() -> Json<ServiceInfo> {
    Json(ServiceInfo::default())
}

#[cfg(test)]
mod tests;