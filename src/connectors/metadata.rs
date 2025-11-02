//! Provider metadata types
//!
//! Defines the metadata structure for providers and authentication types.

use serde::{Deserialize, Serialize};

/// Authentication type supported by a provider
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthType {
    /// OAuth 2.0 authorization code flow
    OAuth2,
    /// API key authentication
    ApiKey,
    /// Basic authentication (username/password)
    Basic,
    /// Bearer token authentication
    Bearer,
    /// Custom authentication method
    Custom(String),
}

/// Metadata about a provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderMetadata {
    /// Unique identifier for the provider
    pub name: String,
    /// Authentication method(s) supported
    pub auth_type: AuthType,
    /// OAuth scopes required (if applicable)
    pub scopes: Vec<String>,
    /// Whether this provider supports webhooks
    pub webhooks: bool,
}

impl ProviderMetadata {
    /// Create new provider metadata
    pub fn new(name: String, auth_type: AuthType, scopes: Vec<String>, webhooks: bool) -> Self {
        Self {
            name,
            auth_type,
            scopes,
            webhooks,
        }
    }

    /// Create minimal metadata for a provider
    pub fn minimal(name: String, auth_type: AuthType) -> Self {
        Self {
            name,
            auth_type,
            scopes: Vec::new(),
            webhooks: false,
        }
    }
}
