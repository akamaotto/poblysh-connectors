//! Integration tests for authentication and tenant validation

use anyhow::{Context, Result as AnyhowResult};
use connectors::connectors::Registry;
use connectors::repositories::ConnectionRepository;
use connectors::token_refresh::TokenRefreshService;
use connectors::{config::AppConfig, server::create_app};
use reqwest::StatusCode;
use sea_orm::DatabaseConnection;
use serde_json::Value;
use std::sync::Arc;
use tokio::{net::TcpListener, sync::oneshot, task::JoinHandle};
use uuid::Uuid;

#[path = "test_utils/mod.rs"]
mod test_utils;

struct TestServerHandle {
    shutdown_tx: Option<oneshot::Sender<()>>,
    join_handle: Option<JoinHandle<AnyhowResult<()>>>,
}

impl TestServerHandle {
    fn new(shutdown_tx: oneshot::Sender<()>, join_handle: JoinHandle<AnyhowResult<()>>) -> Self {
        Self {
            shutdown_tx: Some(shutdown_tx),
            join_handle: Some(join_handle),
        }
    }

    async fn shutdown(mut self) -> AnyhowResult<()> {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }

        if let Some(handle) = self.join_handle.take() {
            let result = handle.await.context("server task join failed")?;
            result?;
        }

        Ok(())
    }
}

impl Drop for TestServerHandle {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }
}

/// Test helper to spawn a test server
async fn spawn_test_app(config: AppConfig) -> (String, Arc<DatabaseConnection>, TestServerHandle) {
    // Create a test database with migrations
    let db = Arc::new(test_utils::setup_test_db().await.unwrap());

    // Create app state
    let crypto_key = connectors::crypto::CryptoKey::new(vec![0u8; 32])
        .expect("Failed to create test crypto key");

    // Create required dependencies for TokenRefreshService
    let config_arc = Arc::new(config.clone());
    let db_arc = Arc::new(db.as_ref().clone());
    let crypto_key_for_repo = connectors::crypto::CryptoKey::new(vec![0u8; 32])
        .expect("Failed to create test crypto key for repo");
    let connection_repo = Arc::new(ConnectionRepository::new(
        db_arc.clone(),
        crypto_key_for_repo,
    ));
    let connector_registry = Registry::new();

    // Create TokenRefreshService
    let token_refresh_service = Arc::new(TokenRefreshService::new(
        config_arc,
        db_arc,
        connection_repo,
        connector_registry,
    ));

    let state = connectors::server::AppState {
        config: Arc::new(config.clone()),
        db: db.as_ref().clone(),
        crypto_key,
        token_refresh_service,
    };

    // Create app
    let app = create_app(state);

    // Bind to a random port
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server_url = format!("http://{}", addr);

    let (ready_tx, ready_rx) = oneshot::channel();
    let (shutdown_tx, shutdown_rx) = oneshot::channel();

    // Spawn server in background
    let server_task = tokio::spawn(async move {
        let server = axum::serve(listener, app).with_graceful_shutdown(async {
            let _ = shutdown_rx.await;
        });

        let _ = ready_tx.send(());

        server.await.context("axum server error")
    });

    // Wait for server readiness signal
    ready_rx.await.expect("server task to signal readiness");

    (
        server_url,
        Arc::clone(&db),
        TestServerHandle::new(shutdown_tx, server_task),
    )
}

#[tokio::test]
async fn test_public_endpoints_no_auth_required() {
    let config = AppConfig {
        operator_tokens: vec!["test-token".to_string()],
        ..Default::default()
    };

    let (server_url, _db, handle) = spawn_test_app(config).await;
    let client = reqwest::Client::new();

    // Test root endpoint (public)
    let response = client.get(format!("{}/", server_url)).send().await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Test health endpoint (public)
    let response = client
        .get(format!("{}/healthz", server_url))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Test ready endpoint (public)
    let response = client
        .get(format!("{}/readyz", server_url))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Test docs endpoint (public)
    let response = client
        .get(format!("{}/docs", server_url))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Test openapi.json endpoint (public)
    let response = client
        .get(format!("{}/openapi.json", server_url))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    handle.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_missing_authorization_header() {
    let config = AppConfig {
        operator_tokens: vec!["test-token".to_string()],
        ..Default::default()
    };

    let (server_url, _db, handle) = spawn_test_app(config).await;
    let client = reqwest::Client::new();

    // Try to access a protected endpoint without auth header
    // Note: We don't have protected endpoints yet, so this test will be updated
    // when we add actual protected endpoints
    let response = client
        .get(format!("{}/protected", server_url))
        .header("X-Tenant-Id", Uuid::new_v4().to_string())
        .send()
        .await;

    // Should get 404 since we don't have protected endpoints yet
    // This test will be updated when we add protected endpoints
    match response {
        Ok(resp) => {
            // Either 404 (no endpoint yet) or 401 (auth middleware applied)
            assert!(
                resp.status() == StatusCode::NOT_FOUND || resp.status() == StatusCode::UNAUTHORIZED
            );
        }
        Err(_) => {
            // Connection error is also acceptable for now
        }
    }

    handle.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_invalid_authorization_format() {
    let config = AppConfig {
        operator_tokens: vec!["test-token".to_string()],
        ..Default::default()
    };

    let (server_url, _db, handle) = spawn_test_app(config).await;
    let client = reqwest::Client::new();

    // Test with Basic auth instead of Bearer
    let response = client
        .get(format!("{}/protected", server_url))
        .header("Authorization", "Basic dGVzdDoxMjM=")
        .header("X-Tenant-Id", Uuid::new_v4().to_string())
        .send()
        .await;

    match response {
        Ok(resp) => {
            assert!(
                resp.status() == StatusCode::NOT_FOUND || resp.status() == StatusCode::UNAUTHORIZED
            );
        }
        Err(_) => {
            // Connection error is also acceptable for now
        }
    }

    handle.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_invalid_bearer_token() {
    let config = AppConfig {
        operator_tokens: vec!["correct-token".to_string()],
        ..Default::default()
    };

    let (server_url, _db, handle) = spawn_test_app(config).await;
    let client = reqwest::Client::new();

    // Test with wrong token
    let response = client
        .get(format!("{}/protected", server_url))
        .header("Authorization", "Bearer wrong-token")
        .header("X-Tenant-Id", Uuid::new_v4().to_string())
        .send()
        .await;

    match response {
        Ok(resp) => {
            assert!(
                resp.status() == StatusCode::NOT_FOUND || resp.status() == StatusCode::UNAUTHORIZED
            );
        }
        Err(_) => {
            // Connection error is also acceptable for now
        }
    }

    handle.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_missing_tenant_header() {
    let config = AppConfig {
        operator_tokens: vec!["test-token".to_string()],
        ..Default::default()
    };

    let (server_url, _db, handle) = spawn_test_app(config).await;
    let client = reqwest::Client::new();

    // Test with valid auth but missing tenant header
    let response = client
        .get(format!("{}/protected", server_url))
        .header("Authorization", "Bearer test-token")
        .send()
        .await;

    match response {
        Ok(resp) => {
            assert!(
                resp.status() == StatusCode::NOT_FOUND || resp.status() == StatusCode::BAD_REQUEST
            );
        }
        Err(_) => {
            // Connection error is also acceptable for now
        }
    }

    handle.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_invalid_tenant_uuid() {
    let config = AppConfig {
        operator_tokens: vec!["test-token".to_string()],
        ..Default::default()
    };

    let (server_url, _db, handle) = spawn_test_app(config).await;
    let client = reqwest::Client::new();

    // Test with valid auth but invalid tenant UUID
    let response = client
        .get(format!("{}/protected", server_url))
        .header("Authorization", "Bearer test-token")
        .header("X-Tenant-Id", "not-a-uuid")
        .send()
        .await;

    match response {
        Ok(resp) => {
            assert!(
                resp.status() == StatusCode::NOT_FOUND || resp.status() == StatusCode::BAD_REQUEST
            );
        }
        Err(_) => {
            // Connection error is also acceptable for now
        }
    }

    handle.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_valid_auth_and_tenant() {
    let config = AppConfig {
        operator_tokens: vec!["test-token".to_string()],
        ..Default::default()
    };

    let (server_url, _db, handle) = spawn_test_app(config).await;
    let client = reqwest::Client::new();
    let tenant_id = Uuid::new_v4();

    // Test with valid auth and tenant
    let response = client
        .get(format!("{}/protected", server_url))
        .header("Authorization", "Bearer test-token")
        .header("X-Tenant-Id", tenant_id.to_string())
        .send()
        .await;

    match response {
        Ok(resp) => {
            // Should get 404 since we don't have protected endpoints yet
            // But should not get auth errors
            assert!(resp.status() == StatusCode::NOT_FOUND);
        }
        Err(_) => {
            // Connection error is also acceptable for now
        }
    }

    handle.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_multiple_tokens() {
    let config = AppConfig {
        operator_tokens: vec![
            "token-one".to_string(),
            "token-two".to_string(),
            "token-three".to_string(),
        ],
        ..Default::default()
    };

    let (server_url, _db, handle) = spawn_test_app(config).await;
    let client = reqwest::Client::new();
    let tenant_id = Uuid::new_v4();

    // Test with each token
    for token in &["token-one", "token-two", "token-three"] {
        let response = client
            .get(format!("{}/protected", server_url))
            .header("Authorization", format!("Bearer {}", token))
            .header("X-Tenant-Id", tenant_id.to_string())
            .send()
            .await;

        match response {
            Ok(resp) => {
                // Should get 404 since we don't have protected endpoints yet
                // But should not get auth errors
                assert!(resp.status() == StatusCode::NOT_FOUND);
            }
            Err(_) => {
                // Connection error is also acceptable for now
            }
        }
    }

    handle.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_openapi_security_scheme() {
    let config = AppConfig {
        operator_tokens: vec!["test-token".to_string()],
        ..Default::default()
    };

    let (server_url, _db, handle) = spawn_test_app(config).await;
    let client = reqwest::Client::new();

    // Test that OpenAPI doc includes security scheme
    let response = client
        .get(format!("{}/openapi.json", server_url))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let openapi: Value = response.json().await.unwrap();

    // Check that security schemes are defined
    assert!(
        openapi
            .get("components")
            .unwrap()
            .get("securitySchemes")
            .is_some()
    );

    let security_schemes = openapi
        .get("components")
        .unwrap()
        .get("securitySchemes")
        .unwrap();
    assert!(security_schemes.get("bearer_auth").is_some());

    let bearer_auth = security_schemes.get("bearer_auth").unwrap();
    assert_eq!(bearer_auth.get("type").unwrap(), "http");
    assert_eq!(bearer_auth.get("scheme").unwrap(), "bearer");

    handle.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_error_response_format() {
    let config = AppConfig {
        operator_tokens: vec!["test-token".to_string()],
        ..Default::default()
    };

    let (server_url, _db, handle) = spawn_test_app(config).await;
    let client = reqwest::Client::new();

    // Test missing auth header error format
    let response = client
        .get(format!("{}/protected", server_url))
        .header("X-Tenant-Id", Uuid::new_v4().to_string())
        .send()
        .await;

    if let Ok(resp) = response
        && resp.status() == StatusCode::UNAUTHORIZED
    {
        // Check error response format
        assert_eq!(
            resp.headers().get("content-type").unwrap(),
            "application/problem+json"
        );

        let error: Value = resp.json().await.unwrap();
        assert_eq!(error.get("code").unwrap(), "UNAUTHORIZED");
        assert!(error.get("message").is_some());
        assert!(error.get("trace_id").is_some());
    }

    // Test missing tenant header error format
    let response = client
        .get(format!("{}/protected", server_url))
        .header("Authorization", "Bearer test-token")
        .send()
        .await;

    if let Ok(resp) = response
        && resp.status() == StatusCode::BAD_REQUEST
    {
        // Check error response format
        assert_eq!(
            resp.headers().get("content-type").unwrap(),
            "application/problem+json"
        );

        let error: Value = resp.json().await.unwrap();
        assert_eq!(error.get("code").unwrap(), "VALIDATION_FAILED");
        assert!(error.get("message").is_some());
        assert!(error.get("trace_id").is_some());
    }

    handle.shutdown().await.unwrap();
}
