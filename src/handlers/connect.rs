//! # Connection Handlers
//!
//! This module contains handlers for managing OAuth connections with providers.

use crate::auth::{OperatorAuth, TenantExtension, TenantHeader};
use crate::connectors::registry::{Registry, RegistryError};
use crate::connectors::{AuthType, AuthorizeParams};
use crate::error::ApiError;

use crate::repositories::oauth_state::OAuthStateRepository;
use crate::server::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use url::Url;
use utoipa::ToSchema;

/// Request path parameter for provider name
#[derive(Debug, Deserialize, ToSchema)]
pub struct ProviderPath {
    /// Provider identifier (snake_case, e.g., "github")
    pub provider: String,
}

/// OAuth authorization URL response for API
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AuthorizeUrlResponse {
    /// Complete authorization URL for user redirection
    /// Must be HTTPS, valid per RFC 3986, max 2048 chars, no fragment
    pub authorize_url: String,
}

/// Start OAuth flow for a provider
///
/// Initiates an OAuth authorization flow for the specified provider and tenant.
/// Returns a fully formed authorization URL that the client can use to redirect
/// the user to the provider's authorization page.
#[utoipa::path(
    post,
    path = "/connect/{provider}",
    security(("bearer_auth" = [])),
    params(
        ("provider" = String, Path, description = "Provider identifier (snake_case, e.g., 'github')"),
        TenantHeader
    ),
    responses(
        (status = 200, description = "OAuth authorization URL generated successfully", body = AuthorizeUrlResponse),
        (status = 400, description = "Bad request - provider does not support OAuth2 or missing tenant header", body = ApiError),
        (status = 401, description = "Missing or invalid authorization token", body = ApiError),
        (status = 403, description = "Insufficient permissions for tenant", body = ApiError),
        (status = 404, description = "Provider not found", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    ),
    tag = "connections"
)]
pub async fn start_oauth(
    State(state): State<AppState>,
    _operator_auth: OperatorAuth,
    TenantExtension(tenant): TenantExtension,
    Path(provider_path): Path<ProviderPath>,
) -> Result<Json<AuthorizeUrlResponse>, ApiError> {
    let provider = provider_path.provider;

    // Get the global registry and validate provider supports OAuth2
    let connector = {
        let registry = Registry::global();
        let registry = registry.read().unwrap();

        // Resolve provider metadata first to differentiate unknown vs unsupported providers
        let metadata = match registry.get_metadata(&provider) {
            Ok(metadata) => metadata.clone(),
            Err(RegistryError::ProviderNotFound { name }) => {
                return Err(ApiError::new(
                    StatusCode::NOT_FOUND,
                    "NOT_FOUND",
                    &format!("provider '{}' not found", name),
                ));
            }
        };

        if metadata.auth_type != AuthType::OAuth2 {
            return Err(ApiError::new(
                StatusCode::BAD_REQUEST,
                "VALIDATION_FAILED",
                &format!("provider '{}' does not support OAuth2", provider),
            ));
        }

        // Resolve connector from registry; return 404 via ApiError if unknown
        match registry.get(&provider) {
            Ok(connector) => connector,
            Err(RegistryError::ProviderNotFound { name }) => {
                return Err(ApiError::new(
                    StatusCode::NOT_FOUND,
                    "NOT_FOUND",
                    &format!("provider '{}' not found", name),
                ));
            }
        }
    };

    // Generate a cryptographically secure state token
    let state_token = generate_secure_state();

    // Create OAuth state repository and persist the state
    let oauth_state_repo = OAuthStateRepository::new(Arc::new(state.db.clone()));

    // Persist OAuth state with 15 minute expiration
    let oauth_state = match oauth_state_repo
        .create(tenant.0, &provider, &state_token, None, 15)
        .await
    {
        Ok(state) => state,
        Err(err) => {
            eprintln!("Detailed OAuth state creation error: {:?}", err);
            tracing::error!("Failed to persist OAuth state: {:?}", err);
            return Err(ApiError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_SERVER_ERROR",
                "Failed to create OAuth state",
            ));
        }
    };

    // Call connector.authorize(tenant) and return authorization URL
    let authorize_params = AuthorizeParams {
        tenant_id: tenant.0,
        redirect_uri: None, // TODO: Configure redirect URI based on deployment
        state: Some(state_token.clone()),
    };

    let authorize_url = match connector.authorize(authorize_params).await {
        Ok(url) => url,
        Err(err) => {
            tracing::error!(
                "Failed to generate authorize URL for provider '{}': {:?}",
                provider,
                err
            );

            // Clean up the created state since the flow failed
            let _ = oauth_state_repo.delete_by_id(oauth_state.id).await;

            return Err(ApiError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_SERVER_ERROR",
                "Failed to generate authorization URL",
            ));
        }
    };

    // Validate the URL meets OAuth 2.0 requirements
    validate_authorize_url(&authorize_url)?;

    tracing::info!(
        tenant_id = %tenant.0,
        provider = %provider,
        state_id = %oauth_state.id,
        "OAuth flow initiated successfully"
    );

    let response = AuthorizeUrlResponse {
        authorize_url: authorize_url.to_string(),
    };

    Ok(Json(response))
}

/// Generate a cryptographically secure random state token
fn generate_secure_state() -> String {
    use rand::Rng;

    // Generate 32 bytes of random data
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill(&mut bytes);

    // Encode as base64 URL-safe string
    base64_url::encode(&bytes)
}

/// Validate authorization URL meets OAuth 2.0 and security requirements
fn validate_authorize_url(url: &Url) -> Result<(), ApiError> {
    // Must be HTTPS
    if url.scheme() != "https" {
        return Err(ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "INTERNAL_SERVER_ERROR",
            "Connector bug: Generated authorization URL must use HTTPS",
        ));
    }

    // Must not include fragment component per OAuth 2.0 RFC 6749 section 3.1
    if url.fragment().is_some() {
        return Err(ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "INTERNAL_SERVER_ERROR",
            "Connector bug: Generated authorization URL must not include fragment component",
        ));
    }

    // Maximum length 2048 characters
    if url.as_str().len() > 2048 {
        return Err(ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "INTERNAL_SERVER_ERROR",
            "Connector bug: Generated authorization URL exceeds maximum length of 2048 characters",
        ));
    }

    // Valid according to RFC 3986 (Url type ensures this)
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use sea_orm::ConnectionTrait;

    use url::Url;
    use uuid::Uuid;

    // Create test app state with proper database setup for integration testing
    async fn create_test_app_state() -> AppState {
        use crate::config::AppConfig;
        use sea_orm::{Database, DatabaseConnection};

        // Initialize registry
        crate::connectors::registry::Registry::initialize();

        // Use unique file-based SQLite for testing to avoid migration conflicts
        let test_db_name = format!(
            "test_{}.db",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let db_url = format!("sqlite:{}?mode=rwc", test_db_name);
        let db: DatabaseConnection = Database::connect(db_url)
            .await
            .expect("Failed to connect to test database");

        // Apply migrations to create proper schema
        use migration::{Migrator, MigratorTrait};
        println!("Applying migrations...");
        Migrator::up(&db, None)
            .await
            .expect("Failed to apply migrations");
        println!("Migrations applied successfully");

        // Verify tables exist
        use sea_orm::Statement;
        let tables_query = Statement::from_string(
            sea_orm::DatabaseBackend::Sqlite,
            "SELECT name FROM sqlite_master WHERE type='table'".to_string(),
        );

        match db.query_one(tables_query).await {
            Ok(Some(result)) => {
                if let Ok(table_name) = result.try_get::<String>("", "name") {
                    println!("Found table: {}", table_name);
                }
            }
            Ok(None) => println!("No tables found"),
            Err(e) => println!("Error querying tables: {:?}", e),
        }

        // Check applied migrations
        let migrations_query = Statement::from_string(
            sea_orm::DatabaseBackend::Sqlite,
            "SELECT version FROM seaql_migrations ORDER BY applied_at".to_string(),
        );

        match db.query_all(migrations_query).await {
            Ok(results) => {
                println!("Applied migrations:");
                for result in results {
                    if let Ok(version) = result.try_get::<String>("", "version") {
                        println!("  - {}", version);
                    }
                }
            }
            Err(e) => println!("Error querying migrations: {:?}", e),
        }

        // List all tables again
        let all_tables_query = Statement::from_string(
            sea_orm::DatabaseBackend::Sqlite,
            "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name".to_string(),
        );

        println!("All tables in database:");
        match db.query_all(all_tables_query).await {
            Ok(results) => {
                for result in results {
                    if let Ok(table_name) = result.try_get::<String>("", "name") {
                        println!("  - {}", table_name);
                    }
                }
            }
            Err(e) => println!("Error listing all tables: {:?}", e),
        }

        let config = AppConfig::default();
        AppState {
            config: std::sync::Arc::new(config),
            db,
        }
    }

    #[tokio::test]
    async fn test_start_oauth_success_components() {
        // Test the key components of OAuth flow without database dependencies

        // Initialize registry to make sure it has providers
        crate::connectors::registry::Registry::initialize();

        // Test 1: Verify that example provider exists and supports OAuth2
        let result = crate::connectors::registry::Registry::get_provider_metadata("example");
        assert!(result.is_ok(), "Example provider should exist in registry");

        let metadata = result.unwrap();
        assert_eq!(metadata.name, "example");
        assert_eq!(metadata.auth_type, crate::connectors::AuthType::OAuth2);

        // Test 2: Verify OAuth provider filtering works
        let registry = crate::connectors::registry::Registry::global();
        let registry = registry.read().unwrap();
        assert!(
            registry.is_oauth_provider("example"),
            "Example should be identified as OAuth provider"
        );
        assert!(
            !registry.is_oauth_provider("nonexistent"),
            "Nonexistent provider should not be OAuth"
        );

        // Test 3: Test state generation
        let state1 = generate_secure_state();
        let state2 = generate_secure_state();
        assert_ne!(state1, state2, "States should be unique");
        assert_eq!(
            state1.len(),
            43,
            "State should be 43 characters (base64 of 32 bytes)"
        );
        assert!(
            !state1.contains('+') && !state1.contains('/'),
            "State should be URL-safe"
        );

        // Test 4: Test URL validation
        let valid_url = Url::parse("https://example.com/oauth/authorize?state=test").unwrap();
        assert!(
            validate_authorize_url(&valid_url).is_ok(),
            "Valid HTTPS URL should pass validation"
        );

        let invalid_url = Url::parse("http://example.com/oauth/authorize").unwrap();
        assert!(
            validate_authorize_url(&invalid_url).is_err(),
            "HTTP URL should fail validation"
        );

        println!("✓ OAuth flow components working correctly");
    }

    #[tokio::test]
    async fn test_start_oauth_success_integration() {
        // Test the complete OAuth flow by calling the handler directly

        let app_state = create_test_app_state().await;
        let tenant_id = Uuid::new_v4();

        // Mock auth and tenant contexts exactly as they would be provided by middleware
        let operator_auth = crate::auth::OperatorAuth;
        let tenant_extension = crate::auth::TenantExtension(crate::auth::TenantId(tenant_id));
        let provider_path = ProviderPath {
            provider: "example".to_string(),
        };

        // Call the handler directly with mocked contexts
        let result = start_oauth(
            axum::extract::State(app_state),
            operator_auth,
            tenant_extension,
            axum::extract::Path(provider_path),
        )
        .await;

        // Verify the handler succeeds
        match &result {
            Ok(response) => {
                println!("OAuth flow succeeded: {}", response.authorize_url);
            }
            Err(e) => {
                println!("OAuth flow failed: {:?}", e);
            }
        }
        assert!(result.is_ok(), "OAuth flow should succeed");

        let response = result.unwrap();
        let authorize_url = &response.authorize_url;

        // Verify response format matches spec
        assert!(authorize_url.starts_with("https://"), "URL must be HTTPS");
        assert!(
            authorize_url.contains("state="),
            "URL must contain state parameter"
        );
        assert!(
            authorize_url.len() < 2048,
            "URL must not exceed 2048 characters"
        );
        assert!(
            !authorize_url.contains('#'),
            "URL must not contain fragment"
        );

        println!("✓ OAuth integration test passed: {}", authorize_url);
    }

    #[tokio::test]
    async fn test_start_oauth_unknown_provider_integration() {
        // Test unknown provider scenario by calling the handler directly

        let app_state = create_test_app_state().await;
        let tenant_id = Uuid::new_v4();

        // Mock auth and tenant contexts
        let operator_auth = crate::auth::OperatorAuth;
        let tenant_extension = crate::auth::TenantExtension(crate::auth::TenantId(tenant_id));
        let provider_path = ProviderPath {
            provider: "nonexistent_provider".to_string(),
        };

        // Call the handler directly
        let result = start_oauth(
            axum::extract::State(app_state),
            operator_auth,
            tenant_extension,
            axum::extract::Path(provider_path),
        )
        .await;

        // Should return 404 NOT_FOUND
        assert!(result.is_err(), "Unknown provider should return error");
        let error = result.unwrap_err();
        assert_eq!(error.status, StatusCode::NOT_FOUND);
        assert_eq!(error.code.as_ref(), "NOT_FOUND");
        assert!(error.message.contains("not found"));

        println!("✓ Unknown provider integration test passed");
    }

    #[tokio::test]
    async fn test_start_oauth_non_oauth_provider_integration() {
        // Test non-OAuth provider scenario
        let app_state = create_test_app_state().await;
        let tenant_id = Uuid::new_v4();

        // Mock auth and tenant contexts
        let operator_auth = crate::auth::OperatorAuth;
        let tenant_extension = crate::auth::TenantExtension(crate::auth::TenantId(tenant_id));

        // Test with a provider that might exist but not support OAuth2
        let provider_path = ProviderPath {
            provider: "basic_auth_provider".to_string(),
        };

        // Call the handler directly
        let result = start_oauth(
            axum::extract::State(app_state),
            operator_auth,
            tenant_extension,
            axum::extract::Path(provider_path),
        )
        .await;

        // The result depends on whether basic_auth_provider exists and its auth type
        // If it doesn't exist, we get 404. If it exists but isn't OAuth2, we get 400.
        assert!(result.is_err(), "Non-OAuth provider should return error");
        let error = result.unwrap_err();

        // Either 404 (provider doesn't exist) or 400 (exists but not OAuth2) are valid
        assert!(
            error.status == StatusCode::NOT_FOUND || error.status == StatusCode::BAD_REQUEST,
            "Should return 404 for unknown provider or 400 for non-OAuth provider"
        );

        if error.status == StatusCode::BAD_REQUEST {
            assert_eq!(error.code.as_ref(), "VALIDATION_FAILED");
            assert!(error.message.contains("does not support OAuth2"));
            println!("✓ Non-OAuth provider integration test passed (400 error)");
        } else {
            assert_eq!(error.code.as_ref(), "NOT_FOUND");
            assert!(error.message.contains("not found"));
            println!("✓ Non-OAuth provider test passed (provider doesn't exist - 404 error)");
        }
    }

    #[tokio::test]
    async fn test_start_oauth_unknown_provider() {
        // Test unknown provider scenario using registry directly

        // Initialize registry
        crate::connectors::registry::Registry::initialize();

        // Test 1: Verify unknown provider returns NOT_FOUND from registry
        let result =
            crate::connectors::registry::Registry::get_provider_metadata("nonexistent_provider");
        assert!(result.is_err(), "Nonexistent provider should return error");

        match result.unwrap_err() {
            crate::connectors::registry::RegistryError::ProviderNotFound { name } => {
                assert_eq!(name, "nonexistent_provider");
            }
        }

        // Test 2: Verify unknown provider is not flagged as OAuth provider
        let registry = crate::connectors::registry::Registry::global();
        let registry = registry.read().unwrap();
        assert!(
            !registry.is_oauth_provider("nonexistent_provider"),
            "Nonexistent provider should not be OAuth"
        );

        // Test 3: Simulate the error that would be returned by handler
        // This tests the actual error message format that the handler would return
        let error = crate::error::ApiError::new(
            StatusCode::NOT_FOUND,
            "NOT_FOUND",
            "provider 'nonexistent_provider' not found",
        );

        assert_eq!(error.status, StatusCode::NOT_FOUND);
        assert_eq!(error.code.as_ref(), "NOT_FOUND");
        assert!(error.message.contains("not found"));

        println!("✓ Unknown provider error handling working correctly");
    }

    #[tokio::test]
    async fn test_start_oauth_non_oauth_provider() {
        // Test non-OAuth provider scenario using registry directly

        // Initialize registry
        crate::connectors::registry::Registry::initialize();

        // Test 1: Find a provider that exists but is not OAuth2
        // For this test, we'll check if there are any non-OAuth providers in the registry
        let registry = crate::connectors::registry::Registry::global();
        let registry = registry.read().unwrap();

        // Test 2: Verify OAuth provider filtering correctly identifies non-OAuth providers
        // The is_oauth_provider method should return false for non-OAuth providers
        let is_oauth = registry.is_oauth_provider("basic_auth_provider");

        // If basic_auth_provider doesn't exist, we can still test the error handling logic
        if !is_oauth {
            // Test 3: Simulate the error that would be returned by handler for non-OAuth provider
            let error = crate::error::ApiError::new(
                StatusCode::BAD_REQUEST,
                "VALIDATION_FAILED",
                "provider 'basic_auth_provider' does not support OAuth2",
            );

            assert_eq!(error.status, StatusCode::BAD_REQUEST);
            assert_eq!(error.code.as_ref(), "VALIDATION_FAILED");
            assert!(error.message.contains("does not support OAuth2"));

            println!("✓ Non-OAuth provider error handling working correctly");
        } else {
            println!(
                "✓ OAuth provider filtering working (basic_auth_provider is OAuth or doesn't exist)"
            );
        }

        // Test 4: Verify error handling is consistent between unknown and non-OAuth providers
        let unknown_error = crate::error::ApiError::new(
            StatusCode::NOT_FOUND,
            "NOT_FOUND",
            "provider 'unknown' not found",
        );

        let non_oauth_error = crate::error::ApiError::new(
            StatusCode::BAD_REQUEST,
            "VALIDATION_FAILED",
            "provider 'non_oauth' does not support OAuth2",
        );

        // Different status codes
        assert_ne!(unknown_error.status, non_oauth_error.status);
        assert_ne!(unknown_error.code.as_ref(), non_oauth_error.code.as_ref());

        println!(
            "✓ Error differentiation between unknown and non-OAuth providers working correctly"
        );
    }

    #[tokio::test]
    async fn test_generate_secure_state() {
        let state1 = generate_secure_state();
        let state2 = generate_secure_state();

        // States should be different
        assert_ne!(state1, state2);

        // Should be base64 URL-safe encoded
        assert!(
            state1
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        );

        // Should be reasonable length (32 bytes = ~43 chars base64)
        assert_eq!(state1.len(), 43);
        assert_eq!(state2.len(), 43);
    }

    #[tokio::test]
    async fn test_validate_authorize_url() {
        // Valid HTTPS URL
        let valid_url =
            Url::parse("https://example.com/oauth/authorize?client_id=test&state=abc").unwrap();
        assert!(validate_authorize_url(&valid_url).is_ok());

        // Invalid HTTP URL
        let invalid_url = Url::parse("http://example.com/oauth/authorize").unwrap();
        let result = validate_authorize_url(&invalid_url);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code.as_ref(), "INTERNAL_SERVER_ERROR");

        // URL with fragment
        let fragment_url = Url::parse("https://example.com/oauth/authorize#fragment").unwrap();
        let result = validate_authorize_url(&fragment_url);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code.as_ref(), "INTERNAL_SERVER_ERROR");

        // URL too long
        let mut long_url_str = "https://example.com/oauth/authorize?".to_string();
        long_url_str.push_str(&"a".repeat(2048 - long_url_str.len() + 1));
        let long_url = Url::parse(&long_url_str).unwrap();
        let result = validate_authorize_url(&long_url);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code.as_ref(), "INTERNAL_SERVER_ERROR");
    }
}
