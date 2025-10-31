//! # Tests for Handlers
//!
//! This module contains unit tests for the API handlers.

use axum::response::Json;
use crate::handlers::root;
use crate::models::ServiceInfo;
use serde_json::Value;

#[tokio::test]
async fn test_root_handler_returns_success() {
    // Call the root handler
    let response = root().await;
    
    // Verify the response is a Json type
    assert!(matches!(response, Json(_)));
}

#[tokio::test]
async fn test_root_handler_returns_expected_service_info() {
    // Call the root handler
    let response = root().await;
    
    // Extract the ServiceInfo from the Json response
    let Json(service_info) = response;
    
    // Verify the service name
    assert_eq!(service_info.service, "poblysh-connectors");
    
    // Verify the version
    assert_eq!(service_info.version, "0.1.0");
}

#[tokio::test]
async fn test_root_handler_returns_valid_json() {
    // Call the root handler
    let response = root().await;
    
    // Extract the ServiceInfo from the Json response
    let Json(service_info) = response;
    
    // Convert to JSON value to verify it can be serialized
    let json_value: Value = serde_json::to_value(&service_info).expect("Failed to serialize ServiceInfo");
    
    // Verify the JSON structure
    assert!(json_value.get("service").is_some());
    assert!(json_value.get("version").is_some());
    
    // Verify the values in the JSON
    assert_eq!(json_value.get("service").unwrap().as_str().unwrap(), "poblysh-connectors");
    assert_eq!(json_value.get("version").unwrap().as_str().unwrap(), "0.1.0");
}

#[tokio::test]
async fn test_service_info_default() {
    // Test the default implementation of ServiceInfo
    let service_info = ServiceInfo::default();
    
    assert_eq!(service_info.service, "poblysh-connectors");
    assert_eq!(service_info.version, "0.1.0");
}