//! Example connector implementation
//!
//! A stub connector that demonstrates the Connector trait interface.
//! This can be used as a reference for implementing real connectors.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use url::Url;
use uuid::Uuid;

use crate::connectors::{
    AuthType, Connector, ProviderMetadata, Registry,
    trait_::{AuthorizeParams, ExchangeTokenParams, SyncParams, WebhookParams},
};
use crate::models::{connection::Model as Connection, signal::Model as Signal};

/// Example stub connector
pub struct ExampleConnector;

#[async_trait]
impl Connector for ExampleConnector {
    async fn authorize(
        &self,
        params: AuthorizeParams,
    ) -> Result<Url, Box<dyn std::error::Error + Send + Sync>> {
        // Stub implementation - return mock authorization URL with HTTPS
        let mut url = Url::parse("https://example.com/oauth/authorize")?;
        url.query_pairs_mut()
            .append_pair("client_id", "example_client_id")
            .append_pair(
                "redirect_uri",
                &params
                    .redirect_uri
                    .unwrap_or_else(|| "https://localhost:3000/callback".to_string()),
            )
            .append_pair(
                "state",
                &params.state.unwrap_or_else(|| "random_state".to_string()),
            )
            .append_pair("response_type", "code");

        Ok(url)
    }

    async fn exchange_token(
        &self,
        params: ExchangeTokenParams,
    ) -> Result<Connection, Box<dyn std::error::Error + Send + Sync>> {
        // Stub implementation - return mock connection
        let now = DateTime::from(Utc::now());
        Ok(Connection {
            id: Uuid::new_v4(),
            tenant_id: params.tenant_id,
            provider_slug: "example".to_string(),
            external_id: "user_123".to_string(),
            status: "active".to_string(),
            display_name: Some("Example Connection".to_string()),
            access_token_ciphertext: Some(b"mock_access_token".to_vec()),
            refresh_token_ciphertext: Some(b"mock_refresh_token".to_vec()),
            expires_at: Some(now + chrono::Duration::hours(1)),
            scopes: Some(serde_json::json!(["read", "write"])),
            metadata: Some(serde_json::json!({"provider": "example"})),
            created_at: now,
            updated_at: now,
        })
    }

    async fn refresh_token(
        &self,
        connection: Connection,
    ) -> Result<Connection, Box<dyn std::error::Error + Send + Sync>> {
        // Stub implementation - return connection with new tokens
        let now = DateTime::from(Utc::now());
        Ok(Connection {
            id: connection.id,
            tenant_id: connection.tenant_id,
            provider_slug: connection.provider_slug,
            external_id: connection.external_id,
            status: connection.status,
            display_name: connection.display_name,
            access_token_ciphertext: Some(b"refreshed_access_token".to_vec()),
            refresh_token_ciphertext: Some(b"new_refresh_token".to_vec()),
            expires_at: Some(now + chrono::Duration::hours(1)),
            scopes: connection.scopes,
            metadata: connection.metadata,
            created_at: connection.created_at,
            updated_at: now,
        })
    }

    async fn sync(
        &self,
        _params: SyncParams,
    ) -> Result<Vec<Signal>, Box<dyn std::error::Error + Send + Sync>> {
        // Stub implementation - return mock signals
        let now = DateTime::from(Utc::now());
        Ok(vec![Signal {
            id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            provider_slug: "example".to_string(),
            connection_id: Uuid::new_v4(),
            kind: "example_event".to_string(),
            occurred_at: now,
            received_at: now,
            payload: serde_json::json!({
                "type": "example",
                "message": "Mock signal from example connector"
            }),
            dedupe_key: Some("example_signal_1".to_string()),
            created_at: now,
            updated_at: now,
        }])
    }

    async fn handle_webhook(
        &self,
        params: WebhookParams,
    ) -> Result<Vec<Signal>, Box<dyn std::error::Error + Send + Sync>> {
        // Stub implementation - return mock signals from webhook
        let now = DateTime::from(Utc::now());
        let event_type = params
            .payload
            .get("event_type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        Ok(vec![Signal {
            id: Uuid::new_v4(),
            tenant_id: params.tenant_id,
            provider_slug: "example".to_string(),
            connection_id: Uuid::new_v4(),
            kind: format!("webhook:{}", event_type),
            occurred_at: now,
            received_at: now,
            payload: params.payload,
            dedupe_key: Some(format!("webhook_{}", now.timestamp())),
            created_at: now,
            updated_at: now,
        }])
    }
}

/// Initialize the example connector in the registry
pub fn register_example_connector(registry: &mut Registry) {
    let metadata = ProviderMetadata::new(
        "example".to_string(),
        AuthType::OAuth2,
        vec![
            "read:repositories".to_string(),
            "write:repositories".to_string(),
            "read:user".to_string(),
        ],
        true, // webhooks supported
    );

    let connector = Arc::new(ExampleConnector);
    registry.register(connector, metadata);
}
