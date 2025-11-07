//! # Tests for Handlers
//!
//! This module contains unit tests for API handlers.

use std::sync::Arc;

use crate::config::AppConfig;
use crate::handlers::root;
use crate::models::ServiceInfo;
use crate::server::AppState;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Json,
};
use sea_orm::DatabaseConnection;
use serde_json::Value;
use uuid::Uuid;

#[tokio::test]
async fn test_root_handler_returns_success() {
    // Create a mock state for testing
    let db = DatabaseConnection::default();
    let crypto_key =
        crate::crypto::CryptoKey::new(vec![0u8; 32]).expect("Failed to create test crypto key");
    let state = AppState {
        config: Arc::new(AppConfig::default()),
        db,
        crypto_key,
    };
    let state = State(state);

    // Call root handler
    let result = root(state).await;

    // Verify response is Ok(Json(_))
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(matches!(response, Json(_)));
}

#[tokio::test]
async fn test_root_handler_returns_expected_service_info() {
    // Create a mock state for testing
    let db = DatabaseConnection::default();
    let crypto_key =
        crate::crypto::CryptoKey::new(vec![0u8; 32]).expect("Failed to create test crypto key");
    let state = AppState {
        config: Arc::new(AppConfig::default()),
        db,
        crypto_key,
    };
    let state = State(state);

    // Call root handler
    let result = root(state).await;
    let response = result.unwrap();

    // Extract ServiceInfo from Json response
    let Json(service_info) = response;

    // Verify service name
    assert_eq!(service_info.service, "poblysh-connectors");

    // Verify version
    assert_eq!(service_info.version, "0.1.0");
}

#[tokio::test]
async fn test_root_handler_returns_valid_json() {
    // Create a mock state for testing
    let db = DatabaseConnection::default();
    let crypto_key =
        crate::crypto::CryptoKey::new(vec![0u8; 32]).expect("Failed to create test crypto key");
    let state = AppState {
        config: Arc::new(AppConfig::default()),
        db,
        crypto_key,
    };
    let state = State(state);

    // Call root handler
    let result = root(state).await;
    let response = result.unwrap();

    // Extract ServiceInfo from Json response
    let Json(service_info) = response;

    // Convert to JSON value to verify it can be serialized
    let json_value: Value =
        serde_json::to_value(&service_info).expect("Failed to serialize ServiceInfo");

    // Verify JSON structure
    assert!(json_value.get("service").is_some());
    assert!(json_value.get("version").is_some());

    // Verify values in JSON
    assert_eq!(
        json_value.get("service").unwrap().as_str().unwrap(),
        "poblysh-connectors"
    );
    assert_eq!(
        json_value.get("version").unwrap().as_str().unwrap(),
        "0.1.0"
    );
}

#[tokio::test]
async fn test_service_info_default() {
    // Test the default implementation of ServiceInfo
    let service_info = ServiceInfo::default();

    assert_eq!(service_info.service, "poblysh-connectors");
    assert_eq!(service_info.version, "0.1.0");
}

// Tests for CRITICAL functionality
#[cfg(test)]
mod critical_tests {
    use super::*;

    #[test]
    fn test_validate_redirect_uri_allowed_patterns() {
        // Test allowed redirect URIs
        let tenant_id = Uuid::new_v4();

        // These should all succeed
        crate::handlers::connect::validate_redirect_uri(
            "http://localhost:3000/callback",
            tenant_id,
        )
        .unwrap();
        crate::handlers::connect::validate_redirect_uri(
            "https://app.poblysh.com/callback",
            tenant_id,
        )
        .unwrap();
        crate::handlers::connect::validate_redirect_uri(
            "https://customer.poblysh.com/callback",
            tenant_id,
        )
        .unwrap();
    }

    #[test]
    fn test_validate_redirect_uri_blocked_patterns() {
        // Test blocked redirect URIs
        let tenant_id = Uuid::new_v4();

        // These should all fail
        assert!(
            crate::handlers::connect::validate_redirect_uri("http://evil.com/callback", tenant_id)
                .is_err()
        );
        assert!(
            crate::handlers::connect::validate_redirect_uri("https://evil.com/callback", tenant_id)
                .is_err()
        );
        assert!(
            crate::handlers::connect::validate_redirect_uri(
                "http://localhost:3001/callback",
                tenant_id
            )
            .is_err()
        );
        assert!(
            crate::handlers::connect::validate_redirect_uri(
                "https://app.poblysh.com/other",
                tenant_id
            )
            .is_err()
        );
    }

    #[test]
    fn test_verify_jira_webhook_secret_valid_header() {
        // Test valid webhook secret verification via X-Webhook-Secret header
        let mut headers = HeaderMap::new();
        headers.insert("X-Webhook-Secret", "test-secret-123".parse().unwrap());

        let body = b"test payload";
        let result = crate::handlers::webhooks::verify_jira_webhook_secret(
            &headers,
            body,
            "test-secret-123",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_jira_webhook_secret_valid_auth_header() {
        // Test valid webhook secret verification via Authorization header
        let mut headers = HeaderMap::new();
        headers.insert("authorization", "Bearer test-secret-123".parse().unwrap());

        let body = b"test payload";
        let result = crate::handlers::webhooks::verify_jira_webhook_secret(
            &headers,
            body,
            "test-secret-123",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_jira_webhook_secret_invalid_secret() {
        // Test invalid webhook secret
        let mut headers = HeaderMap::new();
        headers.insert("X-Webhook-Secret", "wrong-secret".parse().unwrap());

        let body = b"test payload";
        let result = crate::handlers::webhooks::verify_jira_webhook_secret(
            &headers,
            body,
            "test-secret-123",
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_jira_webhook_secret_missing_header() {
        // Test missing webhook secret headers
        let headers = HeaderMap::new();
        let body = b"test payload";
        let result = crate::handlers::webhooks::verify_jira_webhook_secret(
            &headers,
            body,
            "test-secret-123",
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_config_validation_local_profile_skips_jira() {
        // Test that local/test profiles don't require Jira configuration
        let mut config = AppConfig::default();
        config.profile = "local".to_string();
        config.github_client_id = Some("test-github-id".to_string());
        config.github_client_secret = Some("test-github-secret".to_string());
        config.crypto_key = Some(vec![0u8; 32]);

        // Should succeed even without Jira config
        assert!(config.validate().is_ok());

        config.profile = "test".to_string();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation_production_profile_requires_jira() {
        // Test that production profiles require Jira configuration
        let mut config = AppConfig::default();
        config.profile = "production".to_string();
        config.github_client_id = Some("test-github-id".to_string());
        config.github_client_secret = Some("test-github-secret".to_string());
        config.crypto_key = Some(vec![0u8; 32]);

        // Should fail without Jira config
        assert!(config.validate().is_err());

        // Add Jira config and should succeed
        config.jira_client_id = Some("test-jira-id".to_string());
        config.jira_client_secret = Some("test-jira-secret".to_string());
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation_staging_profile_requires_jira() {
        // Test that staging profiles require Jira configuration
        let mut config = AppConfig::default();
        config.profile = "staging".to_string();
        config.github_client_id = Some("test-github-id".to_string());
        config.github_client_secret = Some("test-github-secret".to_string());
        config.crypto_key = Some(vec![0u8; 32]);

        // Should fail without Jira config
        assert!(config.validate().is_err());

        // Add Jira config and should succeed
        config.jira_client_id = Some("test-jira-id".to_string());
        config.jira_client_secret = Some("test-jira-secret".to_string());
        assert!(config.validate().is_ok());
    }
}
