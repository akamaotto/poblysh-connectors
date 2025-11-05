//! Google Drive connector implementation (MVP stub)
//!
//! Implements OAuth authorize URL generation, stub token exchange/refresh,
//! webhook channel handling (headers forwarded in payload), and polling fallback
//! sync returning normalized file change signals.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use url::Url;
use uuid::Uuid;

use crate::connectors::{
    AuthType, Connector, ProviderMetadata, Registry,
    trait_::{AuthorizeParams, ExchangeTokenParams, SyncParams, SyncResult, WebhookParams},
};
use crate::models::{connection::Model as Connection, signal::Model as Signal};

/// Google Drive connector (stub)
pub struct GoogleDriveConnector;

#[async_trait]
impl Connector for GoogleDriveConnector {
    async fn authorize(
        &self,
        params: AuthorizeParams,
    ) -> Result<Url, Box<dyn std::error::Error + Send + Sync>> {
        // Build a Google OAuth authorize URL (stub values for client id)
        let mut url = Url::parse("https://accounts.google.com/o/oauth2/v2/auth")?;
        url.query_pairs_mut()
            .append_pair("client_id", "stub_google_client_id")
            .append_pair(
                "redirect_uri",
                &params
                    .redirect_uri
                    .unwrap_or_else(|| "https://localhost:3000/callback".to_string()),
            )
            .append_pair("scope", "https://www.googleapis.com/auth/drive.readonly")
            .append_pair("response_type", "code")
            .append_pair("access_type", "offline")
            .append_pair(
                "state",
                &params.state.unwrap_or_else(|| "random_state".to_string()),
            );

        Ok(url)
    }

    async fn exchange_token(
        &self,
        params: ExchangeTokenParams,
    ) -> Result<Connection, Box<dyn std::error::Error + Send + Sync>> {
        // Stub token exchange: create a connection record with placeholder tokens
        let now = DateTime::from(Utc::now());
        Ok(Connection {
            id: Uuid::new_v4(),
            tenant_id: params.tenant_id,
            provider_slug: "google-drive".to_string(),
            external_id: "drive-user-123".to_string(),
            status: "active".to_string(),
            display_name: Some("Google Drive".to_string()),
            access_token_ciphertext: Some(b"mock_google_access_token".to_vec()),
            refresh_token_ciphertext: Some(b"mock_google_refresh_token".to_vec()),
            expires_at: Some(now + chrono::Duration::hours(1)),
            scopes: Some(serde_json::json!([
                "https://www.googleapis.com/auth/drive.readonly"
            ])),
            metadata: Some(serde_json::json!({
                "provider": "google-drive",
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
        // Stub refresh: rotate tokens and bump expiry
        let now = DateTime::from(Utc::now());
        Ok(Connection {
            id: connection.id,
            tenant_id: connection.tenant_id,
            provider_slug: connection.provider_slug,
            external_id: connection.external_id,
            status: connection.status,
            display_name: connection.display_name,
            access_token_ciphertext: Some(b"refreshed_google_access_token".to_vec()),
            refresh_token_ciphertext: Some(b"new_google_refresh_token".to_vec()),
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
        // Polling fallback stub: produce a single file_updated signal, include cursor if present
        let now = DateTime::from(Utc::now());
        let cursor = params
            .cursor
            .as_ref()
            .map(|c| c.as_json().clone())
            .unwrap_or_default();

        Ok(SyncResult {
            signals: vec![Signal {
                id: Uuid::new_v4(),
                tenant_id: params.connection.tenant_id,
                provider_slug: "google-drive".to_string(),
                connection_id: params.connection.id,
                kind: "file_updated".to_string(),
                occurred_at: now,
                received_at: now,
                payload: serde_json::json!({
                    "type": "google-drive",
                    "event": "file_updated",
                    "cursor": cursor,
                }),
                dedupe_key: Some(format!("gdrive_sync_{}", now.timestamp())),
                created_at: now,
                updated_at: now,
            }],
            next_cursor: None, // No pagination in this stub implementation
            has_more: false,
        })
    }

    async fn handle_webhook(
        &self,
        params: WebhookParams,
    ) -> Result<Vec<Signal>, Box<dyn std::error::Error + Send + Sync>> {
        // Google Drive pushes key details via headers; platform should forward into payload.headers
        let now = DateTime::from(Utc::now());
        let headers = params
            .payload
            .get("headers")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({}));

        let resource_state = headers
            .get("x-goog-resource-state")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let kind = match resource_state {
            "add" => Some("file_created"),
            "trash" => Some("file_trashed"),
            "update" => Some("file_updated"),
            _ => None,
        };

        if let Some(kind) = kind {
            Ok(vec![Signal {
                id: Uuid::new_v4(),
                tenant_id: params.tenant_id,
                provider_slug: "google-drive".to_string(),
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

/// Initialize the Google Drive connector in the registry
pub fn register_google_drive_connector(registry: &mut Registry) {
    let metadata = ProviderMetadata::new(
        "google-drive".to_string(),
        AuthType::OAuth2,
        vec!["https://www.googleapis.com/auth/drive.readonly".to_string()],
        true, // webhooks supported
    );

    let connector = Arc::new(GoogleDriveConnector);
    registry.register(connector, metadata);
}
