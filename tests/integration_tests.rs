//! Basic integration tests for the Connectors API HTTP surface.

use connectors::server::{AppState, create_app};
use reqwest::Client;
use sea_orm::DatabaseConnection;
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

    // Create a mock AppState for testing
    let db = DatabaseConnection::default();
    let crypto_key = connectors::crypto::CryptoKey::new(vec![0u8; 32])
        .expect("Failed to create test crypto key");
    let state = AppState {
        config: std::sync::Arc::new(connectors::config::AppConfig::default()),
        db,
        crypto_key,
    };

    let app = create_app(state);
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
    let server_url = start_test_server().await;
    let client = Client::new();

    let response = client
        .get(format!("{}/", server_url))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);
    assert_eq!(
        response.headers().get("content-type").unwrap(),
        "application/json"
    );

    let body: Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(
        body.get("service").unwrap().as_str().unwrap(),
        "poblysh-connectors"
    );
    assert_eq!(body.get("version").unwrap().as_str().unwrap(), "0.1.0");
}

#[tokio::test]
async fn test_openapi_endpoint() {
    let server_url = start_test_server().await;
    let client = Client::new();

    let response = client
        .get(format!("{}/openapi.json", server_url))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);
    assert_eq!(
        response.headers().get("content-type").unwrap(),
        "application/json"
    );

    let body: Value = response.json().await.expect("Failed to parse JSON");
    assert!(body.get("openapi").is_some());
    let info = body.get("info").unwrap();
    assert_eq!(
        info.get("title").unwrap().as_str().unwrap(),
        "Poblysh Connectors API"
    );
    assert_eq!(info.get("version").unwrap().as_str().unwrap(), "0.1.0");
}
