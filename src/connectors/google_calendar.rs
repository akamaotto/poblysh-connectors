//! Google Calendar connector implementation (MVP stub)
//!
//! Implements OAuth authorize URL generation, stub token exchange/refresh,
//! webhook channel handling (headers forwarded in payload), and incremental sync
//! using Google Calendar Events API with syncToken for incremental updates.
//!
//! ## Webhook Headers
//!
//! Google Calendar Channel notifications send metadata via HTTP headers. The platform
//! forwards these headers into the webhook payload under `payload.headers.*` with
//! lower-case names:
//!
//! - `x-goog-channel-id` - Unique identifier for the notification channel
//! - `x-goog-resource-id` - Identifier for the monitored resource
//! - `x-goog-resource-state` - State change (`sync`, `exists`, etc.)
//! - `x-goog-message-number` - Sequential message number for the channel
//! - `x-goog-resource-uri` - URI for the resource (when present)
//!
//! ## Supported Events
//!
//! - `event_updated` - Calendar event created or updated
//! - `event_deleted` - Calendar event deleted

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

/// Google Calendar connector (MVP stub implementation)
///
/// Provides OAuth2 authorization, token exchange/refresh, webhook handling for
/// Google Calendar Channel notifications, and incremental sync using syncToken.
pub struct GoogleCalendarConnector;

#[async_trait]
impl Connector for GoogleCalendarConnector {
    async fn authorize(
        &self,
        params: AuthorizeParams,
    ) -> Result<Url, Box<dyn std::error::Error + Send + Sync>> {
        // Build a Google OAuth authorize URL for Calendar scope
        let mut url = Url::parse("https://accounts.google.com/o/oauth2/v2/auth")
            .map_err(|e| format!("Failed to parse Google OAuth URL: {}", e))?;
        url.query_pairs_mut()
            .append_pair("client_id", "stub_google_calendar_client_id")
            .append_pair(
                "redirect_uri",
                &params
                    .redirect_uri
                    .unwrap_or_else(|| "https://localhost:3000/callback".to_string()),
            )
            .append_pair("scope", "https://www.googleapis.com/auth/calendar.readonly")
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
        // In production, this would exchange the authorization code for access/refresh tokens
        let now = DateTime::from(Utc::now());
        Ok(Connection {
            id: Uuid::new_v4(),
            tenant_id: params.tenant_id,
            provider_slug: "google-calendar".to_string(),
            external_id: "calendar-user-123".to_string(),
            status: "active".to_string(),
            display_name: Some("Google Calendar".to_string()),
            access_token_ciphertext: Some(b"mock_google_calendar_access_token".to_vec()),
            refresh_token_ciphertext: Some(b"mock_google_calendar_refresh_token".to_vec()),
            expires_at: Some(now + chrono::Duration::hours(1)),
            scopes: Some(serde_json::json!([
                "https://www.googleapis.com/auth/calendar.readonly"
            ])),
            metadata: Some(serde_json::json!({
                "provider": "google-calendar",
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
        // In production, this would use the refresh token to get a new access token
        let now = DateTime::from(Utc::now());
        Ok(Connection {
            id: connection.id,
            tenant_id: connection.tenant_id,
            provider_slug: connection.provider_slug,
            external_id: connection.external_id,
            status: connection.status,
            display_name: connection.display_name,
            access_token_ciphertext: Some(b"refreshed_google_calendar_access_token".to_vec()),
            refresh_token_ciphertext: Some(b"new_google_calendar_refresh_token".to_vec()),
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
        // Incremental sync stub: produce event signals using syncToken for pagination
        // In production, this would use Google Calendar API events.list with syncToken
        let now = DateTime::from(Utc::now());
        let cursor = params
            .cursor
            .as_ref()
            .map(|c| c.as_json().clone())
            .unwrap_or_default();

        // Mock nextSyncToken for incremental sync
        let next_sync_token = format!("mock_next_sync_token_{}", now.timestamp());

        Ok(SyncResult {
            signals: vec![
                Signal {
                    id: Uuid::new_v4(),
                    tenant_id: params.connection.tenant_id,
                    provider_slug: "google-calendar".to_string(),
                    connection_id: params.connection.id,
                    kind: "event_updated".to_string(),
                    occurred_at: now,
                    received_at: now,
                    payload: serde_json::json!({
                        "type": "google-calendar",
                        "event": "event_updated",
                        "cursor": cursor,
                        "syncToken": next_sync_token,
                    }),
                    dedupe_key: Some(format!("gcal_sync_{}", now.timestamp())),
                    created_at: now,
                    updated_at: now,
                },
                Signal {
                    id: Uuid::new_v4(),
                    tenant_id: params.connection.tenant_id,
                    provider_slug: "google-calendar".to_string(),
                    connection_id: params.connection.id,
                    kind: "event_deleted".to_string(),
                    occurred_at: now - chrono::Duration::minutes(30),
                    received_at: now,
                    payload: serde_json::json!({
                        "type": "google-calendar",
                        "event": "event_deleted",
                        "cursor": cursor,
                        "syncToken": next_sync_token,
                    }),
                    dedupe_key: Some(format!("gcal_deleted_{}", now.timestamp() - 1800)),
                    created_at: now,
                    updated_at: now,
                },
            ],
            next_cursor: Some(crate::connectors::trait_::Cursor::from_string(
                next_sync_token,
            )),
            has_more: false, // No more events in this stub implementation
        })
    }

    async fn handle_webhook(
        &self,
        params: WebhookParams,
    ) -> Result<Vec<Signal>, Box<dyn std::error::Error + Send + Sync>> {
        // Google Calendar pushes key details via headers; platform should forward into payload.headers
        // Process Calendar Channel notifications and enqueue sync jobs (no direct signals per MVP)
        let headers = params
            .payload
            .get("headers")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({}));

        let resource_state = headers
            .get("x-goog-resource-state")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // For MVP, webhook handler returns no signals and relies on sync job enqueueing
        // In production, this would trigger an incremental sync for the connection
        match resource_state {
            "sync" | "exists" => {
                // Valid calendar channel states - these would trigger sync jobs
                // Return empty signals per MVP specification
                Ok(vec![])
            }
            _ => {
                // Unknown or unhandled states
                Ok(vec![])
            }
        }
    }
}

/// Initialize the Google Calendar connector in the registry
pub fn register_google_calendar_connector(registry: &mut Registry) {
    let metadata = ProviderMetadata::new(
        "google-calendar".to_string(),
        AuthType::OAuth2,
        vec!["https://www.googleapis.com/auth/calendar.readonly".to_string()],
        true, // webhooks supported
    );

    let connector = Arc::new(GoogleCalendarConnector);
    registry.register(connector, metadata);
}
