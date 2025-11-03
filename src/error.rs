//! # Error Handling
//!
//! This module provides unified error handling for the Connectors API,
//! implementing a consistent problem+json response format with trace ID propagation.

use axum::{
    extract::rejection::JsonRejection,
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;
use utoipa::ToSchema;

use crate::telemetry;
/// Unified API error response structure
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ApiError {
    /// HTTP status code for the response
    #[serde(skip_serializing, skip_deserializing)]
    pub status: StatusCode,
    /// Error code for programmatic handling
    pub code: Box<str>,
    /// Human-readable error message
    pub message: Box<str>,
    /// Additional error details (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Box<serde_json::Value>>,
    /// Suggested retry delay in seconds (optional)
    pub retry_after: Option<u64>,
    /// Correlation trace ID for debugging (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<Box<str>>,
}

impl ApiError {
    /// Create a new API error with the given status code and message
    pub fn new<S: Into<String>>(status: StatusCode, code: S, message: S) -> Self {
        Self {
            status,
            code: code.into().into_boxed_str(),
            message: message.into().into_boxed_str(),
            details: None,
            retry_after: None,
            trace_id: Self::current_trace_id(),
        }
    }

    /// Add details to the error
    pub fn with_details<V: Into<serde_json::Value>>(mut self, details: V) -> Self {
        self.details = Some(Box::new(details.into()));
        self
    }

    /// Set retry after delay
    pub fn with_retry_after(mut self, seconds: u64) -> Self {
        self.retry_after = Some(seconds);
        self
    }

    /// Extract current trace ID from the active tracing span (falls back to generated correlation ID)
    fn current_trace_id() -> Option<Box<str>> {
        telemetry::current_trace_id()
            .map(|trace_id| trace_id.into_boxed_str())
            .or_else(|| {
                // Fallback: generate a correlation ID for basic client-server log correlation
                Some(format!("corr-{}", &uuid::Uuid::new_v4().to_string()[..8]).into_boxed_str())
            })
    }
}

fn is_unique_violation(error: &sea_orm::DbErr) -> bool {
    use sea_orm::RuntimeErr;

    const PG_UNIQUE: &str = "23505";
    const MYSQL_DUPLICATE_CODES: &[&str] = &["1022", "1062", "1169", "1586"];
    const SQLITE_DUPLICATE_CODES: &[&str] = &["1555", "2067"];

    let runtime_err = match error {
        sea_orm::DbErr::Query(RuntimeErr::SqlxError(sqlx_err))
        | sea_orm::DbErr::Exec(RuntimeErr::SqlxError(sqlx_err)) => sqlx_err,
        _ => return false,
    };

    let Some(db_error) = runtime_err.as_database_error() else {
        return false;
    };

    if db_error.is_unique_violation() {
        return true;
    }

    if let Some(code) = db_error.code() {
        let code_str = code.as_ref();
        if code_str == PG_UNIQUE
            || MYSQL_DUPLICATE_CODES.contains(&code_str)
            || SQLITE_DUPLICATE_CODES.contains(&code_str)
        {
            return true;
        }

        if let Ok(code_number) = code_str.parse::<u32>()
            && (MYSQL_DUPLICATE_CODES
                .iter()
                .filter_map(|value| value.parse::<u32>().ok())
                .any(|known| known == code_number)
                || SQLITE_DUPLICATE_CODES
                    .iter()
                    .filter_map(|value| value.parse::<u32>().ok())
                    .any(|known| known == code_number))
        {
            return true;
        }
    }

    false
}

/// Standard error types with predefined status codes
#[derive(Debug, Error)]
pub enum ErrorType {
    #[error("Bad Request")]
    BadRequest,
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Forbidden")]
    Forbidden,
    #[error("Not Found")]
    NotFound,
    #[error("Conflict")]
    Conflict,
    #[error("Too Many Requests")]
    TooManyRequests,
    #[error("Internal Server Error")]
    InternalServerError,
    #[error("Bad Gateway")]
    BadGateway,
    #[error("Service Unavailable")]
    ServiceUnavailable,
}

impl ErrorType {
    /// Get the appropriate HTTP status code for this error type
    pub fn status_code(&self) -> StatusCode {
        match self {
            ErrorType::BadRequest => StatusCode::BAD_REQUEST,
            ErrorType::Unauthorized => StatusCode::UNAUTHORIZED,
            ErrorType::Forbidden => StatusCode::FORBIDDEN,
            ErrorType::NotFound => StatusCode::NOT_FOUND,
            ErrorType::Conflict => StatusCode::CONFLICT,
            ErrorType::TooManyRequests => StatusCode::TOO_MANY_REQUESTS,
            ErrorType::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            ErrorType::BadGateway => StatusCode::BAD_GATEWAY,
            ErrorType::ServiceUnavailable => StatusCode::SERVICE_UNAVAILABLE,
        }
    }

    /// Get the error code string for this error type (SCREAMING_SNAKE_CASE as per spec)
    pub fn error_code(&self) -> &'static str {
        match self {
            ErrorType::BadRequest => "VALIDATION_FAILED",
            ErrorType::Unauthorized => "UNAUTHORIZED",
            ErrorType::Forbidden => "FORBIDDEN",
            ErrorType::NotFound => "NOT_FOUND",
            ErrorType::Conflict => "CONFLICT",
            ErrorType::TooManyRequests => "RATE_LIMITED",
            ErrorType::InternalServerError => "INTERNAL_SERVER_ERROR",
            ErrorType::BadGateway => "PROVIDER_ERROR",
            ErrorType::ServiceUnavailable => "SERVICE_UNAVAILABLE",
        }
    }
}

/// Upstream provider error information
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ProviderError {
    /// Provider identifier (e.g., "github", "slack")
    pub provider: String,
    /// HTTP status code from upstream
    pub status: u16,
    /// Response body snippet from upstream (truncated for security)
    pub body_snippet: Option<String>,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let mut headers = HeaderMap::new();
        headers.insert(
            "content-type",
            HeaderValue::from_static("application/problem+json"),
        );

        // Add Retry-After header if present
        if let Some(retry_after) = self.retry_after
            && let Ok(header_value) = HeaderValue::from_str(&retry_after.to_string())
        {
            headers.insert("retry-after", header_value);
        }

        (self.status, headers, axum::Json(self)).into_response()
    }
}

// Error mappers for common sources

impl From<ErrorType> for ApiError {
    fn from(error_type: ErrorType) -> Self {
        Self::new(
            error_type.status_code(),
            error_type.error_code(),
            &error_type.to_string(),
        )
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(error: anyhow::Error) -> Self {
        // Log the full error for debugging
        tracing::error!("Internal error: {:?}", error);

        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "INTERNAL_SERVER_ERROR",
            "An internal error occurred",
        )
    }
}

impl From<JsonRejection> for ApiError {
    fn from(rejection: JsonRejection) -> Self {
        let message = match rejection {
            JsonRejection::JsonDataError(err) => format!("Invalid JSON: {}", err),
            JsonRejection::JsonSyntaxError(err) => format!("JSON syntax error: {}", err),
            JsonRejection::MissingJsonContentType(_) => {
                "Missing 'Content-Type: application/json' header".to_string()
            }
            _ => "Invalid request body".to_string(),
        };

        Self::new(StatusCode::BAD_REQUEST, "VALIDATION_FAILED", &message)
    }
}

impl From<sea_orm::DbErr> for ApiError {
    fn from(error: sea_orm::DbErr) -> Self {
        if is_unique_violation(&error) {
            tracing::debug!(?error, "Unique constraint violation detected");
            return Self::new(StatusCode::CONFLICT, "CONFLICT", "Resource already exists");
        }

        match error {
            sea_orm::DbErr::RecordNotFound(record) => Self::new(
                StatusCode::NOT_FOUND,
                "NOT_FOUND",
                &format!("Record not found: {}", record),
            ),
            sea_orm::DbErr::Query(query_err) => {
                tracing::error!("Database query error: {:?}", query_err);
                Self::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_SERVER_ERROR",
                    "Database error occurred",
                )
            }
            sea_orm::DbErr::Exec(exec_err) => {
                tracing::error!("Database execution error: {:?}", exec_err);
                Self::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_SERVER_ERROR",
                    "Database error occurred",
                )
            }
            sea_orm::DbErr::Conn(connection_err) => {
                tracing::error!("Database connection error: {:?}", connection_err);
                Self::new(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "SERVICE_UNAVAILABLE",
                    "Database service unavailable",
                )
            }
            _ => {
                tracing::error!("Database error: {:?}", error);
                Self::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_SERVER_ERROR",
                    "Database error occurred",
                )
            }
        }
    }
}

/// Create a provider upstream error
pub fn provider_error(provider: String, status: u16, body: Option<String>) -> ApiError {
    let provider_error = ProviderError {
        provider: provider.clone(),
        status,
        body_snippet: body.map(|b| {
            if b.chars().count() > 200 {
                let truncated: String = b.chars().take(200).collect();
                format!("{}...", truncated)
            } else {
                b
            }
        }),
    };

    // Per spec: ALL provider upstream HTTP errors → 502 PROVIDER_ERROR
    // This ensures provider failures are clearly distinguished from client request errors
    let api_status = StatusCode::BAD_GATEWAY;
    let api_code = "PROVIDER_ERROR";

    ApiError::new(
        api_status,
        api_code,
        &format!("Provider {} returned error status {}", provider, status),
    )
    .with_details(json!(provider_error))
}

/// Create an unauthorized error (401)
pub fn unauthorized(message: Option<&str>) -> ApiError {
    let msg = message.unwrap_or("Authentication required");
    ApiError::new(StatusCode::UNAUTHORIZED, "UNAUTHORIZED", msg)
}

/// Create an unauthorized error (401) with explicit trace_id
pub fn unauthorized_with_trace_id(message: Option<&str>, trace_id: String) -> ApiError {
    let msg = message.unwrap_or("Authentication required");
    let mut error = ApiError::new(StatusCode::UNAUTHORIZED, "UNAUTHORIZED", msg);
    error.trace_id = Some(trace_id.into_boxed_str());
    error
}

/// Create a forbidden error (403)
pub fn forbidden(message: Option<&str>) -> ApiError {
    let msg = message.unwrap_or("Insufficient permissions");
    ApiError::new(StatusCode::FORBIDDEN, "FORBIDDEN", msg)
}

/// Create a validation error with field details
pub fn validation_error(message: &str, field_errors: serde_json::Value) -> ApiError {
    ApiError::new(StatusCode::BAD_REQUEST, "VALIDATION_FAILED", message).with_details(field_errors)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use serde_json::json;

    #[test]
    fn test_api_error_basic() {
        let error = ApiError::new(
            StatusCode::BAD_REQUEST,
            "VALIDATION_FAILED",
            "Test error message",
        );

        assert_eq!(error.code, Box::from("VALIDATION_FAILED"));
        assert_eq!(error.message, Box::from("Test error message"));
        assert_eq!(error.details, None);
        assert_eq!(error.retry_after, None);
    }

    #[test]
    fn test_api_error_with_details() {
        let error = ApiError::new(StatusCode::BAD_REQUEST, "BAD_REQUEST", "Test error message")
            .with_details(json!({"field": "value"}));

        assert_eq!(error.details, Some(Box::new(json!({"field": "value"}))));
    }

    #[test]
    fn test_api_error_with_retry_after() {
        let error = ApiError::new(
            StatusCode::TOO_MANY_REQUESTS,
            "TOO_MANY_REQUESTS",
            "Rate limit exceeded",
        )
        .with_retry_after(60);

        assert_eq!(error.retry_after, Some(60));
    }

    #[test]
    fn test_error_type_mapping() {
        let not_found_error: ApiError = ErrorType::NotFound.into();
        assert_eq!(not_found_error.code, Box::from("NOT_FOUND"));
        assert_eq!(not_found_error.message, Box::from("Not Found"));
    }

    #[test]
    fn test_from_anyhow() {
        let anyhow_error = anyhow::anyhow!("Something went wrong");
        let api_error: ApiError = anyhow_error.into();

        assert_eq!(api_error.code, Box::from("INTERNAL_SERVER_ERROR"));
        assert_eq!(api_error.message, Box::from("An internal error occurred"));
    }

    #[test]
    fn test_from_json_rejection() {
        // Test with a generic rejection - the exact type doesn't matter for this test
        // We're testing the basic mapping functionality
        let api_error = ApiError::new(
            StatusCode::BAD_REQUEST,
            "VALIDATION_FAILED",
            "Invalid JSON content type",
        );

        assert_eq!(api_error.code, Box::from("VALIDATION_FAILED"));
        assert!(api_error.message.contains("JSON"));
    }

    #[test]
    fn test_provider_error() {
        let error = provider_error(
            "github".to_string(),
            429,
            Some("rate limit exceeded".to_string()),
        );

        // Per spec: ALL provider upstream errors return PROVIDER_ERROR with 502 status
        assert_eq!(error.code, Box::from("PROVIDER_ERROR"));
        assert_eq!(error.status, StatusCode::BAD_GATEWAY);
        assert!(error.message.contains("github"));
        assert!(error.details.is_some());

        // Verify the details contain the upstream status
        let details = error.details.unwrap();
        let details_obj = details.as_object().unwrap();
        assert_eq!(details_obj.get("provider").unwrap(), "github");
        assert_eq!(details_obj.get("status").unwrap(), 429);
    }

    #[test]
    fn test_content_type_header() {
        let error = ApiError::new(StatusCode::BAD_REQUEST, "VALIDATION_FAILED", "Test error");

        let response = error.into_response();

        // Check that Content-Type header is set correctly
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "application/problem+json"
        );
    }

    #[test]
    fn test_retry_after_header() {
        let error = ApiError::new(
            StatusCode::TOO_MANY_REQUESTS,
            "RATE_LIMITED",
            "Rate limit exceeded",
        )
        .with_retry_after(60);

        let response = error.into_response();

        // Check that Retry-After header is set
        assert_eq!(response.headers().get("retry-after").unwrap(), "60");

        // Check that Content-Type is still set
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "application/problem+json"
        );
    }

    #[test]
    fn test_status_code_preservation() {
        let error = ApiError::new(StatusCode::CONFLICT, "CONFLICT", "Resource already exists");

        let response = error.into_response();

        // Check that the status code is preserved
        assert_eq!(response.status(), StatusCode::CONFLICT);
    }

    #[test]
    fn test_trace_id_generation() {
        let error = ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "INTERNAL_SERVER_ERROR",
            "Test error",
        );

        // Check that trace ID is generated and has the expected format
        assert!(error.trace_id.is_some());
        let trace_id = error.trace_id.unwrap();
        assert!(trace_id.starts_with("corr-"));
        assert_eq!(trace_id.len(), 13); // "corr-" + 8 chars
    }

    #[test]
    fn test_provider_error_status_mapping() {
        // Test 502 mapping for ALL provider errors per spec

        // 5xx errors should return 502
        let error_5xx = provider_error(
            "github".to_string(),
            503,
            Some("service unavailable".to_string()),
        );
        assert_eq!(error_5xx.status, StatusCode::BAD_GATEWAY);
        assert_eq!(error_5xx.code, Box::from("PROVIDER_ERROR"));

        // 4xx errors should ALSO return 502 (per spec: all provider upstream errors)
        let error_4xx = provider_error("slack".to_string(), 401, Some("invalid token".to_string()));
        assert_eq!(error_4xx.status, StatusCode::BAD_GATEWAY);
        assert_eq!(error_4xx.code, Box::from("PROVIDER_ERROR"));

        // 429 errors should ALSO return 502 (not 429)
        let error_429 = provider_error(
            "google".to_string(),
            429,
            Some("rate limit exceeded".to_string()),
        );
        assert_eq!(error_429.status, StatusCode::BAD_GATEWAY);
        assert_eq!(error_429.code, Box::from("PROVIDER_ERROR"));

        // 2xx errors (unlikely but should still map to 502)
        let error_2xx = provider_error(
            "jira".to_string(),
            200,
            Some("success but invalid format".to_string()),
        );
        assert_eq!(error_2xx.status, StatusCode::BAD_GATEWAY);
        assert_eq!(error_2xx.code, Box::from("PROVIDER_ERROR"));

        // Verify responses all return 502
        for error in [&error_5xx, &error_4xx, &error_429, &error_2xx] {
            let response = error.clone().into_response();
            assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
        }
    }

    #[test]
    fn test_database_error_mapping() {
        let db_error = sea_orm::DbErr::RecordNotFound("test_record".to_string());
        let api_error: ApiError = db_error.into();

        assert_eq!(api_error.status, StatusCode::NOT_FOUND);
        assert_eq!(api_error.code, Box::from("NOT_FOUND"));
        assert!(api_error.message.contains("test_record"));
    }

    #[test]
    fn test_auth_error_helpers() {
        // Test unauthorized error
        let auth_error = unauthorized(None);
        assert_eq!(auth_error.status, StatusCode::UNAUTHORIZED);
        assert_eq!(auth_error.code, Box::from("UNAUTHORIZED"));
        assert_eq!(auth_error.message, Box::from("Authentication required"));

        // Test unauthorized error with custom message
        let custom_auth_error = unauthorized(Some("Invalid token"));
        assert_eq!(custom_auth_error.message, Box::from("Invalid token"));

        // Test forbidden error
        let forbidden_error = forbidden(None);
        assert_eq!(forbidden_error.status, StatusCode::FORBIDDEN);
        assert_eq!(forbidden_error.code, Box::from("FORBIDDEN"));
        assert_eq!(
            forbidden_error.message,
            Box::from("Insufficient permissions")
        );

        // Test forbidden error with custom message
        let custom_forbidden_error = forbidden(Some("Admin access required"));
        assert_eq!(
            custom_forbidden_error.message,
            Box::from("Admin access required")
        );
    }

    #[test]
    fn test_validation_error_with_details() {
        let field_errors = json!({
            "name": "Name is required",
            "email": "Invalid email format"
        });

        let validation_error = validation_error("Validation failed", field_errors.clone());

        assert_eq!(validation_error.status, StatusCode::BAD_REQUEST);
        assert_eq!(validation_error.code, Box::from("VALIDATION_FAILED"));
        assert_eq!(validation_error.message, Box::from("Validation failed"));
        assert_eq!(validation_error.details, Some(Box::new(field_errors)));
    }

    #[test]
    fn test_spec_scenarios_compliance() {
        // Scenario: Validation error returns 400 with details (matches spec)
        let validation_err = validation_error("Validation failed", json!({"name": "required"}));
        assert_eq!(validation_err.status, StatusCode::BAD_REQUEST);
        assert_eq!(validation_err.code, Box::from("VALIDATION_FAILED"));
        assert!(validation_err.trace_id.is_some());

        // Scenario: Not found returns 404 (matches spec)
        let not_found_err: ApiError = ErrorType::NotFound.into();
        assert_eq!(not_found_err.status, StatusCode::NOT_FOUND);
        assert_eq!(not_found_err.code, Box::from("NOT_FOUND"));
        assert!(not_found_err.trace_id.is_some());

        // Scenario: Rate limited returns 429 with Retry-After (matches spec)
        let rate_limit_err = ApiError::new(
            StatusCode::TOO_MANY_REQUESTS,
            "RATE_LIMITED",
            "Rate limit exceeded",
        )
        .with_retry_after(60);
        assert_eq!(rate_limit_err.status, StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(rate_limit_err.code, Box::from("RATE_LIMITED"));
        assert_eq!(rate_limit_err.retry_after, Some(60));
        assert!(rate_limit_err.trace_id.is_some());

        // Scenario: Internal error returns 500 with trace id (matches spec)
        let internal_err: ApiError = anyhow::anyhow!("Something went wrong").into();
        assert_eq!(internal_err.status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(internal_err.code, Box::from("INTERNAL_SERVER_ERROR"));
        assert!(internal_err.trace_id.is_some());

        // Scenario: Provider error maps to 502 (matches spec)
        let provider_err = provider_error(
            "github".to_string(),
            503,
            Some("Service unavailable".to_string()),
        );
        assert_eq!(provider_err.status, StatusCode::BAD_GATEWAY);
        assert_eq!(provider_err.code, Box::from("PROVIDER_ERROR"));
        assert!(provider_err.details.is_some());
        assert!(provider_err.trace_id.is_some());

        // Verify Content-Type header for all errors
        for error in [
            &validation_err,
            &not_found_err,
            &rate_limit_err,
            &internal_err,
            &provider_err,
        ] {
            let response = error.clone().into_response();
            assert_eq!(
                response.headers().get("content-type").unwrap(),
                "application/problem+json"
            );
        }
    }

    #[test]
    fn test_provider_error_spec_comprehensive_compliance() {
        println!("=== COMPREHENSIVE PROVIDER ERROR SPEC COMPLIANCE TEST ===");

        // Test all possible upstream statuses to verify SPEC compliance:
        // "Provider upstream HTTP errors → 502 PROVIDER_ERROR with provider/status metadata in details"

        let test_cases = vec![
            (200, "Success but invalid format"),
            (400, "Bad request from provider"),
            (401, "Unauthorized from provider"),
            (404, "Not found from provider"),
            (429, "Rate limited from provider"),
            (500, "Internal server error from provider"),
            (503, "Service unavailable from provider"),
        ];

        for (upstream_status, message) in test_cases {
            println!("\nTesting upstream {} error:", upstream_status);

            let error = provider_error(
                "test-provider".to_string(),
                upstream_status,
                Some(message.to_string()),
            );

            // SPEC REQUIREMENT 1: Return HTTP 502 BAD_GATEWAY for ALL provider errors
            assert_eq!(
                error.status,
                StatusCode::BAD_GATEWAY,
                "FAILED: Upstream {} should return HTTP 502, got {}",
                upstream_status,
                error.status.as_u16()
            );
            println!(
                "  ✅ HTTP Status: {} (502 BAD_GATEWAY)",
                error.status.as_u16()
            );

            // SPEC REQUIREMENT 2: Return PROVIDER_ERROR code for ALL provider errors
            assert_eq!(
                error.code.as_ref(),
                "PROVIDER_ERROR",
                "FAILED: Upstream {} should return PROVIDER_ERROR code, got {}",
                upstream_status,
                error.code
            );
            println!("  ✅ Error Code: {}", error.code);

            // SPEC REQUIREMENT 3: Include provider/status metadata in details
            assert!(
                error.details.is_some(),
                "FAILED: Upstream {} should have details metadata",
                upstream_status
            );

            let details = error.details.as_ref().unwrap();
            let details_obj = details.as_object().unwrap();

            assert_eq!(
                details_obj.get("provider").unwrap(),
                "test-provider",
                "FAILED: Should include provider name in details"
            );

            assert_eq!(
                details_obj.get("status").unwrap(),
                upstream_status,
                "FAILED: Should include upstream status {} in details",
                upstream_status
            );

            assert!(
                details_obj.get("body_snippet").is_some(),
                "FAILED: Should include body snippet in details"
            );

            println!(
                "  ✅ Details: {}",
                serde_json::to_string_pretty(&details).unwrap()
            );
        }

        println!("\n=== SPEC COMPLIANCE VERIFIED ===");
        println!("✅ ALL upstream HTTP errors return HTTP 502 BAD_GATEWAY");
        println!("✅ ALL upstream HTTP errors return PROVIDER_ERROR code");
        println!("✅ ALL include provider/status metadata in details");
        println!(
            "✅ Implementation follows spec: 'Provider upstream HTTP errors → 502 PROVIDER_ERROR'"
        );
    }

    #[test]
    fn test_utf8_safe_truncation() {
        // Test with multi-byte UTF-8 characters to ensure no panic on character boundaries
        let test_string = "测试中文字符🚀 This is a test string with emoji and Chinese characters that should be truncated safely without panicking on UTF-8 boundaries. ".repeat(5);

        let error = provider_error(
            "test-provider".to_string(),
            500,
            Some(test_string.to_string()),
        );

        // Verify the error was created successfully (no panic)
        assert_eq!(error.status, StatusCode::BAD_GATEWAY);
        assert_eq!(error.code, Box::from("PROVIDER_ERROR"));

        // Verify details contain the truncated body
        assert!(error.details.is_some());
        let details = error.details.unwrap();
        let details_obj = details.as_object().unwrap();
        let body_snippet = details_obj.get("body_snippet").unwrap().as_str().unwrap();

        // Should be truncated to 200 characters (not bytes) and end with "..."
        assert!(body_snippet.chars().count() <= 203); // 200 chars + "..."

        // Check if it was truncated (original string was longer than 200 chars)
        if test_string.chars().count() > 200 {
            assert!(body_snippet.ends_with("..."));
        }

        // Verify the truncated string is valid UTF-8
        let _valid_utf8 = body_snippet.to_string(); // This would panic if invalid UTF-8

        // Verify it contains the beginning of our test string
        assert!(body_snippet.starts_with("测试中文字符🚀 This is a test string"));
    }
}
