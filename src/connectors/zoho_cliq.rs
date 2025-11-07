//! Zoho Cliq connector implementation
//!
//! Webhook-only Zoho Cliq connector satisfying the Connector trait.
//! MVP supports token-based authentication and message event ingestion.

use anyhow::anyhow;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::sync::Arc;
use tracing::{debug, info, warn};
use url::Url;

use uuid::Uuid;

use crate::connectors::{
    AuthType, Connector, ProviderMetadata, Registry,
    trait_::{AuthorizeParams, ExchangeTokenParams, SyncParams, SyncResult, WebhookParams},
};
use crate::models::{connection::Model as Connection, signal::Model as Signal};

/// Zoho Cliq connector
#[derive(Debug)]
pub struct ZohoCliqConnector;

impl ZohoCliqConnector {
    /// Create a new Zoho Cliq connector
    pub fn new() -> Self {
        Self
    }
}

impl Default for ZohoCliqConnector {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
struct ZohoCliqMessageEvent {
    /// Event type from Zoho Cliq
    #[serde(rename = "event_type")]
    event_type: String,
    /// Message data
    message: ZohoCliqMessage,
    /// User who performed the action
    user: ZohoCliqUser,
    /// Channel where the action occurred
    chat: ZohoCliqChat,
    /// Timestamp of the event
    #[serde(rename = "time_stamp")]
    timestamp: String,
}

#[derive(Debug, Deserialize)]
struct ZohoCliqMessage {
    /// Unique message identifier
    id: String,
    /// Message content
    text: Option<String>,
    /// Message type (text, image, file, etc.)
    #[serde(rename = "message_type")]
    message_type: String,
    /// Time when message was posted
    #[serde(rename = "posted_time")]
    posted_time: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ZohoCliqUser {
    /// Unique user identifier
    id: String,
    /// User's display name
    #[serde(rename = "first_name")]
    first_name: Option<String>,
    #[serde(rename = "last_name")]
    last_name: Option<String>,
    /// User's email address
    email: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ZohoCliqChat {
    /// Unique chat/channel identifier
    id: String,
    /// Chat name (for channels) or chat type
    name: Option<String>,
    /// Chat type (group, direct, etc.)
    #[serde(rename = "chat_type")]
    chat_type: Option<String>,
}

#[async_trait]
impl Connector for ZohoCliqConnector {
    async fn authorize(
        &self,
        _params: AuthorizeParams,
    ) -> Result<Url, Box<dyn std::error::Error + Send + Sync>> {
        // OAuth is not supported in MVP for Zoho Cliq
        Err(anyhow!("OAuth authorization is not supported for Zoho Cliq in MVP").into())
    }

    async fn exchange_token(
        &self,
        _params: ExchangeTokenParams,
    ) -> Result<Connection, Box<dyn std::error::Error + Send + Sync>> {
        // OAuth is not supported in MVP for Zoho Cliq
        Err(anyhow!("Token exchange is not supported for Zoho Cliq in MVP").into())
    }

    async fn refresh_token(
        &self,
        _connection: Connection,
    ) -> Result<Connection, Box<dyn std::error::Error + Send + Sync>> {
        // OAuth is not supported in MVP for Zoho Cliq
        Err(anyhow!("Token refresh is not supported for Zoho Cliq in MVP").into())
    }

    async fn sync(
        &self,
        _params: SyncParams,
    ) -> Result<SyncResult, Box<dyn std::error::Error + Send + Sync>> {
        // Historical sync/backfill is not supported in MVP for Zoho Cliq
        warn!("Sync operation called for Zoho Cliq but not supported in MVP");
        Ok(SyncResult {
            signals: vec![],
            next_cursor: None,
            has_more: false,
        })
    }

    async fn handle_webhook(
        &self,
        params: WebhookParams,
    ) -> Result<Vec<Signal>, Box<dyn std::error::Error + Send + Sync>> {
        let received_at = DateTime::from(Utc::now());

        debug!(
            tenant_id = %params.tenant_id,
            "Processing Zoho Cliq webhook"
        );

        // First extract the event type to check if it's supported
        let event_type = params.payload.get("event_type")
            .and_then(|v| v.as_str());

        // If there's no event_type, the payload is malformed
        let event_type = match event_type {
            Some(event_type) => event_type,
            None => {
                return Err(anyhow!("Invalid Zoho Cliq webhook payload: missing event_type").into());
            }
        };

        // Map event type to signal kind
        let signal_kind = match event_type {
            "message_posted" => "message_posted",
            "message_updated" => "message_updated",
            "message_deleted" => "message_deleted",
            _ => {
                debug!(
                    event_type = %event_type,
                    "Ignoring Zoho Cliq event type"
                );
                return Ok(vec![]);
            }
        };

        // Now try to parse the payload as a Zoho Cliq event for supported types
        let event: ZohoCliqMessageEvent = serde_json::from_value(params.payload.clone())
            .map_err(|e| {
                debug!(error = %e, "Failed to parse Zoho Cliq webhook payload");
                anyhow!("Invalid Zoho Cliq webhook payload: {}", e)
            })?;

        info!(
            tenant_id = %params.tenant_id,
            event_type = %event.event_type,
            signal_kind = %signal_kind,
            message_id = %event.message.id,
            "Zoho Cliq webhook mapped to signal"
        );

        // Extract normalized fields from Zoho Cliq webhook payload
        let normalized_payload = extract_normalized_fields(&event);
        let occurred_at = parse_zoho_timestamp(&event.message.posted_time)
            .unwrap_or_else(|| parse_zoho_timestamp(&Some(event.timestamp.clone())).unwrap_or_else(Utc::now));

        // Generate dedupe key using message ID and event type
        let dedupe_key = format!("zoho-cliq:{}:{}:{}", signal_kind, event.message.id, occurred_at.timestamp());

        Ok(vec![Signal {
            id: Uuid::new_v4(),
            tenant_id: params.tenant_id,
            provider_slug: "zoho-cliq".to_string(),
            connection_id: Uuid::new_v4(), // Will be populated by webhook handler
            kind: signal_kind.to_string(),
            occurred_at: occurred_at.into(),
            received_at,
            payload: normalized_payload,
            dedupe_key: Some(dedupe_key),
            created_at: received_at,
            updated_at: received_at,
        }])
    }
}

/// Initialize the Zoho Cliq connector in the registry
pub fn register_zoho_cliq_connector(registry: &mut Registry, connector: Arc<ZohoCliqConnector>) {
    let metadata = ProviderMetadata::new(
        "zoho-cliq".to_string(),
        AuthType::Custom("webhook".to_string()),
        vec![], // No OAuth scopes in MVP
        true,   // Webhooks supported
    );

    registry.register(connector, metadata);
}

/// Extract normalized fields from Zoho Cliq webhook payload
fn extract_normalized_fields(event: &ZohoCliqMessageEvent) -> serde_json::Value {
    let user_display_name = match (&event.user.first_name, &event.user.last_name) {
        (Some(first), Some(last)) => format!("{} {}", first, last),
        (Some(first), None) => first.clone(),
        (None, Some(last)) => last.clone(),
        (None, None) => event.user.id.clone(),
    };

    serde_json::json!({
        "message_id": event.message.id,
        "channel_id": event.chat.id,
        "channel_name": event.chat.name,
        "channel_type": event.chat.chat_type,
        "user_id": event.user.id,
        "user_name": user_display_name,
        "user_email": event.user.email,
        "text": event.message.text,
        "message_type": event.message.message_type,
        "occurred_at": parse_zoho_timestamp(&event.message.posted_time)
            .or_else(|| parse_zoho_timestamp(&Some(event.timestamp.clone())))
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_else(|| Utc::now().to_rfc3339()),
    })
}

/// Parse Zoho Cliq timestamp format to DateTime<Utc>
fn parse_zoho_timestamp(timestamp_str: &Option<String>) -> Option<DateTime<Utc>> {
    match timestamp_str {
        Some(ts) => {
            // Try RFC3339 format first (most explicit)
            if let Ok(dt) = DateTime::parse_from_rfc3339(ts) {
                return Some(dt.with_timezone(&Utc));
            }

            // Try numeric timestamp formats
            if let Ok(timestamp) = ts.parse::<i64>() {
                // Distinguish between seconds and milliseconds based on length:
                // - 10 digits: seconds (e.g., 1699123456)
                // - 13 digits: milliseconds (e.g., 1699123456789)
                // - Other lengths: try as seconds first, then milliseconds
                let seconds = if ts.len() == 13 {
                    // Milliseconds - convert to seconds
                    timestamp / 1000
                } else if ts.len() >= 10 && ts.len() <= 12 {
                    // Seconds
                    timestamp
                } else {
                    // Ambiguous length, try both approaches
                    // If it's too large to be seconds, treat as milliseconds
                    if timestamp > 10_i64.pow(10) {
                        timestamp / 1000
                    } else {
                        timestamp
                    }
                };

                return DateTime::from_timestamp(seconds, 0);
            }

            warn!("Failed to parse Zoho Cliq timestamp: {}", ts);
            None
        }
        None => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connectors::trait_::{WebhookParams};
    use chrono::{Datelike, Timelike};
    use uuid::Uuid;

    #[tokio::test]
    async fn test_zoho_cliq_webhook_message_posted() {
        let connector = ZohoCliqConnector::new();
        let tenant_id = Uuid::new_v4();

        let payload = serde_json::json!({
            "event_type": "message_posted",
            "message": {
                "id": "msg_12345",
                "text": "Hello, team!",
                "message_type": "text",
                "posted_time": "1699123456"
            },
            "user": {
                "id": "user_67890",
                "first_name": "John",
                "last_name": "Doe",
                "email": "john.doe@example.com"
            },
            "chat": {
                "id": "chat_11111",
                "name": "general",
                "chat_type": "group"
            },
            "time_stamp": "1699123456"
        });

        let params = WebhookParams {
            tenant_id,
            payload,
            db: None,
            auth_header: None,
        };

        let signals = connector.handle_webhook(params).await.unwrap();
        assert_eq!(signals.len(), 1);
        assert_eq!(signals[0].kind, "message_posted");
        assert_eq!(signals[0].provider_slug, "zoho-cliq");
        assert_eq!(signals[0].tenant_id, tenant_id);

        // Verify normalized payload
        let payload = &signals[0].payload;
        assert_eq!(payload.get("message_id").unwrap(), "msg_12345");
        assert_eq!(payload.get("channel_id").unwrap(), "chat_11111");
        assert_eq!(payload.get("user_name").unwrap(), "John Doe");
        assert_eq!(payload.get("text").unwrap(), "Hello, team!");

        // Verify dedupe key format
        let dedupe_key = signals[0].dedupe_key.as_ref().unwrap();
        assert!(dedupe_key.starts_with("zoho-cliq:message_posted:msg_12345:"));
    }

    #[tokio::test]
    async fn test_zoho_cliq_webhook_message_updated() {
        let connector = ZohoCliqConnector::new();
        let tenant_id = Uuid::new_v4();

        let payload = serde_json::json!({
            "event_type": "message_updated",
            "message": {
                "id": "msg_12345",
                "text": "Hello, team! (edited)",
                "message_type": "text",
                "posted_time": "1699123456"
            },
            "user": {
                "id": "user_67890",
                "first_name": "John",
                "last_name": "Doe"
            },
            "chat": {
                "id": "chat_11111",
                "name": "general",
                "chat_type": "group"
            },
            "time_stamp": "1699123467"
        });

        let params = WebhookParams {
            tenant_id,
            payload,
            db: None,
            auth_header: None,
        };

        let signals = connector.handle_webhook(params).await.unwrap();
        assert_eq!(signals.len(), 1);
        assert_eq!(signals[0].kind, "message_updated");

        let dedupe_key = signals[0].dedupe_key.as_ref().unwrap();
        assert!(dedupe_key.starts_with("zoho-cliq:message_updated:msg_12345:"));
    }

    #[tokio::test]
    async fn test_zoho_cliq_webhook_message_deleted() {
        let connector = ZohoCliqConnector::new();
        let tenant_id = Uuid::new_v4();

        let payload = serde_json::json!({
            "event_type": "message_deleted",
            "message": {
                "id": "msg_12345",
                "message_type": "text",
                "posted_time": "1699123456"
            },
            "user": {
                "id": "user_67890",
                "first_name": "John",
                "last_name": "Doe"
            },
            "chat": {
                "id": "chat_11111",
                "name": "general",
                "chat_type": "group"
            },
            "time_stamp": "1699123500"
        });

        let params = WebhookParams {
            tenant_id,
            payload,
            db: None,
            auth_header: None,
        };

        let signals = connector.handle_webhook(params).await.unwrap();
        assert_eq!(signals.len(), 1);
        assert_eq!(signals[0].kind, "message_deleted");

        let dedupe_key = signals[0].dedupe_key.as_ref().unwrap();
        assert!(dedupe_key.starts_with("zoho-cliq:message_deleted:msg_12345:"));
    }

    #[tokio::test]
    async fn test_zoho_cliq_webhook_unsupported_event() {
        let connector = ZohoCliqConnector::new();
        let tenant_id = Uuid::new_v4();

        let payload = serde_json::json!({
            "event_type": "user_joined",
            "user": {
                "id": "user_67890"
            },
            "chat": {
                "id": "chat_11111"
            },
            "time_stamp": "1699123456"
        });

        let params = WebhookParams {
            tenant_id,
            payload,
            db: None,
            auth_header: None,
        };

        let signals = connector.handle_webhook(params).await.unwrap();
        assert_eq!(signals.len(), 0); // Should be ignored
    }

    #[tokio::test]
    async fn test_zoho_cliq_webhook_invalid_payload() {
        let connector = ZohoCliqConnector::new();
        let tenant_id = Uuid::new_v4();

        let payload = serde_json::json!({
            "invalid": "payload"
        });

        let params = WebhookParams {
            tenant_id,
            payload,
            db: None,
            auth_header: None,
        };

        let result = connector.handle_webhook(params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_zoho_cliq_oauth_not_supported() {
        let connector = ZohoCliqConnector::new();
        let tenant_id = Uuid::new_v4();

        let params = AuthorizeParams {
            tenant_id,
            redirect_uri: None,
            state: None,
        };

        let result = connector.authorize(params).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("OAuth authorization is not supported"));

        let exchange_params = ExchangeTokenParams {
            code: "test_code".to_string(),
            redirect_uri: None,
            tenant_id,
        };

        let result = connector.exchange_token(exchange_params).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Token exchange is not supported"));
    }

    #[test]
    fn test_parse_zoho_timestamp_unix_seconds() {
        let timestamp = Some("1699123456".to_string());
        let result = parse_zoho_timestamp(&timestamp);
        assert!(result.is_some());

        // Should be November 4, 2023, 18:44:16 UTC
        let dt = result.unwrap();
        assert_eq!(dt.year(), 2023);
        assert_eq!(dt.month(), 11);
        assert_eq!(dt.day(), 4);
        assert_eq!(dt.hour(), 18);
        assert_eq!(dt.minute(), 44);
        assert_eq!(dt.second(), 16);
    }

    #[test]
    fn test_parse_zoho_timestamp_milliseconds() {
        let timestamp = Some("1699123456789".to_string()); // 13 digits
        let result = parse_zoho_timestamp(&timestamp);
        assert!(result.is_some());

        // Should be November 4, 2023, 18:44:16 UTC (milliseconds truncated)
        let dt = result.unwrap();
        assert_eq!(dt.year(), 2023);
        assert_eq!(dt.month(), 11);
        assert_eq!(dt.day(), 4);
        assert_eq!(dt.hour(), 18);
        assert_eq!(dt.minute(), 44);
        assert_eq!(dt.second(), 16);
    }

    #[test]
    fn test_parse_zoho_timestamp_iso8601() {
        let timestamp = Some("2023-11-04T18:50:56Z".to_string());
        let result = parse_zoho_timestamp(&timestamp);
        assert!(result.is_some());

        let dt = result.unwrap();
        assert_eq!(dt.year(), 2023);
        assert_eq!(dt.month(), 11);
        assert_eq!(dt.day(), 4);
        assert_eq!(dt.hour(), 18);
        assert_eq!(dt.minute(), 50);
        assert_eq!(dt.second(), 56);
    }

    #[test]
    fn test_parse_zoho_timestamp_iso8601_with_milliseconds() {
        let timestamp = Some("2023-11-04T18:50:56.123Z".to_string());
        let result = parse_zoho_timestamp(&timestamp);
        assert!(result.is_some());

        let dt = result.unwrap();
        assert_eq!(dt.year(), 2023);
        assert_eq!(dt.month(), 11);
        assert_eq!(dt.day(), 4);
        assert_eq!(dt.hour(), 18);
        assert_eq!(dt.minute(), 50);
        assert_eq!(dt.second(), 56);
    }

    #[test]
    fn test_parse_zoho_timestamp_small_number() {
        let timestamp = Some("123456789".to_string()); // 9 digits, treated as seconds
        let result = parse_zoho_timestamp(&timestamp);
        assert!(result.is_some());

        let dt = result.unwrap();
        assert_eq!(dt.year(), 1973);
        assert_eq!(dt.month(), 11);
        assert_eq!(dt.day(), 29);
    }

    #[test]
    fn test_parse_zoho_timestamp_large_number() {
        let timestamp = Some("12345678901234".to_string()); // 14 digits, should be treated as milliseconds
        let result = parse_zoho_timestamp(&timestamp);
        assert!(result.is_some());

        let dt = result.unwrap();
        assert_eq!(dt.year(), 2361);
        assert_eq!(dt.month(), 3);
    }

    #[test]
    fn test_parse_zoho_timestamp_invalid() {
        let timestamp = Some("invalid_timestamp".to_string());
        let result = parse_zoho_timestamp(&timestamp);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_zoho_timestamp_none() {
        let timestamp: Option<String> = None;
        let result = parse_zoho_timestamp(&timestamp);
        assert!(result.is_none());
    }
}