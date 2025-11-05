//! # Cursor Utilities
//!
//! This module provides utilities for encoding and decoding pagination cursors
//! with comprehensive validation and security checks.

use crate::error::ApiError;
use axum::http::StatusCode;
use base64::Engine;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Re-export the CursorData from repositories to avoid duplication
pub use crate::repositories::signal::CursorData;

/// Encode cursor data as an opaque base64 string
pub fn encode_cursor(occurred_at: &DateTime<Utc>, id: &Uuid) -> String {
    let cursor_data = CursorData {
        occurred_at: *occurred_at,
        id: *id,
    };
    let json = serde_json::to_string(&cursor_data).unwrap();
    base64::engine::general_purpose::STANDARD.encode(json.as_bytes())
}

/// Decode cursor data from an opaque base64 string with validation
pub fn decode_cursor(cursor: &str) -> Result<CursorData, ApiError> {
    // Check cursor length to prevent extremely large inputs
    if cursor.len() > 1000 {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            "VALIDATION_FAILED",
            "cursor is too long",
        ));
    }

    // Check for empty cursor
    if cursor.is_empty() {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            "VALIDATION_FAILED",
            "cursor cannot be empty",
        ));
    }

    // Validate base64 format
    if !cursor
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=')
    {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            "VALIDATION_FAILED",
            "cursor contains invalid characters",
        ));
    }

    // Decode base64
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(cursor)
        .map_err(|_| {
            ApiError::new(
                StatusCode::BAD_REQUEST,
                "VALIDATION_FAILED",
                "cursor is not valid base64",
            )
        })?;

    // Check decoded size
    if decoded.is_empty() {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            "VALIDATION_FAILED",
            "cursor is empty after decoding",
        ));
    }

    if decoded.len() > 500 {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            "VALIDATION_FAILED",
            "decoded cursor is too large",
        ));
    }

    // Convert to UTF-8 string
    let json = String::from_utf8(decoded).map_err(|_| {
        ApiError::new(
            StatusCode::BAD_REQUEST,
            "VALIDATION_FAILED",
            "cursor contains invalid UTF-8 data",
        )
    })?;

    // Parse JSON
    let cursor_data: CursorData = serde_json::from_str(&json).map_err(|_| {
        ApiError::new(
            StatusCode::BAD_REQUEST,
            "VALIDATION_FAILED",
            "cursor contains invalid JSON structure",
        )
    })?;

    // Validate timestamp is reasonable (not too far in future or past)
    let now = Utc::now();
    let one_year_ago = now - chrono::Duration::days(365);
    let one_year_from_now = now + chrono::Duration::days(365);

    if cursor_data.occurred_at < one_year_ago {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            "VALIDATION_FAILED",
            "cursor timestamp is too old",
        ));
    }

    if cursor_data.occurred_at > one_year_from_now {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            "VALIDATION_FAILED",
            "cursor timestamp is too far in the future",
        ));
    }

    // Validate UUID is not nil
    if cursor_data.id == uuid::Uuid::nil() {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            "VALIDATION_FAILED",
            "cursor contains invalid ID",
        ));
    }

    Ok(cursor_data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn test_cursor_encoding_decoding() {
        let occurred_at = Utc::now();
        let id = Uuid::new_v4();

        let cursor_str = encode_cursor(&occurred_at, &id);
        let decoded = decode_cursor(&cursor_str).unwrap();

        assert_eq!(decoded.occurred_at, occurred_at);
        assert_eq!(decoded.id, id);
    }

    #[test]
    fn test_invalid_cursor_decoding() {
        let invalid_cursor = "invalid-base64!";
        let result = decode_cursor(invalid_cursor);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_cursor() {
        let result = decode_cursor("");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, "VALIDATION_FAILED".into());
        assert!(err.message.contains("cannot be empty"));
    }

    #[test]
    fn test_cursor_too_long() {
        let long_cursor = "a".repeat(1001);
        let result = decode_cursor(&long_cursor);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, "VALIDATION_FAILED".into());
        assert!(err.message.contains("too long"));
    }

    #[test]
    fn test_cursor_invalid_characters() {
        let invalid_cursor = "cursor@#$%";
        let result = decode_cursor(invalid_cursor);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, "VALIDATION_FAILED".into());
        assert!(err.message.contains("invalid characters"));
    }

    #[test]
    fn test_cursor_invalid_utf8() {
        // Create base64 that decodes to invalid UTF-8
        let invalid_utf8_base64 = "//8=";
        let result = decode_cursor(invalid_utf8_base64);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, "VALIDATION_FAILED".into());
        assert!(err.message.contains("invalid UTF-8"));
    }

    #[test]
    fn test_cursor_invalid_json() {
        // Create base64 that decodes to invalid JSON
        let invalid_json_base64 = "aW52YWxpZCBqc29u"; // "invalid json"
        let result = decode_cursor(invalid_json_base64);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, "VALIDATION_FAILED".into());
        assert!(err.message.contains("invalid JSON structure"));
    }

    #[test]
    fn test_cursor_timestamp_too_old() {
        let occurred_at = Utc::now() - chrono::Duration::days(400); // More than 1 year ago
        let id = Uuid::new_v4();

        let cursor_str = encode_cursor(&occurred_at, &id);
        let result = decode_cursor(&cursor_str);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, "VALIDATION_FAILED".into());
        assert!(err.message.contains("too old"));
    }

    #[test]
    fn test_cursor_timestamp_too_future() {
        let occurred_at = Utc::now() + chrono::Duration::days(400); // More than 1 year in future
        let id = Uuid::new_v4();

        let cursor_str = encode_cursor(&occurred_at, &id);
        let result = decode_cursor(&cursor_str);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, "VALIDATION_FAILED".into());
        assert!(err.message.contains("too far in the future"));
    }

    #[test]
    fn test_cursor_nil_uuid() {
        let occurred_at = Utc::now();
        let id = uuid::Uuid::nil();

        let cursor_str = encode_cursor(&occurred_at, &id);
        let result = decode_cursor(&cursor_str);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, "VALIDATION_FAILED".into());
        assert!(err.message.contains("invalid ID"));
    }

    #[test]
    fn test_cursor_decoded_too_large() {
        // Create a large JSON object and encode it
        let large_data = "x".repeat(600); // More than 500 bytes after decoding
        let json = format!(
            r#"{{"occurred_at":"2024-01-01T00:00:00Z","id":"550e8400-e29b-41d4-a716-446655440000","data":"{}"}}"#,
            large_data
        );
        let cursor_str = base64::engine::general_purpose::STANDARD.encode(json.as_bytes());

        let result = decode_cursor(&cursor_str);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, "VALIDATION_FAILED".into());
        assert!(err.message.contains("too large"));
    }

    #[test]
    fn test_cursor_security_validation() {
        // Test various security scenarios

        // 1. Base64 with padding attacks
        let malicious_cursor = "a".repeat(1000) + "=";
        let result = decode_cursor(&malicious_cursor);
        assert!(result.is_err());

        // 2. JSON injection attempts
        let current_time = Utc::now();
        let json_injection = format!(
            r#"{{"occurred_at":"{}","id":"550e8400-e29b-41d4-a716-446655440000","injected":true}}"#,
            current_time.to_rfc3339()
        );
        let injected_cursor =
            base64::engine::general_purpose::STANDARD.encode(json_injection.as_bytes());
        let result = decode_cursor(&injected_cursor);
        // Should succeed because extra fields are ignored by serde
        assert!(result.is_ok());

        // 3. Timestamp boundary testing
        let now = Utc::now();
        let recent_timestamp = now - chrono::Duration::days(30); // 30 days ago - should be valid
        let id = Uuid::new_v4();
        let recent_cursor = encode_cursor(&recent_timestamp, &id);
        let result = decode_cursor(&recent_cursor);
        // Should succeed
        assert!(result.is_ok());

        let old_timestamp = now - chrono::Duration::days(400); // More than 1 year ago - should be invalid
        let old_cursor = encode_cursor(&old_timestamp, &id);
        let result = decode_cursor(&old_cursor);
        assert!(result.is_err());
    }
}
