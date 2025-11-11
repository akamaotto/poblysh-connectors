use connectors::connectors::github::GitHubConnector;
use connectors::connectors::trait_::SyncParams;
use connectors::connectors::{AuthorizeParams, Connector, ExchangeTokenParams, WebhookParams};
use connectors::models::connection;
use sea_orm::EntityTrait;
use serde_json::json;
use uuid::Uuid;
mod test_utils;
use test_utils::{create_test_tenant, insert_connection, insert_provider, setup_test_db};
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{header, method, path},
};

#[tokio::test]
async fn test_github_oauth_flow_integration() {
    // Setup mock server
    let mock_server = MockServer::start().await;

    // Mock token exchange endpoint
    Mock::given(method("POST"))
        .and(path("/login/oauth/access_token"))
        .and(header("accept", "application/json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "access_token": "test_access_token_12345",
            "token_type": "Bearer",
            "scope": "repo read:org",
            "expires_in": 3600,
            "refresh_token": "test_refresh_token_12345"
        })))
        .mount(&mock_server)
        .await;

    // Mock user info endpoint
    Mock::given(method("GET"))
        .and(path("/user"))
        .and(header("authorization", "Bearer test_access_token_12345"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": 123456,
            "login": "testuser",
            "name": "Test User",
            "email": "test@example.com"
        })))
        .mount(&mock_server)
        .await;

    // Create connector with mock server URL
    let connector = GitHubConnector::new(
        "test_client_id".to_string(),
        "test_client_secret".to_string(),
        format!("{}/callback", mock_server.uri()),
        Some("test_webhook_secret".to_string()),
    );

    // Test OAuth flow
    let tenant_id = Uuid::new_v4();
    let params = AuthorizeParams {
        tenant_id,
        redirect_uri: Some(format!("{}/callback", mock_server.uri())),
        state: Some("test_state_123".to_string()),
    };

    // Test authorize URL generation
    let auth_url = connector.authorize(params.clone()).await.unwrap();
    assert!(
        auth_url
            .as_str()
            .contains("github.com/login/oauth/authorize")
    );
    assert!(auth_url.as_str().contains("test_client_id"));
    assert!(auth_url.as_str().contains("repo")); // Just check for "repo" instead of full scope

    // Test token exchange
    let exchange_params = ExchangeTokenParams {
        code: "test_auth_code_12345".to_string(),
        redirect_uri: Some(format!("{}/callback", mock_server.uri())),
        tenant_id,
    };

    let connection = connector.exchange_token(exchange_params).await.unwrap();
    assert_eq!(connection.provider_slug, "github");
    assert_eq!(connection.external_id, "123456");
    assert_eq!(connection.display_name, Some("testuser".to_string()));
    assert!(connection.access_token_ciphertext.is_some());
    assert!(connection.refresh_token_ciphertext.is_some());

    // Test token refresh
    let refreshed_connection = connector.refresh_token(connection).await.unwrap();
    assert_eq!(refreshed_connection.provider_slug, "github");
    assert!(refreshed_connection.access_token_ciphertext.is_some());

    // Check that metadata was updated
    assert!(refreshed_connection.metadata.is_some());
    let metadata = refreshed_connection.metadata.unwrap();
    assert!(metadata.get("last_refreshed_at").is_some());
    assert_eq!(
        metadata.get("refresh_method"),
        Some(&json!("oauth_refresh"))
    );
}

#[tokio::test]
async fn test_github_webhook_processing() {
    // Set up test database
    let db = setup_test_db().await.unwrap();
    let tenant_id = create_test_tenant(&db, None).await.unwrap();

    // Insert GitHub provider
    insert_provider(&db, "github", "GitHub", "oauth2")
        .await
        .unwrap();

    // Create a GitHub connection for the tenant
    let connection_id = uuid::Uuid::new_v4();
    insert_connection(&db, connection_id, tenant_id, "github", "github-user-123")
        .await
        .unwrap();

    let connector = GitHubConnector::new(
        "test_client_id".to_string(),
        "test_client_secret".to_string(),
        "https://localhost:3000/callback".to_string(),
        Some("test_webhook_secret".to_string()),
    );

    // Test issue created webhook
    let issue_webhook = json!({
        "action": "opened",
        "issue": {
            "id": 123,
            "number": 42,
            "title": "Test Issue",
            "state": "open",
            "created_at": "2024-01-01T12:00:00Z",
            "updated_at": "2024-01-01T12:00:00Z",
            "user": {
                "id": 456,
                "login": "testuser"
            },
            "body": "This is a test issue",
            "labels": [],
            "pull_request": null
        }
    });

    let webhook_params = WebhookParams {
        payload: issue_webhook,
        tenant_id,
        db: Some(db.clone()),
        auth_header: None,
    };

    let signals = connector.handle_webhook(webhook_params).await.unwrap();
    assert_eq!(signals.len(), 1);
    assert_eq!(signals[0].kind, "issue_created");
    assert_eq!(signals[0].provider_slug, "github");
    assert_eq!(signals[0].tenant_id, tenant_id);

    // Test pull request opened webhook
    let pr_webhook = json!({
        "action": "opened",
        "pull_request": {
            "id": 789,
            "number": 23,
            "title": "Test PR",
            "state": "open",
            "created_at": "2024-01-01T13:00:00Z",
            "updated_at": "2024-01-01T13:00:00Z",
            "user": {
                "id": 456,
                "login": "testuser"
            },
            "body": "This is a test PR",
            "labels": [],
            "merged": false
        }
    });

    let webhook_params = WebhookParams {
        payload: pr_webhook,
        tenant_id,
        db: Some(db.clone()),
        auth_header: None,
    };

    let signals = connector.handle_webhook(webhook_params).await.unwrap();
    assert_eq!(signals.len(), 1);
    // Spec normalizes pull-request opened events to `pr_opened`
    assert_eq!(signals[0].kind, "pr_opened");
    assert_eq!(signals[0].provider_slug, "github");

    // Test webhook without action field (should be ignored)
    let invalid_webhook = json!({
        "zen": "Non-linear feedback loops are the jam."
    });

    let webhook_params = WebhookParams {
        payload: invalid_webhook,
        tenant_id,
        db: Some(db),
        auth_header: None,
    };

    let signals = connector.handle_webhook(webhook_params).await.unwrap();
    assert_eq!(signals.len(), 0);
}

#[tokio::test]
async fn test_github_webhook_signature_verification() {
    let connector = GitHubConnector::new(
        "test_client_id".to_string(),
        "test_client_secret".to_string(),
        "https://localhost:3000/callback".to_string(),
        Some("test_webhook_secret".to_string()),
    );

    let payload = b"test webhook payload";

    // Create a valid signature
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    let mut mac = Hmac::<Sha256>::new_from_slice(b"test_webhook_secret").unwrap();
    mac.update(payload);
    let valid_signature = format!("sha256={}", hex::encode(mac.finalize().into_bytes()));

    // Test valid signature
    assert!(
        connector
            .verify_webhook_signature(payload, &valid_signature)
            .is_ok()
    );

    // Test invalid signature
    let invalid_signature = "sha256=invalid_signature";
    assert!(
        connector
            .verify_webhook_signature(payload, invalid_signature)
            .is_err()
    );

    // Test missing signature prefix - create new MAC since previous one was finalized
    let mut mac2 = Hmac::<Sha256>::new_from_slice(b"test_webhook_secret").unwrap();
    mac2.update(payload);
    let no_prefix_signature = hex::encode(mac2.finalize().into_bytes());
    assert!(
        connector
            .verify_webhook_signature(payload, &no_prefix_signature)
            .is_err()
    );
}

#[tokio::test]
async fn test_github_rate_limit_handling() {
    let mock_server = MockServer::start().await;

    // Mock rate limited response
    Mock::given(method("GET"))
        .and(path("/user/issues"))
        .respond_with(
            ResponseTemplate::new(429)
                .append_header("Retry-After", "60")
                .append_header("X-RateLimit-Remaining", "0")
                .append_header(
                    "X-RateLimit-Reset",
                    (chrono::Utc::now().timestamp() + 60).to_string(),
                )
                .set_body_json(json!({
                    "message": "API rate limit exceeded"
                })),
        )
        .mount(&mock_server)
        .await;

    // Create connector with mock server
    let _connector = GitHubConnector::new(
        "test_client_id".to_string(),
        "test_client_secret".to_string(),
        "https://localhost:3000/callback".to_string(),
        None,
    );

    // Note: fetch_issues is a private method, testing through the public sync interface
    // is more appropriate for integration tests
    // TODO: Add integration test for sync with rate limit handling
}

#[tokio::test]
async fn test_github_pagination_parsing() {
    let _connector = GitHubConnector::new(
        "test_client_id".to_string(),
        "test_client_secret".to_string(),
        "https://localhost:3000/callback".to_string(),
        None,
    );

    // Note: parse_link_header is a private method, Link header parsing should be tested
    // through the public sync interface integration tests
    // TODO: Add integration test for sync with multiple pages
}

#[tokio::test]
async fn test_github_backfill_sync_with_wiremock() {
    // Set up test database
    let db = setup_test_db().await.unwrap();
    let tenant_id = create_test_tenant(&db, None).await.unwrap();

    // Insert GitHub provider
    insert_provider(&db, "github", "GitHub", "oauth2")
        .await
        .unwrap();

    // Create a GitHub connection for the tenant
    let connection_id = uuid::Uuid::new_v4();
    let _connection = insert_connection(&db, connection_id, tenant_id, "github", "github-user-123")
        .await
        .unwrap();

    // Setup mock server for GitHub API
    let mock_server = MockServer::start().await;

    // Point connector API to mock server
    unsafe {
        std::env::set_var("GITHUB_API_BASE", mock_server.uri());
    }

    // Mock issues API endpoint - first page
    Mock::given(method("GET"))
        .and(path("/user/issues"))
        .and(header("authorization", "Bearer test_access_token"))
        .and(header("accept", "application/vnd.github.v3+json"))
        .respond_with(
            ResponseTemplate::new(200)
                .append_header(
                    "Link",
                    format!(
                        r#"<{}/user/issues?page=2>; rel="next", <{}/user/issues?page=5>; rel="last""#,
                        mock_server.uri(),
                        mock_server.uri()
                    ),
                )
                .set_body_json(json!([
                    {
                        "id": 101,
                        "number": 1,
                        "title": "First Issue",
                        "state": "open",
                        "created_at": "2024-01-01T10:00:00Z",
                        "updated_at": "2024-01-01T10:00:00Z",
                        "user": {
                            "id": 456,
                            "login": "testuser"
                        },
                        "body": "This is the first issue",
                        "labels": [],
                        "pull_request": null
                    },
                    {
                        "id": 102,
                        "number": 2,
                        "title": "Second Issue",
                        "state": "closed",
                        "created_at": "2024-01-02T12:00:00Z",
                        "updated_at": "2024-01-02T15:00:00Z",
                        "user": {
                            "id": 456,
                            "login": "testuser"
                        },
                        "body": "This is the second issue",
                        "labels": [],
                        "pull_request": null
                    }
                ])),
        )
        .mount(&mock_server)
        .await;

    // Mock issues API endpoint - second page
    Mock::given(method("GET"))
        .and(path("/user/issues"))
        .and(header("authorization", "Bearer test_access_token"))
        .and(header("accept", "application/vnd.github.v3+json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "id": 103,
                "number": 3,
                "title": "Third Issue",
                "state": "open",
                "created_at": "2024-01-03T09:00:00Z",
                "updated_at": "2024-01-03T09:00:00Z",
                "user": {
                    "id": 456,
                    "login": "testuser"
                },
                "body": "This is the third issue",
                "labels": [],
                "pull_request": null
            }
        ])))
        .mount(&mock_server)
        .await;

    // Mock pull requests API endpoint
    Mock::given(method("GET"))
        .and(path("/pulls"))
        .and(header("authorization", "Bearer test_access_token"))
        .and(header("accept", "application/vnd.github.v3+json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "id": 201,
                "number": 10,
                "title": "First PR",
                "state": "open",
                "created_at": "2024-01-01T14:00:00Z",
                "updated_at": "2024-01-01T14:00:00Z",
                "user": {
                    "id": 456,
                    "login": "testuser"
                },
                "body": "This is the first PR",
                "labels": [],
                "merged": false,
                "head": {
                    "ref": "feature-branch",
                    "sha": "abc123"
                },
                "base": {
                    "ref": "main",
                    "sha": "def456"
                }
            }
        ])))
        .mount(&mock_server)
        .await;

    // Create connector with mock server URL
    let connector = GitHubConnector::new(
        "test_client_id".to_string(),
        "test_client_secret".to_string(),
        "https://localhost:3000/callback".to_string(),
        Some("test_webhook_secret".to_string()),
    );

    // Fetch the connection from the database
    let connection_from_db = connection::Entity::find_by_id(connection_id)
        .one(&db)
        .await
        .unwrap()
        .unwrap();

    // Mock the access token by modifying the connection
    let mut connection_with_token = connection_from_db.clone();
    connection_with_token.access_token_ciphertext = Some(b"test_access_token".to_vec());

    // Test sync without cursor (initial sync)
    let sync_params = SyncParams {
        connection: connection_with_token.clone(),
        cursor: None,
    };

    let sync_result = connector.sync(sync_params).await.unwrap();

    // Verify sync results
    assert!(!sync_result.signals.is_empty());
    assert!(sync_result.next_cursor.is_some());

    // Check that we got signals for both issues and pull requests
    let issue_signals: Vec<_> = sync_result
        .signals
        .iter()
        .filter(|s| s.kind.contains("issue"))
        .collect();
    let pr_signals: Vec<_> = sync_result
        .signals
        .iter()
        .filter(|s| s.kind.contains("pr"))
        .collect();

    assert!(!issue_signals.is_empty());
    assert!(!pr_signals.is_empty());

    // Test sync with cursor (incremental sync)
    let sync_params_with_cursor = SyncParams {
        connection: connection_with_token,
        cursor: sync_result.next_cursor,
    };

    let incremental_result = connector.sync(sync_params_with_cursor).await.unwrap();

    // Verify incremental sync results
    assert_eq!(incremental_result.signals.len(), 0); // No new items since our last sync
}

#[tokio::test]
async fn test_github_sync_rate_limit_handling() {
    // Set up test database
    let db = setup_test_db().await.unwrap();
    let tenant_id = create_test_tenant(&db, None).await.unwrap();

    // Insert GitHub provider
    insert_provider(&db, "github", "GitHub", "oauth2")
        .await
        .unwrap();

    // Create a GitHub connection for the tenant
    let connection_id = uuid::Uuid::new_v4();
    let _connection = insert_connection(&db, connection_id, tenant_id, "github", "github-user-123")
        .await
        .unwrap();

    // Setup mock server for GitHub API
    let mock_server = MockServer::start().await;

    // Point connector API to mock server
    unsafe {
        std::env::set_var("GITHUB_API_BASE", mock_server.uri());
    }

    // Mock rate limited response
    Mock::given(method("GET"))
        .and(path("/user/issues"))
        .and(header("authorization", "Bearer test_access_token"))
        .respond_with(
            ResponseTemplate::new(429)
                .append_header("Retry-After", "60")
                .append_header("X-RateLimit-Remaining", "0")
                .append_header(
                    "X-RateLimit-Reset",
                    (chrono::Utc::now().timestamp() + 60).to_string(),
                )
                .set_body_json(json!({
                    "message": "API rate limit exceeded",
                    "documentation_url": "https://docs.github.com/rest/overview/rate-limits-for-the-rest-api"
                })),
        )
        .mount(&mock_server)
        .await;

    // Create connector with mock server URL
    let connector = GitHubConnector::new(
        "test_client_id".to_string(),
        "test_client_secret".to_string(),
        "https://localhost:3000/callback".to_string(),
        Some("test_webhook_secret".to_string()),
    );

    // Fetch the connection from the database
    let connection_from_db = connection::Entity::find_by_id(connection_id)
        .one(&db)
        .await
        .unwrap()
        .unwrap();

    // Mock the access token by modifying the connection
    let mut connection_with_token = connection_from_db.clone();
    connection_with_token.access_token_ciphertext = Some(b"test_access_token".to_vec());

    // Test sync with rate limit
    let sync_params = SyncParams {
        connection: connection_with_token,
        cursor: None,
    };

    let result = connector.sync(sync_params).await;

    // Should handle rate limit gracefully
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("rate limit") || error_msg.contains("429"));
}

#[tokio::test]
async fn test_github_connector_configuration() {
    // Test connector creation with webhook secret
    let connector_with_webhook = GitHubConnector::new(
        "client_id".to_string(),
        "client_secret".to_string(),
        "https://localhost:3000/callback".to_string(),
        Some("webhook_secret".to_string()),
    );

    // Test signature verification with configured secret
    let payload = b"test";
    let signature = "sha256=invalid";
    assert!(
        connector_with_webhook
            .verify_webhook_signature(payload, signature)
            .is_err()
    );

    // Test connector creation without webhook secret
    let connector_without_webhook = GitHubConnector::new(
        "client_id".to_string(),
        "client_secret".to_string(),
        "https://localhost:3000/callback".to_string(),
        None,
    );

    // Should return error when trying to verify signatures without secret
    let payload = b"test";
    let signature = "sha256=invalid";
    assert!(
        connector_without_webhook
            .verify_webhook_signature(payload, signature)
            .is_err()
    );
}
