//! Integration Tests for Connectors API
//!
//! These tests verify that the HTTP endpoints work correctly when the server is running.

use connectors::server::create_app;
use reqwest::Client;
use serde_json::Value;
use std::net::SocketAddr;
use tokio::net::TcpListener;

/// Helper function to get a random available port
async fn get_available_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    port
}

/// Helper function to start the server on a random port
async fn start_test_server() -> String {
    let port = get_available_port().await;
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    
    let app = create_app();
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    
    // Start the server in the background
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    
    // Give the server a moment to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    format!("http://127.0.0.1:{}", port)
}

#[tokio::test]
async fn test_root_endpoint() {
    // Start the test server
    let server_url = start_test_server().await;
    let client = Client::new();
    
    // Make a GET request to the root endpoint
    let response = client
        .get(&format!("{}/", server_url))
        .send()
        .await
        .expect("Failed to execute request");
    
    // Assert response status is 200 OK
    assert_eq!(response.status(), 200);
    
    // Assert Content-Type is application/json
    assert_eq!(
        response.headers().get("content-type").unwrap(),
        "application/json"
    );
    
    // Parse and validate the response body
    let body: Value = response.json().await.expect("Failed to parse JSON");
    
    // Check for expected service info payload
    assert_eq!(body.get("service").unwrap().as_str().unwrap(), "poblysh-connectors");
    assert_eq!(body.get("version").unwrap().as_str().unwrap(), "0.1.0");
}

#[tokio::test]
async fn test_openapi_endpoint() {
    // Start the test server
    let server_url = start_test_server().await;
    let client = Client::new();
    
    // Make a GET request to the OpenAPI endpoint
    let response = client
        .get(&format!("{}/openapi.json", server_url))
        .send()
        .await
        .expect("Failed to execute request");
    
    // Assert response status is 200 OK
    assert_eq!(response.status(), 200);
    
    // Assert Content-Type is application/json
    assert_eq!(
        response.headers().get("content-type").unwrap(),
        "application/json"
    );
    
    // Parse and validate the response body
    let body: Value = response.json().await.expect("Failed to parse JSON");
    
    // Check for required OpenAPI fields
    assert!(body.get("openapi").is_some(), "Missing 'openapi' field");
    assert!(body.get("info").is_some(), "Missing 'info' field");
    assert!(body.get("paths").is_some(), "Missing 'paths' field");
    
    // Validate the info object has required fields
    let info = body.get("info").unwrap();
    assert!(info.get("title").is_some(), "Missing 'title' in info object");
    assert!(info.get("version").is_some(), "Missing 'version' in info object");
    
    // Check for specific values
    assert_eq!(info.get("title").unwrap().as_str().unwrap(), "Poblysh Connectors API");
    assert_eq!(info.get("version").unwrap().as_str().unwrap(), "0.1.0");
}