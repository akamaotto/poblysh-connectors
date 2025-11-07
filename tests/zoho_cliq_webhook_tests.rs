//! Integration tests for Zoho Cliq webhook processing
//!
//! Tests focused on webhook verification and processing without complex setup

use connectors::webhook_verification::{verify_webhook_signature, VerificationError};
use connectors::config::AppConfig;
use axum::http::HeaderMap;

/// Test Zoho Cliq webhook verification with valid Bearer token
#[tokio::test]
async fn test_zoho_cliq_webhook_verification_integration() {
    let mut config = AppConfig::default();
    config.webhook_zoho_cliq_token = Some("test-zoho-cliq-token".to_string());

    let mut headers = HeaderMap::new();
    headers.insert("authorization", "Bearer test-zoho-cliq-token".parse().unwrap());

    let payload = r#"{"event_type": "message_posted", "message": {"id": "msg_123"}}"#;

    let result = verify_webhook_signature("zoho-cliq", payload.as_bytes(), &headers, &config);
    assert!(result.is_ok());
}

/// Test Zoho Cliq webhook verification rejection with invalid token
#[tokio::test]
async fn test_zoho_cliq_webhook_verification_rejection_integration() {
    let mut config = AppConfig::default();
    config.webhook_zoho_cliq_token = Some("correct-token".to_string());

    let mut headers = HeaderMap::new();
    headers.insert("authorization", "Bearer wrong-token".parse().unwrap());

    let payload = r#"{"event_type": "message_posted", "message": {"id": "msg_123"}}"#;

    let result = verify_webhook_signature("zoho-cliq", payload.as_bytes(), &headers, &config);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), VerificationError::VerificationFailed));
}

/// Test Zoho Cliq webhook verification when not configured
#[tokio::test]
async fn test_zoho_cliq_webhook_verification_not_configured_integration() {
    let config = AppConfig::default(); // No zoho-cliq token configured

    let mut headers = HeaderMap::new();
    headers.insert("authorization", "Bearer some-token".parse().unwrap());

    let payload = r#"{"event_type": "message_posted", "message": {"id": "msg_123"}}"#;

    let result = verify_webhook_signature("zoho-cliq", payload.as_bytes(), &headers, &config);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), VerificationError::NotConfigured { .. }));
}

/// Test Zoho Cliq webhook verification with missing Authorization header
#[tokio::test]
async fn test_zoho_cliq_webhook_verification_missing_auth_integration() {
    let mut config = AppConfig::default();
    config.webhook_zoho_cliq_token = Some("some-token".to_string());

    let headers = HeaderMap::new(); // No Authorization header

    let payload = r#"{"event_type": "message_posted", "message": {"id": "msg_123"}}"#;

    let result = verify_webhook_signature("zoho-cliq", payload.as_bytes(), &headers, &config);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), VerificationError::MissingSignature { .. }));
}

/// Test that other providers still work correctly
#[tokio::test]
async fn test_other_providers_unaffected_integration() {
    let mut config = AppConfig::default();
    config.webhook_zoho_cliq_token = Some("zoho-token".to_string());
    config.webhook_github_secret = Some("github-secret".to_string());

    // Test GitHub still works
    let mut headers = HeaderMap::new();
    headers.insert("x-hub-signature-256", "sha256=invalid".parse().unwrap());

    let result = verify_webhook_signature("github", b"test", &headers, &config);
    assert!(result.is_err()); // Should fail verification, but not due to missing configuration

    // Test unsupported provider
    let result = verify_webhook_signature("unknown", b"test", &headers, &config);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), VerificationError::UnsupportedProvider { .. }));
}