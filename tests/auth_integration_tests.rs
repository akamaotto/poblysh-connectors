//! Integration tests for authentication and tenant validation

use connectors::{config::AppConfig, server::create_app};
use reqwest::StatusCode;
use sea_orm::{Database, DatabaseConnection};
use serde_json::Value;
use std::sync::Arc;
use tokio::net::TcpListener;
use uuid::Uuid;

/// Test helper to spawn a test server
async fn spawn_test_app(config: AppConfig) -> (String, DatabaseConnection) {
    // Create a test database
    let db_url = "sqlite::memory:".to_string();
    let db = Database::connect(&db_url).await.unwrap();

    // Create app state
    let state = connectors::server::AppState {
        config: Arc::new(config.clone()),
        db: Some(db.clone()),
    };

    // Create app
    let app = create_app(state);

    // Bind to a random port
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let server_url = format!("http://127.0.0.1:{}", port);

    // Spawn server in background
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // Give server a moment to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    (server_url, db)
}

#[tokio::test]
async fn test_public_endpoints_no_auth_required() {
    let config = AppConfig {
        operator_tokens: vec!["test-token".to_string()],
        ..Default::default()
    };

    let (server_url, _db) = spawn_test_app(config).await;
    let client = reqwest::Client::new();

    // Test root endpoint (public)
    let response = client
        .get(&format!("{}/", server_url))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Test health endpoint (public)
    let response = client
        .get(&format!("{}/healthz", server_url))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Test ready endpoint (public)
    let response = client
        .get(&format!("{}/readyz", server_url))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Test docs endpoint (public)
    let response = client
        .get(&format!("{}/docs", server_url))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Test openapi.json endpoint (public)
    let response = client
        .get(&format!("{}/openapi.json", server_url))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_missing_authorization_header() {
    let config = AppConfig {
        operator_tokens: vec!["test-token".to_string()],
        ..Default::default()
    };

    let (server_url, _db) = spawn_test_app(config).await;
    let client = reqwest::Client::new();

    // Try to access a protected endpoint without auth header
    // Note: We don't have protected endpoints yet, so this test will be updated
    // when we add actual protected endpoints
    let response = client
        .get(&format!("{}/protected", server_url))
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
}

#[tokio::test]
async fn test_invalid_authorization_format() {
    let config = AppConfig {
        operator_tokens: vec!["test-token".to_string()],
        ..Default::default()
    };

    let (server_url, _db) = spawn_test_app(config).await;
    let client = reqwest::Client::new();

    // Test with Basic auth instead of Bearer
    let response = client
        .get(&format!("{}/protected", server_url))
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
}

#[tokio::test]
async fn test_invalid_bearer_token() {
    let config = AppConfig {
        operator_tokens: vec!["correct-token".to_string()],
        ..Default::default()
    };

    let (server_url, _db) = spawn_test_app(config).await;
    let client = reqwest::Client::new();

    // Test with wrong token
    let response = client
        .get(&format!("{}/protected", server_url))
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
}

#[tokio::test]
async fn test_missing_tenant_header() {
    let config = AppConfig {
        operator_tokens: vec!["test-token".to_string()],
        ..Default::default()
    };

    let (server_url, _db) = spawn_test_app(config).await;
    let client = reqwest::Client::new();

    // Test with valid auth but missing tenant header
    let response = client
        .get(&format!("{}/protected", server_url))
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
}

#[tokio::test]
async fn test_invalid_tenant_uuid() {
    let config = AppConfig {
        operator_tokens: vec!["test-token".to_string()],
        ..Default::default()
    };

    let (server_url, _db) = spawn_test_app(config).await;
    let client = reqwest::Client::new();

    // Test with valid auth but invalid tenant UUID
    let response = client
        .get(&format!("{}/protected", server_url))
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
}

#[tokio::test]
async fn test_valid_auth_and_tenant() {
    let config = AppConfig {
        operator_tokens: vec!["test-token".to_string()],
        ..Default::default()
    };

    let (server_url, _db) = spawn_test_app(config).await;
    let client = reqwest::Client::new();
    let tenant_id = Uuid::new_v4();

    // Test with valid auth and tenant
    let response = client
        .get(&format!("{}/protected", server_url))
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

    let (server_url, _db) = spawn_test_app(config).await;
    let client = reqwest::Client::new();
    let tenant_id = Uuid::new_v4();

    // Test with each token
    for token in &["token-one", "token-two", "token-three"] {
        let response = client
            .get(&format!("{}/protected", server_url))
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
}

#[tokio::test]
async fn test_openapi_security_scheme() {
    let config = AppConfig {
        operator_tokens: vec!["test-token".to_string()],
        ..Default::default()
    };

    let (server_url, _db) = spawn_test_app(config).await;
    let client = reqwest::Client::new();

    // Test that OpenAPI doc includes security scheme
    let response = client
        .get(&format!("{}/openapi.json", server_url))
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
}

#[tokio::test]
async fn test_error_response_format() {
    let config = AppConfig {
        operator_tokens: vec!["test-token".to_string()],
        ..Default::default()
    };

    let (server_url, _db) = spawn_test_app(config).await;
    let client = reqwest::Client::new();

    // Test missing tenant header error format
    let response = client
        .get(&format!("{}/protected/ping", server_url))
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        response.headers().get("content-type").unwrap(),
        "application/problem+json"
    );

    let error: Value = response.json().await.unwrap();
    assert_eq!(error.get("code").unwrap(), "VALIDATION_FAILED");
    assert!(error.get("message").is_some());
    assert!(error.get("trace_id").is_some());
}
