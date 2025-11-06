//! Integration tests for OAuth callback endpoint
//!
//! These tests verify the OAuth callback functionality end-to-end, including:
//! - State replay protection when provider denies authorization
//! - Malformed response error envelope formatting
//! - Structured error handling for upstream provider failures

use anyhow::{Context, Result as AnyhowResult};
use connectors::connectors::Registry;
use connectors::repositories::ConnectionRepository;
use connectors::token_refresh::TokenRefreshService;
use connectors::{
    config::AppConfig, repositories::oauth_state::OAuthStateRepository, server::create_app,
};
use reqwest::StatusCode;
use sea_orm::{ConnectionTrait, DatabaseConnection, Statement};
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

/// Test helper to spawn a test server
async fn spawn_test_app(config: AppConfig) -> (String, Arc<DatabaseConnection>, TestServerHandle) {
    // Create a test database with migrations
    let db = Arc::new(test_utils::setup_test_db().await.unwrap());

    // Initialize the connector registry (required for OAuth callback tests)
    connectors::connectors::Registry::initialize();

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

/// Helper to create a tenant for testing
async fn create_test_tenant(db: &DatabaseConnection) -> AnyhowResult<Uuid> {
    let tenant_id = Uuid::new_v4();
    let stmt = Statement::from_string(
        db.get_database_backend(),
        format!(
            "INSERT INTO tenants (id, name, created_at) VALUES ('{}', 'Test Tenant', datetime('now'))",
            tenant_id
        ),
    );

    db.execute(stmt).await?;
    Ok(tenant_id)
}

/// Helper to check if OAuth state exists using direct SQL
async fn oauth_state_exists(db: &DatabaseConnection, provider: &str, state: &str) -> bool {
    let stmt = Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        format!(
            "SELECT COUNT(*) as count FROM oauth_states WHERE provider = '{}' AND state = '{}'",
            provider, state
        ),
    );

    match db.query_one(stmt).await {
        Ok(Some(result)) => {
            let count = result.try_get::<i64>("", "count").unwrap_or(0);
            println!(
                "oauth_state_exists: provider={}, state={}, count={}",
                provider, state, count
            );
            count > 0
        }
        Err(e) => {
            println!("oauth_state_exists query error: {:?}", e);
            false
        }
        Ok(None) => {
            println!("oauth_state_exists: no result returned");
            false
        }
    }
}

/// Helper to create OAuth state that expires

#[tokio::test]
async fn test_oauth_callback_provider_denied_replay_protection()
-> Result<(), Box<dyn std::error::Error>> {
    // Test that state is consumed when provider denies authorization
    // and subsequent attempts with the same state fail (replay protection)
    let config = AppConfig {
        operator_tokens: vec!["test-token".to_string()],
        ..Default::default()
    };

    let (server_url, db, handle) = spawn_test_app(config).await;
    let client = reqwest::Client::new();

    // Create test tenant
    let tenant_id = create_test_tenant(&*db).await?;
    let state_token = "test_replay_state_token_12345";
    println!("Created tenant: {}", tenant_id);

    // Create OAuth state using the repository (proper timestamp handling)
    let oauth_state_repo = OAuthStateRepository::new(db.clone());
    oauth_state_repo
        .create(tenant_id, "example", state_token, None, 15)
        .await?;
    println!(
        "Created OAuth state for tenant: {}, provider: example, state: {}",
        tenant_id, state_token
    );

    // Verify the state was actually created
    let exists = oauth_state_exists(&*db, "example", state_token).await;
    println!("OAuth state exists immediately after creation: {}", exists);

    // First callback attempt - provider denies authorization
    let response = client
        .get(&format!(
            "{}/connect/example/callback?code=some_code&state={}&error=access_denied",
            server_url, state_token
        ))
        .send()
        .await?;

    // Should return 400 for provider denial
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let error_response: Value = response.json().await?;
    assert_eq!(error_response["code"], "VALIDATION_FAILED");
    assert!(
        error_response["message"]
            .as_str()
            .unwrap()
            .contains("provider denied authorization")
    );

    // Verify state was consumed
    assert!(
        !oauth_state_exists(&*db, "example", state_token).await,
        "State should be consumed"
    );

    // Second callback attempt with same state (replay attempt)
    let replay_response = client
        .get(&format!(
            "{}/connect/example/callback?code=another_code&state={}",
            server_url, state_token
        ))
        .send()
        .await?;

    // Should return 400 for invalid/reused state
    assert_eq!(replay_response.status(), StatusCode::BAD_REQUEST);

    let replay_error: Value = replay_response.json().await?;
    assert_eq!(replay_error["code"], "VALIDATION_FAILED");
    assert!(
        replay_error["message"]
            .as_str()
            .unwrap()
            .contains("invalid state")
    );

    handle.shutdown().await?;
    println!("✓ OAuth callback provider denied replay protection integration test passed");
    Ok(())
}

#[tokio::test]
async fn test_oauth_callback_malformed_response_error_envelope()
-> Result<(), Box<dyn std::error::Error>> {
    // Test that malformed response from connector returns properly formatted PROVIDER_ERROR envelope
    let config = AppConfig {
        operator_tokens: vec!["test-token".to_string()],
        ..Default::default()
    };

    let (server_url, db, handle) = spawn_test_app(config).await;
    let client = reqwest::Client::new();

    // Create test tenant
    let tenant_id = create_test_tenant(&*db).await?;
    let state_token = "test_malformed_response_state_12345";

    // Create OAuth state using the repository (proper timestamp handling)
    let oauth_state_repo = OAuthStateRepository::new(db.clone());
    oauth_state_repo
        .create(tenant_id, "example", state_token, None, 15)
        .await?;

    // Callback with malformed response trigger code
    let response = client
        .get(&format!(
            "{}/connect/example/callback?code=test_malformed_response&state={}",
            server_url, state_token
        ))
        .send()
        .await?;

    // Should return 502 for malformed response
    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);

    let error_response: Value = response.json().await?;

    // Verify basic error structure
    assert_eq!(error_response["code"], "PROVIDER_ERROR");
    assert!(
        error_response["message"]
            .as_str()
            .unwrap()
            .contains("malformed response")
    );
    assert!(error_response["details"].is_object());

    // Verify detailed error envelope structure
    let details = &error_response["details"];
    assert!(details["provider"].is_object());

    let provider_info = &details["provider"];
    assert_eq!(provider_info["name"], "example");
    assert_eq!(provider_info["error_type"], "malformed_response");
    assert!(provider_info["message"].is_string());
    assert!(provider_info["response_body"].is_string());
    assert_eq!(provider_info["status"], 502);

    // Verify trace_id is present for debugging
    assert!(error_response["trace_id"].is_string());
    assert!(!error_response["trace_id"].as_str().unwrap().is_empty());

    handle.shutdown().await?;
    println!("✓ OAuth callback malformed response error envelope integration test passed");
    Ok(())
}

#[tokio::test]
async fn test_oauth_callback_expired_state_validation() -> Result<(), Box<dyn std::error::Error>> {
    // Test that expired state tokens are properly rejected
    let config = AppConfig {
        operator_tokens: vec!["test-token".to_string()],
        ..Default::default()
    };

    let (server_url, db, handle) = spawn_test_app(config).await;
    let client = reqwest::Client::new();

    // Create test tenant
    let tenant_id = create_test_tenant(&*db).await?;
    let state_token = "test_expired_state_token_12345";

    // Create an expired OAuth state by using repository with negative expiration
    let oauth_state_repo = OAuthStateRepository::new(db.clone());

    // Create state with -60 minutes expiration (should be expired immediately)
    let created_state = oauth_state_repo
        .create(tenant_id, "example", state_token, None, -60)
        .await?;
    println!(
        "Created expired state with expires_at: {}",
        created_state.expires_at
    );

    // Verify it's actually expired by checking if current time is past expires_at
    let now = chrono::Utc::now();
    let is_expired = now > created_state.expires_at;
    println!("Current time: {}, Is state expired: {}", now, is_expired);

    // Verify the expired state was created
    let exists = oauth_state_exists(&*db, "example", state_token).await;
    println!(
        "Expired OAuth state exists immediately after creation: {}",
        exists
    );

    // Callback with expired state
    let response = client
        .get(&format!(
            "{}/connect/example/callback?code=some_code&state={}",
            server_url, state_token
        ))
        .send()
        .await?;

    let status = response.status();
    println!("Response status for expired state test: {}", status);

    // Should return 400 for expired state
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let error_response: Value = response.json().await?;
    println!(
        "Error response: {}",
        serde_json::to_string_pretty(&error_response).unwrap()
    );
    assert_eq!(error_response["code"], "VALIDATION_FAILED");
    assert!(
        error_response["message"]
            .as_str()
            .unwrap()
            .contains("missing, expired, or invalid state parameter")
    );

    // Expired state validation is working - no detailed error type needed for validation failures

    handle.shutdown().await?;
    println!("✓ OAuth callback expired state validation integration test passed");
    Ok(())
}

#[tokio::test]
async fn test_oauth_callback_detailed_http_error_info() -> Result<(), Box<dyn std::error::Error>> {
    // Test that HTTP errors from upstream providers include detailed information
    let config = AppConfig {
        operator_tokens: vec!["test-token".to_string()],
        ..Default::default()
    };

    let (server_url, db, handle) = spawn_test_app(config).await;
    let client = reqwest::Client::new();

    // Create test tenant
    let tenant_id = create_test_tenant(&*db).await?;
    let state_token = "test_http_error_state_12345";

    // Create OAuth state using the repository (proper timestamp handling)
    let oauth_state_repo = OAuthStateRepository::new(db.clone());
    oauth_state_repo
        .create(tenant_id, "example", state_token, None, 15)
        .await?;

    // Callback with HTTP error trigger code
    let response = client
        .get(&format!(
            "{}/connect/example/callback?code=test_http_error&state={}",
            server_url, state_token
        ))
        .send()
        .await?;

    // Should return 502 for HTTP error
    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);

    let error_response: Value = response.json().await?;

    // Verify basic error structure
    assert_eq!(error_response["code"], "PROVIDER_ERROR");
    assert!(
        error_response["message"]
            .as_str()
            .unwrap()
            .contains("HTTP 500")
    );
    assert!(error_response["details"].is_object());

    // Verify detailed error envelope structure
    let details = &error_response["details"];
    assert!(details["provider"].is_object());

    let provider_info = &details["provider"];
    assert_eq!(provider_info["name"], "example");
    assert_eq!(provider_info["error_type"], "http_error");
    assert_eq!(provider_info["status"], 500);
    assert_eq!(provider_info["message"], "HTTP 500 error");
    assert_eq!(provider_info["response_body"], "Internal Server Error");

    // Verify response headers are included
    assert!(provider_info["response_headers"].is_array());

    // Verify trace_id is present for debugging
    assert!(error_response["trace_id"].is_string());
    assert!(!error_response["trace_id"].as_str().unwrap().is_empty());

    handle.shutdown().await?;
    println!("✓ OAuth callback detailed HTTP error info integration test passed");
    Ok(())
}
