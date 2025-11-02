//! # Tests for Handlers
//!
//! This module contains unit tests for API handlers.

use std::sync::Arc;

use crate::config::AppConfig;
use crate::handlers::root;
use crate::models::ServiceInfo;
use crate::server::AppState;
use axum::{extract::State, response::Json};
use sea_orm::DatabaseConnection;
use serde_json::Value;

#[tokio::test]
async fn test_root_handler_returns_success() {
    // Create a mock state for testing
    let db = DatabaseConnection::default();
    let state = AppState {
        config: Arc::new(AppConfig::default()),
        db,
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
    let state = AppState {
        config: Arc::new(AppConfig::default()),
        db,
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
    let state = AppState {
        config: Arc::new(AppConfig::default()),
        db,
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
