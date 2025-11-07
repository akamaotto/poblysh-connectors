//! Provider registry
//!
//! In-memory registry for storing and retrieving provider connectors and metadata.

use std::collections::HashMap;
use std::sync::{Arc, OnceLock, RwLock};

use crate::config::AppConfig;
use crate::connectors::{AuthType, Connector, ProviderMetadata};
use tracing::warn;

/// Error type for registry operations
#[derive(Debug, Clone, thiserror::Error)]
pub enum RegistryError {
    #[error("Provider '{name}' not found")]
    ProviderNotFound { name: String },
}

/// Global provider registry instance
static REGISTRY: OnceLock<Arc<RwLock<Registry>>> = OnceLock::new();

/// Provider registry that stores connectors and their metadata
#[derive(Clone)]
pub struct Registry {
    connectors: HashMap<String, Arc<dyn Connector>>,
    metadata: HashMap<String, ProviderMetadata>,
}

impl Registry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            connectors: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    /// Get the global registry instance
    pub fn global() -> &'static Arc<RwLock<Registry>> {
        REGISTRY.get_or_init(|| Arc::new(RwLock::new(Self::new())))
    }

    /// Initialize the global registry with providers
    pub fn initialize(config: &AppConfig) {
        let registry = Self::global();
        let mut reg = registry.write().unwrap();

        // Register example connector
        crate::connectors::example::register_example_connector(&mut reg);
        // Register Jira connector only if configured explicitly
        if let (Some(client_id), Some(client_secret)) = (
            config.jira_client_id.clone(),
            config.jira_client_secret.clone(),
        ) {
            let jira_connector = Arc::new(crate::connectors::JiraConnector::new(
                client_id,
                client_secret,
                config.jira_oauth_base.clone(),
                config.jira_api_base.clone(),
            ));
            crate::connectors::register_jira_connector(&mut reg, jira_connector);
        } else {
            warn!("Jira connector not registered: missing Jira client credentials");
        }
        // Register Google Drive connector
        crate::connectors::google_drive::register_google_drive_connector(&mut reg);

        // Register Google Calendar connector
        crate::connectors::google_calendar::register_google_calendar_connector(&mut reg);

        // Register Gmail connector
        let gmail_scopes = config
            .gmail_scopes
            .as_ref()
            .map(|scopes| scopes.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_else(|| {
                crate::connectors::gmail::DEFAULT_GMAIL_SCOPES
                    .iter()
                    .map(|s| s.to_string())
                    .collect()
            });

        let gmail_connector =
            Arc::new(crate::connectors::GmailConnector::new_with_oidc_and_scopes(
                config
                    .gmail_client_id
                    .clone()
                    .unwrap_or_else(|| "local-gmail-client-id".to_string()),
                config
                    .gmail_client_secret
                    .clone()
                    .unwrap_or_else(|| "local-gmail-client-secret".to_string()),
                config.pubsub_oidc_audience.clone(),
                config.pubsub_oidc_issuers.clone(),
                gmail_scopes,
            ));
        crate::connectors::gmail::register_gmail_connector(&mut reg, gmail_connector);

        // Register GitHub connector if configured
        // Note: This is a simplified registration - in production, this would use
        // the actual configuration from the app config
        let client_id = config.github_client_id.clone().or_else(|| {
            std::env::var("GITHUB_CLIENT_ID")
                .or_else(|_| std::env::var("POBLYSH_GITHUB_CLIENT_ID"))
                .ok()
        });
        let client_secret = config.github_client_secret.clone().or_else(|| {
            std::env::var("GITHUB_CLIENT_SECRET")
                .or_else(|_| std::env::var("POBLYSH_GITHUB_CLIENT_SECRET"))
                .ok()
        });

        if let (Some(client_id), Some(client_secret)) = (client_id, client_secret) {
            let webhook_secret = config.webhook_github_secret.clone().or_else(|| {
                std::env::var("GITHUB_WEBHOOK_SECRET")
                    .or_else(|_| std::env::var("POBLYSH_WEBHOOK_GITHUB_SECRET"))
                    .ok()
            });

            let github_connector = Arc::new(crate::connectors::GitHubConnector::new(
                client_id,
                client_secret,
                "https://localhost:3000/callback".to_string(),
                webhook_secret,
            ));
            crate::connectors::register_github_connector(&mut reg, github_connector);
        }

        // Register Zoho Mail connector if configured
        if std::env::var("POBLYSH_ZOHO_MAIL_CLIENT_ID").is_ok()
            && std::env::var("POBLYSH_ZOHO_MAIL_CLIENT_SECRET").is_ok()
            && std::env::var("POBLYSH_ZOHO_MAIL_DC").is_ok()
        {
            match crate::connectors::zoho_mail::ZohoMailConnector::new_from_env() {
                Ok(conn) => {
                    crate::connectors::zoho_mail::register_zoho_mail_connector(
                        &mut reg,
                        Arc::new(conn),
                    );
                }
                Err(err) => {
                    warn!(
                        "Zoho Mail connector not registered due to configuration error: {}",
                        err
                    );
                }
            }
        } else {
            warn!("Zoho Mail connector not registered: missing Zoho Mail client credentials or DC");
        }

        // Register Zoho Cliq connector (webhook-only, no config required for MVP)
        let zoho_cliq_connector = Arc::new(crate::connectors::ZohoCliqConnector::new());
        crate::connectors::register_zoho_cliq_connector(&mut reg, zoho_cliq_connector);
    }

    /// Register a new provider with its connector and metadata
    pub fn register(&mut self, connector: Arc<dyn Connector>, metadata: ProviderMetadata) {
        let name = metadata.name.clone();
        self.connectors.insert(name.clone(), connector);
        self.metadata.insert(name, metadata);
    }

    /// Get only OAuth2 providers for OAuth flows
    pub fn get_oauth_providers(&self) -> Vec<ProviderMetadata> {
        self.metadata
            .values()
            .filter(|metadata| matches!(metadata.auth_type, AuthType::OAuth2))
            .cloned()
            .collect()
    }

    /// Check if a provider supports OAuth2 flows
    pub fn is_oauth_provider(&self, name: &str) -> bool {
        if let Some(metadata) = self.metadata.get(name) {
            matches!(metadata.auth_type, AuthType::OAuth2)
        } else {
            false
        }
    }

    /// Get a connector by provider name
    pub fn get(&self, name: &str) -> Result<Arc<dyn Connector>, RegistryError> {
        self.connectors
            .get(name)
            .cloned()
            .ok_or_else(|| RegistryError::ProviderNotFound {
                name: name.to_string(),
            })
    }

    /// Get metadata for all providers, sorted by name for stable ordering
    pub fn list_metadata(&self) -> Vec<ProviderMetadata> {
        let mut metadata: Vec<_> = self.metadata.values().cloned().collect();
        metadata.sort_by(|a, b| a.name.cmp(&b.name));
        metadata
    }

    /// Get metadata for a specific provider
    pub fn get_metadata(&self, name: &str) -> Result<&ProviderMetadata, RegistryError> {
        self.metadata
            .get(name)
            .ok_or_else(|| RegistryError::ProviderNotFound {
                name: name.to_string(),
            })
    }

    /// Get metadata for a specific provider (convenient method for RwLock)
    pub fn get_provider_metadata(name: &str) -> Result<ProviderMetadata, RegistryError> {
        let registry = Self::global();
        let reg = registry.read().unwrap();
        reg.get_metadata(name).cloned()
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connectors::trait_::{
        AuthorizeParams, Connector, ExchangeTokenParams, SyncParams, SyncResult, WebhookParams,
    };
    use crate::models::{connection::Model as Connection, signal::Model as Signal};
    use async_trait::async_trait;
    use chrono::{DateTime, Utc};
    use url::Url;
    use uuid::Uuid;

    struct TestConnector;

    #[async_trait]
    impl Connector for TestConnector {
        async fn authorize(
            &self,
            _params: AuthorizeParams,
        ) -> Result<Url, Box<dyn std::error::Error + Send + Sync>> {
            Ok(Url::parse("https://example.com/oauth/authorize")?)
        }

        async fn exchange_token(
            &self,
            _params: ExchangeTokenParams,
        ) -> Result<Connection, Box<dyn std::error::Error + Send + Sync>> {
            // Create a mock connection for testing
            Ok(Connection {
                id: Uuid::new_v4(),
                tenant_id: Uuid::new_v4(),
                provider_slug: "test".to_string(),
                external_id: "123".to_string(),
                status: "active".to_string(),
                display_name: None,
                access_token_ciphertext: None,
                refresh_token_ciphertext: None,
                expires_at: None,
                scopes: None,
                metadata: None,
                created_at: DateTime::from(Utc::now()),
                updated_at: DateTime::from(Utc::now()),
            })
        }

        async fn refresh_token(
            &self,
            _connection: Connection,
        ) -> Result<Connection, Box<dyn std::error::Error + Send + Sync>> {
            Err("Not implemented".into())
        }

        async fn sync(
            &self,
            _params: SyncParams,
        ) -> Result<SyncResult, Box<dyn std::error::Error + Send + Sync>> {
            Ok(SyncResult {
                signals: vec![],
                next_cursor: None,
                has_more: false,
            })
        }

        async fn handle_webhook(
            &self,
            _params: WebhookParams,
        ) -> Result<Vec<Signal>, Box<dyn std::error::Error + Send + Sync>> {
            Ok(vec![])
        }
    }

    #[tokio::test]
    async fn test_registry_unknown_provider() {
        let registry = Registry::new();

        // Test get with unknown provider
        let result = registry.get("unknown");
        assert!(result.is_err());
        if let Err(RegistryError::ProviderNotFound { name }) = result {
            assert_eq!(name, "unknown");
        } else {
            panic!("Expected ProviderNotFound error");
        }

        // Test get_metadata with unknown provider
        let result = registry.get_metadata("unknown");
        assert!(result.is_err());
        if let Err(RegistryError::ProviderNotFound { name }) = result {
            assert_eq!(name, "unknown");
        } else {
            panic!("Expected ProviderNotFound error");
        }
    }

    #[tokio::test]
    async fn test_registry_known_provider() {
        let mut registry = Registry::new();

        // Register a known provider
        registry.register(
            Arc::new(TestConnector),
            ProviderMetadata::new(
                "test-provider".to_string(),
                crate::connectors::AuthType::OAuth2,
                vec![],
                true,
            ),
        );

        // Test get with known provider should succeed
        let result = registry.get("test-provider");
        assert!(result.is_ok());

        // The returned connector should be usable (we can't call methods directly in tests without async setup)
        // but we can verify it returns a non-null reference
        let _connector = result.unwrap();

        // Test get_metadata with known provider should succeed
        let metadata_result = registry.get_metadata("test-provider");
        assert!(metadata_result.is_ok());

        let metadata = metadata_result.unwrap();
        assert_eq!(metadata.name, "test-provider");
        assert_eq!(metadata.auth_type, crate::connectors::AuthType::OAuth2);
        assert!(metadata.webhooks);
    }

    #[tokio::test]
    async fn test_registry_list_ordering() {
        let mut registry = Registry::new();

        // Register providers in non-alphabetical order
        registry.register(
            Arc::new(TestConnector),
            ProviderMetadata::new(
                "zebra".to_string(),
                crate::connectors::AuthType::OAuth2,
                vec![],
                true,
            ),
        );
        registry.register(
            Arc::new(TestConnector),
            ProviderMetadata::new(
                "apple".to_string(),
                crate::connectors::AuthType::OAuth2,
                vec![],
                false,
            ),
        );
        registry.register(
            Arc::new(TestConnector),
            ProviderMetadata::new(
                "banana".to_string(),
                crate::connectors::AuthType::OAuth2,
                vec![],
                true,
            ),
        );

        let metadata = registry.list_metadata();
        assert_eq!(metadata.len(), 3);
        assert_eq!(metadata[0].name, "apple");
        assert_eq!(metadata[1].name, "banana");
        assert_eq!(metadata[2].name, "zebra");
    }

    #[tokio::test]
    async fn test_registry_metadata_completeness() {
        let mut registry = Registry::new();

        let provider_metadata = ProviderMetadata::new(
            "test-provider".to_string(),
            crate::connectors::AuthType::OAuth2,
            vec!["read".to_string(), "write".to_string()],
            true,
        );

        registry.register(Arc::new(TestConnector), provider_metadata.clone());

        // Test get_metadata returns complete data
        let retrieved = registry.get_metadata("test-provider").unwrap();
        assert_eq!(retrieved.name, provider_metadata.name);
        assert_eq!(retrieved.auth_type, provider_metadata.auth_type);
        assert_eq!(retrieved.scopes, provider_metadata.scopes);
        assert_eq!(retrieved.webhooks, provider_metadata.webhooks);
    }

    #[tokio::test]
    async fn test_registry_initialization() {
        // Reset the global registry state for this test
        // Note: This requires access to the global registry initialization

        // Call initialize to seed the registry
        let config = crate::config::AppConfig::default();
        Registry::initialize(&config);

        // Verify that the example provider is now present
        let metadata_result = Registry::get_provider_metadata("example");
        assert!(
            metadata_result.is_ok(),
            "Example provider should be registered after initialization"
        );

        let metadata = metadata_result.unwrap();
        assert_eq!(metadata.name, "example");
        assert_eq!(metadata.auth_type, crate::connectors::AuthType::OAuth2);
        assert!(metadata.webhooks);
        assert!(
            !metadata.scopes.is_empty(),
            "Example provider should have OAuth scopes"
        );

        // Verify that we can get the connector (not null)
        let registry = Registry::global();
        let reg = registry.read().unwrap();
        let connector_result = reg.get("example");
        assert!(
            connector_result.is_ok(),
            "Should be able to get example connector after initialization"
        );
    }
}
