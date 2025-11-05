//! Jira connector implementation
//!
//! Minimal Jira connector satisfying the Connector trait with realistic
//! OAuth authorize URL, webhook filtering, and incremental sync stubs.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use url::Url;
use uuid::Uuid;

use crate::connectors::{
    AuthType, Connector, Cursor, ProviderMetadata, Registry,
    trait_::{AuthorizeParams, ExchangeTokenParams, SyncParams, SyncResult, WebhookParams},
};
use crate::models::{connection::Model as Connection, signal::Model as Signal};

/// Jira connector
pub struct JiraConnector;

const JIRA_OAUTH_AUTHORIZE: &str = "https://auth.atlassian.com/authorize";
const JIRA_AUDIENCE: &str = "api.atlassian.com";

#[async_trait]
impl Connector for JiraConnector {
    async fn authorize(
        &self,
        params: AuthorizeParams,
    ) -> Result<Url, Box<dyn std::error::Error + Send + Sync>> {
        // Build Atlassian authorize URL with standard params.
        // Client id and scopes are placeholders in MVP; will be sourced from config later.
        let mut url = Url::parse(JIRA_OAUTH_AUTHORIZE)?;
        url.query_pairs_mut()
            .append_pair("audience", JIRA_AUDIENCE)
            .append_pair("client_id", "jira_client_id_placeholder")
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
            .append_pair("response_type", "code")
            .append_pair("prompt", "consent")
            .append_pair("scope", "read:jira-work read:jira-user offline_access");

        Ok(url)
    }

    async fn exchange_token(
        &self,
        params: ExchangeTokenParams,
    ) -> Result<Connection, Box<dyn std::error::Error + Send + Sync>> {
        // Stub implementation - return mock connection with Jira provider slug
        let now = DateTime::from(Utc::now());
        Ok(Connection {
            id: Uuid::new_v4(),
            tenant_id: params.tenant_id,
            provider_slug: "jira".to_string(),
            external_id: "jira-user-123".to_string(),
            status: "active".to_string(),
            display_name: Some("Jira Connection".to_string()),
            access_token_ciphertext: Some(b"mock_jira_access_token".to_vec()),
            refresh_token_ciphertext: Some(b"mock_jira_refresh_token".to_vec()),
            expires_at: Some(now + chrono::Duration::hours(1)),
            scopes: Some(serde_json::json!(["read:jira-work", "read:jira-user"])),
            metadata: Some(serde_json::json!({
                "provider": "jira",
                "hint": "stub",
            })),
            created_at: now,
            updated_at: now,
        })
    }

    async fn refresh_token(
        &self,
        connection: Connection,
    ) -> Result<Connection, Box<dyn std::error::Error + Send + Sync>> {
        // Stub implementation - return connection with updated tokens
        let now = DateTime::from(Utc::now());
        Ok(Connection {
            id: connection.id,
            tenant_id: connection.tenant_id,
            provider_slug: connection.provider_slug,
            external_id: connection.external_id,
            status: connection.status,
            display_name: connection.display_name,
            access_token_ciphertext: Some(b"refreshed_jira_access_token".to_vec()),
            refresh_token_ciphertext: Some(b"new_jira_refresh_token".to_vec()),
            expires_at: Some(now + chrono::Duration::hours(1)),
            scopes: connection.scopes,
            metadata: connection.metadata,
            created_at: connection.created_at,
            updated_at: now,
        })
    }

    async fn sync(
        &self,
        params: SyncParams,
    ) -> Result<SyncResult, Box<dyn std::error::Error + Send + Sync>> {
        // Stub incremental sync based on cursor timestamp.
        // If a cursor is provided, include it in payload for downstream processing.
        let now = DateTime::from(Utc::now());
        let since = params
            .cursor
            .as_ref()
            .map(|c| c.as_json().clone())
            .unwrap_or_default();

        // Produce a single mock issue_updated signal for demonstration
        // For demo purposes, show pagination by setting has_more=true and returning a cursor
        let next_timestamp = now.timestamp() + 1;
        Ok(SyncResult {
            signals: vec![Signal {
                id: Uuid::new_v4(),
                tenant_id: params.connection.tenant_id,
                provider_slug: "jira".to_string(),
                connection_id: params.connection.id,
                kind: "issue_updated".to_string(),
                occurred_at: now,
                received_at: now,
                payload: serde_json::json!({
                    "type": "jira",
                    "event": "issue_updated",
                    "since": since,
                }),
                dedupe_key: Some(format!("jira_sync_{}", now.timestamp())),
                created_at: now,
                updated_at: now,
            }],
            next_cursor: Some(Cursor::from_string(next_timestamp.to_string())),
            has_more: true, // Demonstrate pagination for testing
        })
    }

    async fn handle_webhook(
        &self,
        params: WebhookParams,
    ) -> Result<Vec<Signal>, Box<dyn std::error::Error + Send + Sync>> {
        // Filter for issue events; ignore others
        let now = DateTime::from(Utc::now());
        let event_type = params
            .payload
            .get("webhookEvent")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let kind = match event_type {
            // Typical Jira webhook event keys
            "jira:issue_created" => Some("issue_created"),
            "jira:issue_updated" => Some("issue_updated"),
            _ => None,
        };

        if let Some(kind) = kind {
            Ok(vec![Signal {
                id: Uuid::new_v4(),
                tenant_id: params.tenant_id,
                provider_slug: "jira".to_string(),
                connection_id: Uuid::new_v4(),
                kind: kind.to_string(),
                occurred_at: now,
                received_at: now,
                payload: params.payload,
                dedupe_key: None,
                created_at: now,
                updated_at: now,
            }])
        } else {
            Ok(vec![])
        }
    }
}

/// Initialize the Jira connector in the registry
pub fn register_jira_connector(registry: &mut Registry) {
    let metadata = ProviderMetadata::new(
        "jira".to_string(),
        AuthType::OAuth2,
        vec!["read:jira-work".to_string(), "read:jira-user".to_string()],
        true, // webhooks supported
    );

    let connector = Arc::new(JiraConnector);
    registry.register(connector, metadata);
}
