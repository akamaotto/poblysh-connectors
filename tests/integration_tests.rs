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

/// Integration tests for the /signals endpoint
mod signals_tests {
    use super::*;
    use chrono::Utc;
    use connectors::config::AppConfig;
    use connectors::db::init_pool;
    use connectors::models::connection::ActiveModel as ConnectionActiveModel;
    use connectors::models::provider::ActiveModel as ProviderActiveModel;
    use connectors::models::signal::ActiveModel as SignalActiveModel;
    use connectors::models::tenant::ActiveModel as TenantActiveModel;
    use migration::MigratorTrait;
    use sea_orm::ActiveModelTrait;
    use serde_json::json;
    use uuid::Uuid;

    /// Helper function to start the server with a real database
    async fn start_test_server_with_db() -> (String, DatabaseConnection) {
        let port = get_available_port().await;
        let addr = SocketAddr::from(([127, 0, 0, 1], port));

        // Create test configuration
        let config = AppConfig {
            profile: "test".to_string(),
            operator_tokens: vec!["test-token".to_string()],
            ..Default::default()
        };

        // Initialize database
        let db = init_pool(&config).await.expect("Failed to init test DB");

        // Apply migrations
        migration::Migrator::up(&db, None)
            .await
            .expect("Failed to apply migrations");

        // Create crypto key
        let crypto_key = connectors::crypto::CryptoKey::new(vec![0u8; 32])
            .expect("Failed to create test crypto key");

        let state = AppState {
            config: std::sync::Arc::new(config),
            db: db.clone(),
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

        (format!("http://127.0.0.1:{}", port), db)
    }

    /// Helper function to create test data
    async fn create_test_data(db: &DatabaseConnection) -> (Uuid, Uuid, String) {
        // Create tenant
        let tenant_id = Uuid::new_v4();
        let tenant = TenantActiveModel {
            id: sea_orm::Set(tenant_id),
            ..Default::default()
        };
        tenant.insert(db).await.unwrap();

        // Create provider
        let provider = ProviderActiveModel {
            slug: sea_orm::Set(format!(
                "test-provider-{}",
                &Uuid::new_v4().to_string()[..8]
            )),
            display_name: sea_orm::Set("Test Provider".to_string()),
            auth_type: sea_orm::Set("oauth".to_string()),
            ..Default::default()
        };
        let provider_slug = provider.slug.clone().unwrap();
        provider.insert(db).await.unwrap();

        // Create connection
        let connection_id = Uuid::new_v4();
        let connection = ConnectionActiveModel {
            id: sea_orm::Set(connection_id),
            tenant_id: sea_orm::Set(tenant_id),
            provider_slug: sea_orm::Set(provider_slug.clone()),
            external_id: sea_orm::Set("ext-123".to_string()),
            ..Default::default()
        };
        connection.insert(db).await.unwrap();

        (tenant_id, connection_id, provider_slug)
    }

    /// Helper function to create test signals
    async fn create_test_signals(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        connection_id: Uuid,
        provider_slug: &str,
        count: usize,
    ) {
        let now = Utc::now();

        for i in 0..count {
            let signal = SignalActiveModel {
                id: sea_orm::Set(Uuid::new_v4()),
                tenant_id: sea_orm::Set(tenant_id),
                provider_slug: sea_orm::Set(provider_slug.to_string()),
                connection_id: sea_orm::Set(connection_id),
                kind: sea_orm::Set(format!("test_event_{}", i)),
                occurred_at: sea_orm::Set((now - chrono::Duration::seconds(i as i64)).into()),
                received_at: sea_orm::Set((now - chrono::Duration::seconds(i as i64)).into()),
                payload: sea_orm::Set(json!({"test": i, "data": "example"})),
                ..Default::default()
            };
            signal.insert(db).await.unwrap();
        }
    }

    #[tokio::test]
    async fn test_signals_endpoint_requires_authentication() {
        let (server_url, _db) = start_test_server_with_db().await;
        let client = Client::new();

        let response = client
            .get(format!("{}/signals", server_url))
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(response.status(), 401);

        let body: Value = response.json().await.expect("Failed to parse JSON");
        assert_eq!(body.get("code").unwrap().as_str().unwrap(), "UNAUTHORIZED");
    }

    #[tokio::test]
    async fn test_signals_endpoint_requires_tenant_header() {
        let (server_url, _db) = start_test_server_with_db().await;
        let client = Client::new();

        let response = client
            .get(format!("{}/signals", server_url))
            .header("Authorization", "Bearer test-token")
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(response.status(), 400);

        let body: Value = response.json().await.expect("Failed to parse JSON");
        assert_eq!(
            body.get("code").unwrap().as_str().unwrap(),
            "VALIDATION_FAILED"
        );
    }

    #[tokio::test]
    async fn test_signals_endpoint_success() {
        let (server_url, db) = start_test_server_with_db().await;
        let (tenant_id, connection_id, provider_slug) = create_test_data(&db).await;
        create_test_signals(&db, tenant_id, connection_id, &provider_slug, 3).await;

        let client = Client::new();
        let tenant_id_str = tenant_id.to_string();

        let response = client
            .get(format!("{}/signals", server_url))
            .header("Authorization", "Bearer test-token")
            .header("X-Tenant-Id", tenant_id_str)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(response.status(), 200);

        let body: Value = response.json().await.expect("Failed to parse JSON");
        let signals = body.get("signals").unwrap().as_array().unwrap();
        assert_eq!(signals.len(), 3);

        // Should be ordered by occurred_at DESC (newest first)
        assert_eq!(
            signals[0].get("kind").unwrap().as_str().unwrap(),
            "test_event_0"
        );
        assert_eq!(
            signals[1].get("kind").unwrap().as_str().unwrap(),
            "test_event_1"
        );
        assert_eq!(
            signals[2].get("kind").unwrap().as_str().unwrap(),
            "test_event_2"
        );

        // Should have next_cursor for more results
        assert!(body.get("next_cursor").is_some());
    }

    #[tokio::test]
    async fn test_signals_endpoint_with_limit() {
        let (server_url, db) = start_test_server_with_db().await;
        let (tenant_id, connection_id, provider_slug) = create_test_data(&db).await;
        create_test_signals(&db, tenant_id, connection_id, &provider_slug, 5).await;

        let client = Client::new();
        let tenant_id_str = tenant_id.to_string();

        let response = client
            .get(format!("{}/signals?limit=2", server_url))
            .header("Authorization", "Bearer test-token")
            .header("X-Tenant-Id", tenant_id_str)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(response.status(), 200);

        let body: Value = response.json().await.expect("Failed to parse JSON");
        let signals = body.get("signals").unwrap().as_array().unwrap();
        assert_eq!(signals.len(), 2);

        // Should have next_cursor since there are more results
        let next_cursor = body.get("next_cursor").unwrap().as_str().unwrap();
        assert!(!next_cursor.is_empty());
    }

    #[tokio::test]
    async fn test_signals_endpoint_with_provider_filter() {
        let (server_url, db) = start_test_server_with_db().await;
        let (tenant_id, connection_id, provider_slug) = create_test_data(&db).await;
        create_test_signals(&db, tenant_id, connection_id, &provider_slug, 3).await;

        let client = Client::new();
        let tenant_id_str = tenant_id.to_string();

        let response = client
            .get(format!("{}/signals?provider={}", server_url, provider_slug))
            .header("Authorization", "Bearer test-token")
            .header("X-Tenant-Id", tenant_id_str)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(response.status(), 200);

        let body: Value = response.json().await.expect("Failed to parse JSON");
        let signals = body.get("signals").unwrap().as_array().unwrap();
        assert_eq!(signals.len(), 3);

        // All signals should be from the specified provider
        for signal in signals {
            assert_eq!(
                signal.get("provider_slug").unwrap().as_str().unwrap(),
                provider_slug
            );
        }
    }

    #[tokio::test]
    async fn test_signals_endpoint_with_kind_filter() {
        let (server_url, db) = start_test_server_with_db().await;
        let (tenant_id, connection_id, provider_slug) = create_test_data(&db).await;

        // Create signals with different kinds
        let now = Utc::now();
        for kind in ["issue_created", "pr_merged", "issue_commented"] {
            let signal = SignalActiveModel {
                id: sea_orm::Set(Uuid::new_v4()),
                tenant_id: sea_orm::Set(tenant_id),
                provider_slug: sea_orm::Set(provider_slug.clone()),
                connection_id: sea_orm::Set(connection_id),
                kind: sea_orm::Set(kind.to_string()),
                occurred_at: sea_orm::Set(now.into()),
                received_at: sea_orm::Set(now.into()),
                payload: sea_orm::Set(json!({"event": kind})),
                ..Default::default()
            };
            signal.insert(&db).await.unwrap();
        }

        let client = Client::new();
        let tenant_id_str = tenant_id.to_string();

        let response = client
            .get(format!("{}/signals?kind=issue_created", server_url))
            .header("Authorization", "Bearer test-token")
            .header("X-Tenant-Id", tenant_id_str)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(response.status(), 200);

        let body: Value = response.json().await.expect("Failed to parse JSON");
        let signals = body.get("signals").unwrap().as_array().unwrap();
        assert_eq!(signals.len(), 1);

        // Should only return signals with the specified kind
        assert_eq!(
            signals[0].get("kind").unwrap().as_str().unwrap(),
            "issue_created"
        );
    }

    #[tokio::test]
    async fn test_signals_endpoint_with_invalid_limit() {
        let (server_url, _db) = start_test_server_with_db().await;
        let client = Client::new();

        let response = client
            .get(format!("{}/signals?limit=101", server_url))
            .header("Authorization", "Bearer test-token")
            .header("X-Tenant-Id", Uuid::new_v4().to_string())
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(response.status(), 400);

        let body: Value = response.json().await.expect("Failed to parse JSON");
        assert_eq!(
            body.get("code").unwrap().as_str().unwrap(),
            "VALIDATION_FAILED"
        );
        assert!(
            body.get("message")
                .unwrap()
                .as_str()
                .unwrap()
                .contains("limit must be between 1 and 100")
        );
    }

    #[tokio::test]
    async fn test_signals_endpoint_with_invalid_connection_id() {
        let (server_url, _db) = start_test_server_with_db().await;
        let client = Client::new();

        let response = client
            .get(format!("{}/signals?connection_id=invalid-uuid", server_url))
            .header("Authorization", "Bearer test-token")
            .header("X-Tenant-Id", Uuid::new_v4().to_string())
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(response.status(), 400);

        let body: Value = response.json().await.expect("Failed to parse JSON");
        assert_eq!(
            body.get("code").unwrap().as_str().unwrap(),
            "VALIDATION_FAILED"
        );
        assert!(
            body.get("message")
                .unwrap()
                .as_str()
                .unwrap()
                .contains("connection_id must be a valid UUID")
        );
    }

    #[tokio::test]
    async fn test_signals_endpoint_with_time_filter() {
        let (server_url, db) = start_test_server_with_db().await;
        let (tenant_id, connection_id, provider_slug) = create_test_data(&db).await;

        // Create signals at specific times
        let base_time = Utc::now();
        let signal1_time = base_time - chrono::Duration::hours(2);
        let signal2_time = base_time - chrono::Duration::hours(1);

        for (i, time) in [signal1_time, signal2_time].iter().enumerate() {
            let signal = SignalActiveModel {
                id: sea_orm::Set(Uuid::new_v4()),
                tenant_id: sea_orm::Set(tenant_id),
                provider_slug: sea_orm::Set(provider_slug.clone()),
                connection_id: sea_orm::Set(connection_id),
                kind: sea_orm::Set(format!("timed_event_{}", i)),
                occurred_at: sea_orm::Set((*time).into()),
                received_at: sea_orm::Set(base_time.into()),
                payload: sea_orm::Set(json!({"index": i})),
                ..Default::default()
            };
            signal.insert(&db).await.unwrap();
        }

        let client = Client::new();
        let tenant_id_str = tenant_id.to_string();

        // Filter for signals after signal1_time but before base_time
        let filter_start = signal1_time + chrono::Duration::minutes(30);
        let response = client
            .get(format!(
                "{}/signals?occurred_after={}&occurred_before={}",
                server_url,
                filter_start.to_rfc3339(),
                base_time.to_rfc3339()
            ))
            .header("Authorization", "Bearer test-token")
            .header("X-Tenant-Id", tenant_id_str)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(response.status(), 200);

        let body: Value = response.json().await.expect("Failed to parse JSON");
        let signals = body.get("signals").unwrap().as_array().unwrap();
        assert_eq!(signals.len(), 1);

        // Should return the signal that falls within the time range
        assert_eq!(
            signals[0].get("kind").unwrap().as_str().unwrap(),
            "timed_event_1"
        );
    }

    #[tokio::test]
    async fn test_signals_endpoint_tenant_isolation() {
        let (server_url, db) = start_test_server_with_db().await;

        // Create two tenants with their own connections and signals
        let (tenant1_id, connection1_id, provider_slug1) = create_test_data(&db).await;
        let (tenant2_id, connection2_id, provider_slug2) = create_test_data(&db).await;

        // Create signals for tenant 1
        create_test_signals(&db, tenant1_id, connection1_id, &provider_slug1, 2).await;

        // Create signals for tenant 2
        create_test_signals(&db, tenant2_id, connection2_id, &provider_slug2, 2).await;

        let client = Client::new();

        // Query as tenant 1 - should only see tenant 1 signals
        let response = client
            .get(format!("{}/signals", server_url))
            .header("Authorization", "Bearer test-token")
            .header("X-Tenant-Id", tenant1_id.to_string())
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(response.status(), 200);

        let body: Value = response.json().await.expect("Failed to parse JSON");
        let signals = body.get("signals").unwrap().as_array().unwrap();
        assert_eq!(signals.len(), 2);

        // All signals should belong to tenant 1's connection
        for signal in signals {
            assert_eq!(
                signal.get("connection_id").unwrap().as_str().unwrap(),
                connection1_id.to_string()
            );
        }
    }

    #[tokio::test]
    async fn test_signals_endpoint_pagination_flow() {
        let (server_url, db) = start_test_server_with_db().await;
        let (tenant_id, connection_id, provider_slug) = create_test_data(&db).await;
        create_test_signals(&db, tenant_id, connection_id, &provider_slug, 5).await;

        let client = Client::new();
        let tenant_id_str = tenant_id.to_string();

        // First page with limit 2
        let response1 = client
            .get(format!("{}/signals?limit=2", server_url))
            .header("Authorization", "Bearer test-token")
            .header("X-Tenant-Id", tenant_id_str.clone())
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(response1.status(), 200);

        let body1: Value = response1.json().await.expect("Failed to parse JSON");
        let signals1 = body1.get("signals").unwrap().as_array().unwrap();
        assert_eq!(signals1.len(), 2);

        let next_cursor = body1.get("next_cursor").unwrap().as_str().unwrap();
        assert!(!next_cursor.is_empty());

        // Second page using cursor
        let response2 = client
            .get(format!(
                "{}/signals?limit=2&cursor={}",
                server_url, next_cursor
            ))
            .header("Authorization", "Bearer test-token")
            .header("X-Tenant-Id", tenant_id_str.clone())
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(response2.status(), 200);

        let body2: Value = response2.json().await.expect("Failed to parse JSON");
        let signals2 = body2.get("signals").unwrap().as_array().unwrap();
        assert_eq!(signals2.len(), 2);

        // Should get different signals than first page
        let first_page_ids: std::collections::HashSet<_> = signals1
            .iter()
            .map(|s| s.get("id").unwrap().as_str().unwrap())
            .collect();
        let second_page_ids: std::collections::HashSet<_> = signals2
            .iter()
            .map(|s| s.get("id").unwrap().as_str().unwrap())
            .collect();

        assert!(
            first_page_ids
                .intersection(&second_page_ids)
                .next()
                .is_none()
        );

        // Third page - should get remaining signal
        let next_cursor2 = body2.get("next_cursor").unwrap().as_str().unwrap();
        assert!(!next_cursor2.is_empty());

        let response3 = client
            .get(format!(
                "{}/signals?limit=2&cursor={}",
                server_url, next_cursor2
            ))
            .header("Authorization", "Bearer test-token")
            .header("X-Tenant-Id", tenant_id_str)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(response3.status(), 200);

        let body3: Value = response3.json().await.expect("Failed to parse JSON");
        let signals3 = body3.get("signals").unwrap().as_array().unwrap();
        assert_eq!(signals3.len(), 1);

        // Last page should have null next_cursor
        assert_eq!(body3.get("next_cursor").unwrap(), &serde_json::Value::Null);
    }
}
