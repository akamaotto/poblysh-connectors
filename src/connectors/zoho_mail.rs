//! Zoho Mail connector implementation (skeleton)
//!
//! This module provides a `ZohoMailConnector` implementation that satisfies the
//! `Connector` trait and aligns with the `add-zoho-mail-connector` OpenSpec
//! change. It is intentionally minimal and focuses on:
//! - Provider metadata for `zoho-mail`
//! - OAuth2 authorization URL generation (region-aware)
//! - Stubs for token exchange, refresh, and sync
//! - Explicitly unsupported webhooks
//!
//! The HTTP calls and DB persistence are left as TODOs for follow-up tasks.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::connectors::metadata::{AuthType, ProviderMetadata};
use crate::connectors::trait_::{
    AuthorizeParams, Connector, Cursor, ExchangeTokenParams, SyncError, SyncParams, SyncResult,
    WebhookParams,
};
use crate::models::{connection::Model as Connection, signal::Model as Signal};

/// Provider slug for Zoho Mail.
pub const ZOHO_MAIL_PROVIDER_SLUG: &str = "zoho-mail";

/// Default Zoho Mail OAuth scope for read-only message access.
pub const DEFAULT_ZOHO_MAIL_SCOPE: &str = "ZohoMail.messages.READ";

/// Default dedupe window in seconds (5 minutes).
pub const DEFAULT_DEDUPE_WINDOW_SECS: u64 = 300;

/// Default HTTP timeout for Zoho Mail operations in seconds.
pub const DEFAULT_HTTP_TIMEOUT_SECS: u64 = 15;

/// Supported Zoho data centers.
///
/// This list is intentionally explicit but can be extended without breaking changes.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ZohoDataCenter {
    Us,
    Eu,
    In,
    Au,
    Jp,
    Ca,
    Sa,
    Uk,
}

impl ZohoDataCenter {
    pub fn from_str(dc: &str) -> Option<Self> {
        match dc.to_ascii_lowercase().as_str() {
            "us" => Some(Self::Us),
            "eu" => Some(Self::Eu),
            "in" => Some(Self::In),
            "au" => Some(Self::Au),
            "jp" => Some(Self::Jp),
            "ca" => Some(Self::Ca),
            "sa" => Some(Self::Sa),
            "uk" => Some(Self::Uk),
            _ => None,
        }
    }

    pub fn accounts_base(&self) -> &'static str {
        match self {
            ZohoDataCenter::Us => "https://accounts.zoho.com",
            ZohoDataCenter::Eu => "https://accounts.zoho.eu",
            ZohoDataCenter::In => "https://accounts.zoho.in",
            ZohoDataCenter::Au => "https://accounts.zoho.com.au",
            ZohoDataCenter::Jp => "https://accounts.zoho.jp",
            ZohoDataCenter::Ca => "https://accounts.zohocloud.ca",
            ZohoDataCenter::Sa => "https://accounts.zoho.sa",
            ZohoDataCenter::Uk => "https://accounts.zoho.uk",
        }
    }

    pub fn mail_api_base(&self) -> &'static str {
        // Representative; adjust as Zoho Mail API docs confirm.
        match self {
            ZohoDataCenter::Us => "https://mail.zoho.com",
            ZohoDataCenter::Eu => "https://mail.zoho.eu",
            ZohoDataCenter::In => "https://mail.zoho.in",
            ZohoDataCenter::Au => "https://mail.zoho.com.au",
            ZohoDataCenter::Jp => "https://mail.zoho.jp",
            ZohoDataCenter::Ca => "https://mail.zoho.com",
            ZohoDataCenter::Sa => "https://mail.zoho.sa",
            ZohoDataCenter::Uk => "https://mail.zoho.uk",
        }
    }
}

/// Configuration for the Zoho Mail connector.
///
/// Values are resolved from environment variables for now:
/// - POBLYSH_ZOHO_MAIL_CLIENT_ID
/// - POBLYSH_ZOHO_MAIL_CLIENT_SECRET
/// - POBLYSH_ZOHO_MAIL_DC
/// - POBLYSH_ZOHO_MAIL_SCOPES (optional)
/// - POBLYSH_ZOHO_MAIL_DEDUPE_WINDOW_SECS (optional)
/// - POBLYSH_ZOHO_MAIL_HTTP_TIMEOUT_SECS (optional)
#[derive(Debug, Clone)]
pub struct ZohoMailConfig {
    pub client_id: String,
    pub client_secret: String,
    pub dc: ZohoDataCenter,
    pub scopes: Vec<String>,
    pub dedupe_window_secs: u64,
    pub http_timeout_secs: u64,
}

impl ZohoMailConfig {
    pub fn from_env() -> Result<Self, SyncError> {
        let client_id = std::env::var("POBLYSH_ZOHO_MAIL_CLIENT_ID").map_err(|_| {
            SyncError::permanent("Missing POBLYSH_ZOHO_MAIL_CLIENT_ID for Zoho Mail")
        })?;

        let client_secret = std::env::var("POBLYSH_ZOHO_MAIL_CLIENT_SECRET").map_err(|_| {
            SyncError::permanent("Missing POBLYSH_ZOHO_MAIL_CLIENT_SECRET for Zoho Mail")
        })?;

        let dc_raw = std::env::var("POBLYSH_ZOHO_MAIL_DC")
            .map_err(|_| SyncError::permanent("Missing POBLYSH_ZOHO_MAIL_DC for Zoho Mail"))?;
        let dc = ZohoDataCenter::from_str(&dc_raw).ok_or_else(|| {
            SyncError::permanent(format!(
                "Invalid POBLYSH_ZOHO_MAIL_DC '{}': must be one of us,eu,in,au,jp,ca,sa,uk",
                dc_raw
            ))
        })?;

        let scopes = std::env::var("POBLYSH_ZOHO_MAIL_SCOPES")
            .ok()
            .map(parse_scopes)
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| vec![DEFAULT_ZOHO_MAIL_SCOPE.to_string()]);

        let dedupe_window_secs = std::env::var("POBLYSH_ZOHO_MAIL_DEDUPE_WINDOW_SECS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(DEFAULT_DEDUPE_WINDOW_SECS);

        let http_timeout_secs = std::env::var("POBLYSH_ZOHO_MAIL_HTTP_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(DEFAULT_HTTP_TIMEOUT_SECS);

        if dedupe_window_secs == 0 {
            return Err(SyncError::permanent(
                "POBLYSH_ZOHO_MAIL_DEDUPE_WINDOW_SECS must be > 0",
            ));
        }
        if http_timeout_secs == 0 {
            return Err(SyncError::permanent(
                "POBLYSH_ZOHO_MAIL_HTTP_TIMEOUT_SECS must be > 0",
            ));
        }

        Ok(Self {
            client_id,
            client_secret,
            dc,
            scopes,
            dedupe_window_secs,
            http_timeout_secs,
        })
    }
}

fn parse_scopes(raw: String) -> Vec<String> {
    raw.split(|c: char| c == ' ' || c == ',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToString::to_string)
        .collect()
}

/// Minimal Zoho Mail connector.
///
/// At this stage, HTTP and database operations are placeholders; the goal is to
/// provide a compilable implementation that follows the trait and spec shape.
#[derive(Clone)]
pub struct ZohoMailConnector {
    pub(crate) config: ZohoMailConfig,
    // Future: add `http_client: reqwest::Client` and other dependencies.
}

impl ZohoMailConnector {
    pub fn new_from_env() -> Result<Self, SyncError> {
        let config = ZohoMailConfig::from_env()?;
        Self::new(config)
    }

    pub fn new(config: ZohoMailConfig) -> Result<Self, SyncError> {
        Ok(Self {
            config,
        })
    }

    /// Build authorization URL for a tenant with configured scopes.
    fn build_authorize_url(&self, params: &AuthorizeParams) -> Result<Url, SyncError> {
        let accounts_base = self.config.dc.accounts_base();
        let mut url = Url::parse(&format!("{}/oauth/v2/auth", accounts_base))
            .map_err(|e| SyncError::permanent(format!("Invalid Zoho auth URL: {e}")))?;

        url.query_pairs_mut()
            .append_pair("client_id", &self.config.client_id)
            .append_pair("response_type", "code")
            .append_pair("access_type", "offline")
            .append_pair(
                "redirect_uri",
                params.redirect_uri.as_deref().unwrap_or("https://localhost:3000/callback"),
            )
            .append_pair(
                "state",
                params.state.as_deref().unwrap_or(&format!(
                    "tenant:{}:{}",
                    params.tenant_id, ZOHO_MAIL_PROVIDER_SLUG
                )),
            );

        // Add scopes
        if !self.config.scopes.is_empty() {
            let scopes_str = self.config.scopes.join(" ");
            url.query_pairs_mut().append_pair("scope", &scopes_str);
        }

        Ok(url)
    }

    /// Compute next cursor from latest lastModifiedTime as RFC3339.
    fn build_cursor_from_ts(ts: DateTime<Utc>) -> Cursor {
        Cursor::from_string(ts.to_rfc3339())
    }

    /// Compute dedupe key from message_id and lastModifiedTime.
    ///
    /// Placeholder: real implementation should use a stable hash.
    #[allow(dead_code)]
    fn build_dedupe_key(message_id: &str, last_modified_time: &str) -> String {
        format!("{}::{}", message_id, last_modified_time)
    }

    /// Build a minimal signal payload map for normalized Zoho Mail events.
    ///
    /// Placeholder until concrete mappings are implemented.
    #[allow(dead_code)]
    fn build_signal_payload(
        &self,
        kind: &str,
        message_id: &str,
        occurred_at: DateTime<Utc>,
    ) -> HashMap<String, serde_json::Value> {
        let mut map = HashMap::new();
        map.insert(
            "provider".to_string(),
            serde_json::Value::String("zoho-mail".to_string()),
        );
        map.insert(
            "kind".to_string(),
            serde_json::Value::String(kind.to_string()),
        );
        map.insert(
            "message_id".to_string(),
            serde_json::Value::String(message_id.to_string()),
        );
        map.insert(
            "occurred_at".to_string(),
            serde_json::Value::String(occurred_at.to_rfc3339()),
        );
        map
    }

    /// HTTP timeout duration used for Zoho Mail calls.
    pub fn http_timeout(&self) -> Duration {
        Duration::from_secs(self.config.http_timeout_secs)
    }
}

/// Register Zoho Mail connector in the provider registry.
///
/// Intended to be called from `Registry::initialize` when configuration is present.
pub fn register_zoho_mail_connector(
    registry: &mut crate::connectors::registry::Registry,
    connector: Arc<ZohoMailConnector>,
) {
    let metadata = ProviderMetadata::new(
        ZOHO_MAIL_PROVIDER_SLUG.to_string(),
        AuthType::OAuth2,
        connector.config.scopes.clone(),
        false, // webhooks=false per MVP
    );

    registry.register(connector, metadata);
}

#[async_trait]
impl Connector for ZohoMailConnector {
    async fn authorize(
        &self,
        params: AuthorizeParams,
    ) -> Result<Url, Box<dyn std::error::Error + Send + Sync>> {
        let url = self
            .build_authorize_url(&params)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
        Ok(url)
    }

    async fn exchange_token(
        &self,
        params: ExchangeTokenParams,
    ) -> Result<Connection, Box<dyn std::error::Error + Send + Sync>> {
        // TODO:
        // - Call Zoho token endpoint with authorization code.
        // - Persist to `connections` using project repositories and crypto.
        // - Include Zoho account/user identifiers in metadata.
        Err(Box::new(SyncError::permanent(format!(
            "Zoho Mail exchange_token not implemented for tenant {}",
            params.tenant_id
        ))))
    }

    async fn refresh_token(
        &self,
        connection: Connection,
    ) -> Result<Connection, Box<dyn std::error::Error + Send + Sync>> {
        // TODO:
        // - Use refresh token to obtain new access token.
        // - Update `connections` row accordingly.
        Err(Box::new(SyncError::permanent(format!(
            "Zoho Mail refresh_token not implemented for connection {}",
            connection.id
        ))))
    }

    async fn sync(
        &self,
        params: SyncParams,
    ) -> Result<SyncResult, Box<dyn std::error::Error + Send + Sync>> {
        // Spec alignment sketch:
        // - If no cursor: establish baseline at now() without historical backfill.
        // - If cursor exists: query messages with lastModifiedTime >= (cursor - window),
        //   dedupe via hash(message_id || lastModifiedTime), emit Signals, advance cursor.
        //
        // Skeleton implementation:
        // - Establish baseline cursor on first run.
        // - Return empty result set with has_more = false.

        let next_cursor = if let Some(existing) = params.cursor {
            Some(existing)
        } else {
            let now = Utc::now();
            Some(Self::build_cursor_from_ts(now))
        };

        Ok(SyncResult {
            signals: Vec::new(),
            next_cursor,
            has_more: false,
        })
    }

    async fn handle_webhook(
        &self,
        _params: WebhookParams,
    ) -> Result<Vec<Signal>, Box<dyn std::error::Error + Send + Sync>> {
        // Webhooks are explicitly not supported for Zoho Mail in this MVP.
        let error = SyncError::permanent(
            "WEBHOOKS_NOT_SUPPORTED: Webhooks not supported for Zoho Mail connector",
        );
        Err(Box::new(error))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connectors::trait_::AuthorizeParams;
    use uuid::Uuid;

    #[test]
    fn dc_from_str_parses_known_values() {
        assert!(matches!(
            ZohoDataCenter::from_str("us"),
            Some(ZohoDataCenter::Us)
        ));
        assert!(matches!(
            ZohoDataCenter::from_str("EU"),
            Some(ZohoDataCenter::Eu)
        ));
        assert!(ZohoDataCenter::from_str("unknown").is_none());
    }

    #[test]
    fn parse_scopes_splits_on_space_and_comma() {
        let scopes = parse_scopes("a b,c".to_string());
        assert_eq!(scopes, vec!["a", "b", "c"]);
    }

    #[tokio::test]
    async fn authorize_builds_region_aware_url() {
        let config = ZohoMailConfig {
            client_id: "id".to_string(),
            client_secret: "secret".to_string(),
            dc: ZohoDataCenter::Us,
            scopes: vec![DEFAULT_ZOHO_MAIL_SCOPE.to_string()],
            dedupe_window_secs: DEFAULT_DEDUPE_WINDOW_SECS,
            http_timeout_secs: DEFAULT_HTTP_TIMEOUT_SECS,
        };
        let connector = ZohoMailConnector::new(config).expect("connector");

        let params = AuthorizeParams {
            tenant_id: Uuid::new_v4(),
            redirect_uri: Some("https://example.com/callback".to_string()),
            state: None,
        };

        let url = connector.authorize(params).await.expect("url");
        assert!(
            url.as_str()
                .starts_with("https://accounts.zoho.com/oauth/v2/auth"),
            "expected US accounts base"
        );
        assert!(
            url.as_str().contains("ZohoMail.messages.READ"),
            "expected ZohoMail scope in URL"
        );
    }

    #[tokio::test]
    async fn sync_without_cursor_sets_baseline() {
        let config = ZohoMailConfig {
            client_id: "id".to_string(),
            client_secret: "secret".to_string(),
            dc: ZohoDataCenter::Us,
            scopes: vec![DEFAULT_ZOHO_MAIL_SCOPE.to_string()],
            dedupe_window_secs: DEFAULT_DEDUPE_WINDOW_SECS,
            http_timeout_secs: DEFAULT_HTTP_TIMEOUT_SECS,
        };
        let connector = ZohoMailConnector::new(config).expect("connector");

        // Minimal stub connection; fields not used by skeleton sync.
        let connection = Connection {
            id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            provider_slug: ZOHO_MAIL_PROVIDER_SLUG.to_string(),
            external_id: "stub".to_string(),
            status: "active".to_string(),
            display_name: None,
            access_token_ciphertext: None,
            refresh_token_ciphertext: None,
            expires_at: None,
            scopes: None,
            metadata: None,
            created_at: chrono::Utc::now().into(),
            updated_at: chrono::Utc::now().into(),
        };

        let result = connector
            .sync(SyncParams {
                connection,
                cursor: None,
            })
            .await
            .expect("sync result");

        assert!(result.next_cursor.is_some());
        assert!(result.signals.is_empty());
        assert!(!result.has_more);
    }

    #[test]
    fn test_cursor_advancement_logic() {
        let now = Utc::now();
        let earlier = now - chrono::Duration::minutes(5);
        let later = now + chrono::Duration::minutes(5);

        // Test building cursor from timestamp
        let cursor_earlier = ZohoMailConnector::build_cursor_from_ts(earlier);
        let cursor_later = ZohoMailConnector::build_cursor_from_ts(later);

        // Cursors should be RFC3339 strings
        assert!(!cursor_earlier.as_str().unwrap().is_empty());
        assert!(!cursor_later.as_str().unwrap().is_empty());

        // Later timestamp should produce different cursor
        assert_ne!(cursor_earlier.as_str(), cursor_later.as_str());
    }

    #[test]
    fn test_dedupe_key_generation() {
        let message_id = "msg_123";
        let timestamp = "2025-01-01T12:00:00Z";

        let key1 = ZohoMailConnector::build_dedupe_key(message_id, timestamp);
        let key2 = ZohoMailConnector::build_dedupe_key(message_id, timestamp);
        let key3 = ZohoMailConnector::build_dedupe_key("msg_456", timestamp);
        let key4 = ZohoMailConnector::build_dedupe_key(message_id, "2025-01-01T13:00:00Z");

        // Same inputs should produce same key
        assert_eq!(key1, key2);

        // Different message_id should produce different key
        assert_ne!(key1, key3);

        // Different timestamp should produce different key
        assert_ne!(key1, key4);

        // Keys should contain both message_id and timestamp
        assert!(key1.contains(message_id));
        assert!(key1.contains(timestamp));
    }

    #[test]
    fn test_signal_payload_structure() {
        let config = ZohoMailConfig {
            client_id: "id".to_string(),
            client_secret: "secret".to_string(),
            dc: ZohoDataCenter::Us,
            scopes: vec![DEFAULT_ZOHO_MAIL_SCOPE.to_string()],
            dedupe_window_secs: DEFAULT_DEDUPE_WINDOW_SECS,
            http_timeout_secs: DEFAULT_HTTP_TIMEOUT_SECS,
        };
        let connector = ZohoMailConnector::new(config).expect("connector");

        let payload = connector.build_signal_payload(
            "email_received",
            "msg_123",
            Utc::now(),
        );

        // Check required fields
        assert_eq!(payload.get("provider").unwrap(), &serde_json::Value::String("zoho-mail".to_string()));
        assert_eq!(payload.get("kind").unwrap(), &serde_json::Value::String("email_received".to_string()));
        assert_eq!(payload.get("message_id").unwrap(), &serde_json::Value::String("msg_123".to_string()));

        // Check that occurred_at is a valid RFC3339 timestamp
        let occurred_at = payload.get("occurred_at").unwrap().as_str().unwrap();
        assert!(chrono::DateTime::parse_from_rfc3339(occurred_at).is_ok());
    }
}
