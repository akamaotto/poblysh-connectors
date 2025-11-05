//! Configuration endpoint handlers
//!
//! These handlers expose configuration information for operational visibility.

use axum::{extract::State, http::StatusCode, response::Json};
use serde_json::Value;

use crate::config::RateLimitPolicyConfig;
use crate::error::ApiError;
use crate::server::AppState;

/// Get rate limit policy configuration
///
/// Returns the current rate limit policy configuration including provider overrides.
/// This endpoint is read-only and provides operational visibility into rate limiting behavior.
#[utoipa::path(
    get,
    path = "/config/rate-limit-policy",
    responses(
        (status = 200, description = "Rate limit policy configuration", body = RateLimitPolicyConfig),
        (status = 500, description = "Internal server error", body = ApiError)
    ),
    tag = "configuration"
)]
pub async fn get_rate_limit_policy_config(
    State(_state): State<AppState>,
) -> Result<Json<RateLimitPolicyConfig>, StatusCode> {
    // For now, return the default configuration. In a real implementation,
    // we would want to pass the actual config through the AppState.
    Ok(Json(RateLimitPolicyConfig::default()))
}

/// Get service configuration summary
///
/// Returns a summary of the current service configuration for operational purposes.
/// Excludes sensitive information like encryption keys and tokens.
#[utoipa::path(
    get,
    path = "/config/summary",
    responses(
        (status = 200, description = "Service configuration summary", body = Value),
        (status = 500, description = "Internal server error", body = ApiError)
    ),
    tag = "configuration"
)]
pub async fn get_config_summary(State(_state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    // Return a basic summary. In a real implementation, this would include
    // non-sensitive configuration from the actual AppConfig.
    let summary = serde_json::json!({
        "rate_limit_policy": RateLimitPolicyConfig::default(),
        "endpoints": {
            "swagger_ui": "/docs",
            "openapi_spec": "/docs/api.json"
        }
    });

    Ok(Json(summary))
}
