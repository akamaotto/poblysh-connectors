//! # Providers API Handlers
//!
//! This module contains handlers for the providers endpoints.

use crate::cursor::{decode_generic_cursor, encode_generic_cursor};
use crate::error::ApiError;
use crate::server::AppState;
use axum::{
    extract::{Query, State},
    response::Json,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

/// Query parameters for providers listing
#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct ListProvidersQuery {
    /// Maximum number of providers to return (default: 50, max: 100)
    pub limit: Option<i64>,
    /// Opaque cursor for pagination continuation
    pub cursor: Option<String>,
}

/// Provider information for public listing
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct ProviderInfo {
    /// Name of the provider (e.g., "github", "slack")
    pub name: String,
    /// Authentication type required by this provider
    pub auth_type: String,
    /// List of OAuth scopes this provider may request
    pub scopes: Vec<String>,
    /// Whether this provider supports webhook events
    pub webhooks: bool,
}

/// Response containing the list of available providers
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ProvidersResponse {
    /// List of available providers
    pub providers: Vec<ProviderInfo>,
    /// Opaque cursor for fetching the next page (null if this is the last page)
    pub next_cursor: Option<String>,
}

/// Public endpoint to list all available providers
#[utoipa::path(
    get,
    path = "/providers",
    params(ListProvidersQuery),
    responses(
        (status = 200, description = "List of available providers", body = ProvidersResponse, example = json!({
            "providers": [
                {
                    "name": "github",
                    "auth_type": "oauth2",
                    "scopes": ["repo", "user:email", "read:org"],
                    "webhooks": true
                },
                {
                    "name": "slack",
                    "auth_type": "oauth2",
                    "scopes": ["channels:read", "chat:write", "users:read"],
                    "webhooks": true
                }
            ],
            "next_cursor": null
        })),
        (status = 400, description = "Validation error", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    ),
    tag = "providers"
)]
pub async fn list_providers(
    State(_state): State<AppState>,
    Query(query): Query<ListProvidersQuery>,
) -> Result<Json<ProvidersResponse>, ApiError> {
    // Validate and parse limit
    let limit = query.limit.unwrap_or(50);
    if !(1..=100).contains(&limit) {
        return Err(ApiError::new(
            axum::http::StatusCode::BAD_REQUEST,
            "VALIDATION_FAILED",
            "limit must be between 1 and 100",
        ));
    }

    // Validate cursor if provided - parse Base64 JSON cursor with provider name
    let start_after_provider = if let Some(ref cursor_str) = query.cursor {
        let cursor = decode_generic_cursor(cursor_str).map_err(|_| {
            ApiError::new(
                axum::http::StatusCode::BAD_REQUEST,
                "VALIDATION_FAILED",
                "cursor must be valid base64-encoded JSON",
            )
        })?;

        let provider_name = cursor.keys["name"].as_str().ok_or_else(|| {
            ApiError::new(
                axum::http::StatusCode::BAD_REQUEST,
                "VALIDATION_FAILED",
                "cursor must contain provider name field",
            )
        })?;

        Some(provider_name.to_string())
    } else {
        None
    };

    // Static list for MVP - will be replaced with registry in future changes
    let mut providers = vec![
        ProviderInfo {
            name: "github".to_string(),
            auth_type: "oauth2".to_string(),
            scopes: vec![
                "repo".to_string(),
                "user:email".to_string(),
                "read:org".to_string(),
            ],
            webhooks: true,
        },
        ProviderInfo {
            name: "slack".to_string(),
            auth_type: "oauth2".to_string(),
            scopes: vec![
                "channels:read".to_string(),
                "chat:write".to_string(),
                "users:read".to_string(),
            ],
            webhooks: true,
        },
        ProviderInfo {
            name: "jira".to_string(),
            auth_type: "oauth2".to_string(),
            scopes: vec!["read:jira-work".to_string(), "read:jira-user".to_string()],
            webhooks: true,
        },
        ProviderInfo {
            name: "google-workspace".to_string(),
            auth_type: "oauth2".to_string(),
            scopes: vec![
                "https://www.googleapis.com/auth/calendar".to_string(),
                "https://www.googleapis.com/auth/drive.readonly".to_string(),
            ],
            webhooks: false,
        },
        ProviderInfo {
            name: "zoho".to_string(),
            auth_type: "oauth2".to_string(),
            scopes: vec![
                "ZohoCRM.modules.all".to_string(),
                "ZohoCRM.settings.all".to_string(),
            ],
            webhooks: true,
        },
        ProviderInfo {
            name: "zoho-cliq".to_string(),
            auth_type: "webhook".to_string(),
            scopes: vec![],
            webhooks: true,
        },
    ];

    // Stable ascending sort by name as per spec
    providers.sort_by(|a, b| a.name.cmp(&b.name));

    // Apply pagination using cursor-based approach
    let (paginated_providers, next_cursor) = if let Some(ref start_after) = start_after_provider {
        // Find providers after the specified provider name
        let mut iter = providers.into_iter().skip_while(|p| p.name <= *start_after);
        let paginated: Vec<_> = iter.by_ref().take(limit as usize).collect();

        // Check if there are more providers
        let has_more = iter.next().is_some();

        let next_cursor = if has_more && paginated.len() == limit as usize {
            if let Some(last_provider) = paginated.last() {
                let keys = serde_json::json!({
                    "name": last_provider.name
                });
                Some(encode_generic_cursor(keys))
            } else {
                None
            }
        } else {
            None
        };

        (paginated, next_cursor)
    } else {
        // First page - take from beginning
        let mut iter = providers.into_iter();
        let paginated: Vec<_> = iter.by_ref().take(limit as usize).collect();

        // Check if there are more providers
        let has_more = iter.next().is_some();

        let next_cursor = if has_more && paginated.len() == limit as usize {
            if let Some(last_provider) = paginated.last() {
                let keys = serde_json::json!({
                    "name": last_provider.name
                });
                Some(encode_generic_cursor(keys))
            } else {
                None
            }
        } else {
            None
        };

        (paginated, next_cursor)
    };

    Ok(Json(ProvidersResponse {
        providers: paginated_providers,
        next_cursor,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[tokio::test]
    async fn test_list_providers_returns_200_with_correct_shape() {
        // Create a mock state (not used by current implementation)
        let state = crate::server::create_test_app_state(
            crate::config::AppConfig::default(),
            sea_orm::Database::connect("sqlite::memory:").await.unwrap(),
        );

        // Call the handler
        let query = ListProvidersQuery {
            limit: None,
            cursor: None,
        };
        let result = list_providers(State(state), Query(query)).await;

        // Assert successful response
        assert!(result.is_ok());
        let response = result.unwrap();

        // Verify the structure and data
        assert_eq!(response.providers.len(), 6);

        // Check that providers are sorted by name
        let provider_names: Vec<String> =
            response.providers.iter().map(|p| p.name.clone()).collect();
        assert_eq!(
            provider_names,
            vec![
                "github",
                "google-workspace",
                "jira",
                "slack",
                "zoho",
                "zoho-cliq"
            ]
        );

        // Verify specific provider data
        let github = response
            .providers
            .iter()
            .find(|p| p.name == "github")
            .unwrap();
        assert_eq!(github.auth_type, "oauth2");
        assert!(github.webhooks);
        assert!(github.scopes.contains(&"repo".to_string()));

        let google = response
            .providers
            .iter()
            .find(|p| p.name == "google-workspace")
            .unwrap();
        assert_eq!(google.auth_type, "oauth2");
        assert!(!google.webhooks);
        assert!(
            google
                .scopes
                .iter()
                .any(|s| s.starts_with("https://www.googleapis.com/"))
        );

        // Test zoho-cliq provider
        let zoho_cliq = response
            .providers
            .iter()
            .find(|p| p.name == "zoho-cliq")
            .expect("zoho-cliq provider should be present");
        assert_eq!(zoho_cliq.auth_type, "webhook");
        assert!(zoho_cliq.webhooks);
        assert!(zoho_cliq.scopes.is_empty()); // No OAuth scopes for webhook-only provider
    }

    #[test]
    fn test_provider_info_serialization() {
        let provider = ProviderInfo {
            name: "test".to_string(),
            auth_type: "oauth2".to_string(),
            scopes: vec!["read".to_string(), "write".to_string()],
            webhooks: true,
        };

        let json = serde_json::to_string(&provider).unwrap();
        let parsed: ProviderInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.name, "test");
        assert_eq!(parsed.auth_type, "oauth2");
        assert_eq!(parsed.scopes, vec!["read".to_string(), "write".to_string()]);
        assert!(parsed.webhooks);
    }

    #[test]
    fn test_providers_response_serialization() {
        let providers = vec![
            ProviderInfo {
                name: "test1".to_string(),
                auth_type: "oauth2".to_string(),
                scopes: vec!["read".to_string()],
                webhooks: false,
            },
            ProviderInfo {
                name: "test2".to_string(),
                auth_type: "oauth2".to_string(),
                scopes: vec!["write".to_string()],
                webhooks: true,
            },
        ];

        let response = ProvidersResponse {
            providers,
            next_cursor: None,
        };
        let json = serde_json::to_string(&response).unwrap();
        let parsed: ProvidersResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.providers.len(), 2);
        assert_eq!(parsed.providers[0].name, "test1");
        assert_eq!(parsed.providers[1].name, "test2");
    }
}
