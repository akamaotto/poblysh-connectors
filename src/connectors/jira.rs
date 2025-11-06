//! Jira connector implementation
//!
//! Minimal Jira connector satisfying the Connector trait with realistic
//! OAuth authorize URL, webhook filtering, and incremental sync stubs.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::StatusCode;
use std::sync::Arc;
use tracing::{debug, info};
use url::Url;

use uuid::Uuid;

use crate::connectors::{
    AuthType, Connector, Cursor, ProviderMetadata, Registry,
    trait_::{AuthorizeParams, ExchangeTokenParams, SyncParams, SyncResult, WebhookParams},
};
use crate::models::{connection::Model as Connection, signal::Model as Signal};

/// Jira connector
pub struct JiraConnector {
    client_id: String,
    oauth_base: String,
}

impl JiraConnector {
    /// Create a new Jira connector with configuration
    pub fn new(client_id: String, oauth_base: String) -> Self {
        Self {
            client_id,
            oauth_base,
        }
    }
}

const JIRA_AUDIENCE: &str = "api.atlassian.com";

#[async_trait]
impl Connector for JiraConnector {
    async fn authorize(
        &self,
        params: AuthorizeParams,
    ) -> Result<Url, Box<dyn std::error::Error + Send + Sync>> {
        info!(
            tenant_id = %params.tenant_id,
            "Generating Jira OAuth authorization URL"
        );

        // Build Atlassian authorize URL with standard params.
        let mut url = Url::parse(&format!("{}/authorize", self.oauth_base))?;
        url.query_pairs_mut()
            .append_pair("audience", JIRA_AUDIENCE)
            .append_pair("client_id", &self.client_id)
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
            .append_pair("access_type", "offline")
            .append_pair("scope", "read:jira-work read:jira-user offline_access");

        debug!(
            tenant_id = %params.tenant_id,
            authorize_url = %url,
            "Generated Jira OAuth authorization URL"
        );

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
        info!(
            tenant_id = %params.connection.tenant_id,
            connection_id = %params.connection.id,
            has_cursor = %params.cursor.is_some(),
            "Starting Jira incremental sync"
        );

        // Extract access token (MVP stores token bytes as plaintext)
        let access_token = params
            .connection
            .access_token_ciphertext
            .clone()
            .ok_or_else(|| crate::connectors::trait_::SyncError::unauthorized("Missing access token"))
            .map(|bytes| String::from_utf8_lossy(&bytes).to_string())?;

        // Determine API base and resource (cloud/site) from connection metadata or discovery
        let api_base = std::env::var("JIRA_API_BASE")
            .or_else(|_| std::env::var("POBLYSH_JIRA_API_BASE"))
            .unwrap_or_else(|_| "https://api.atlassian.com".to_string());

        #[derive(Debug, serde::Deserialize)]
        struct AccessibleResource {
            id: String,
            url: Option<String>,
            #[allow(dead_code)]
            name: Option<String>,
            scopes: Option<Vec<String>>,
        }

        // Try to extract cloud_id/site URL from metadata if present
        let (cloud_id_opt, site_url_opt) = params
            .connection
            .metadata
            .as_ref()
            .and_then(|m| m.as_object())
            .map(|m| {
                let cid = m.get("cloud_id").and_then(|v| v.as_str()).map(|s| s.to_string());
                let site = m.get("site_url").and_then(|v| v.as_str()).map(|s| s.to_string());
                (cid, site)
            })
            .unwrap_or((None, None));

        let client = reqwest::Client::new();

        // Discover accessible resources if needed
        let (cloud_id, site_url, use_ex_api) = if cloud_id_opt.is_some() || site_url_opt.is_some() {
            (
                cloud_id_opt.clone().unwrap_or_default(),
                site_url_opt.clone().unwrap_or_default(),
                cloud_id_opt.is_some(),
            )
        } else {
            let url = format!("{}/oauth/token/accessible-resources", api_base.trim_end_matches('/'));
            let resp = client
                .get(&url)
                .bearer_auth(&access_token)
                .send()
                .await?;

            match resp.status() {
                StatusCode::OK => {
                    let resources: Vec<AccessibleResource> = resp.json().await?;
                    // Pick first Jira site with read:jira-work scope
                    let chosen = resources
                        .iter()
                        .find(|r| r.scopes.as_ref().is_some_and(|s| s.iter().any(|sc| sc.contains("read:jira-work"))))
                        .or_else(|| resources.first())
                        .ok_or_else(|| crate::connectors::trait_::SyncError::permanent("No accessible Jira resources"))?;
                    let cid = chosen.id.clone();
                    let site = chosen.url.clone().unwrap_or_default();
                    (cid, site, true)
                }
                StatusCode::UNAUTHORIZED => {
                    return Err(crate::connectors::trait_::SyncError::unauthorized("Invalid Jira token").into());
                }
                StatusCode::TOO_MANY_REQUESTS => {
                    let retry_after = resp
                        .headers()
                        .get("Retry-After")
                        .and_then(|h| h.to_str().ok())
                        .and_then(|s| s.parse::<u64>().ok());
                    return Err(crate::connectors::trait_::SyncError::rate_limited(retry_after).into());
                }
                status if status.is_server_error() => {
                    return Err(crate::connectors::trait_::SyncError::transient(format!(
                        "Jira resource discovery failed: {}",
                        status
                    ))
                    .into());
                }
                status => {
                    return Err(crate::connectors::trait_::SyncError::permanent(format!(
                        "Jira resource discovery failed: {}",
                        status
                    ))
                    .into());
                }
            }
        };

        // Compute since filter from cursor
        let since_rfc3339: String = if let Some(cursor) = &params.cursor {
            // Accept cursor as JSON string or number (unix seconds)
            let v = cursor.as_json();
            if let Some(s) = v.as_str() {
                s.to_string()
            } else if let Some(n) = v.as_i64() {
                DateTime::<Utc>::from_timestamp(n, 0)
                    .unwrap_or_else(|| Utc::now() - chrono::Duration::hours(1))
                    .to_rfc3339()
            } else {
                Utc::now().to_rfc3339()
            }
        } else {
            // Default to 1 hour back to limit first scan
            (Utc::now() - chrono::Duration::hours(1)).to_rfc3339()
        };

        // Build base search URL
        let base_search = if use_ex_api && !cloud_id.is_empty() {
            format!(
                "{}/ex/jira/{}/rest/api/3/search",
                api_base.trim_end_matches('/'),
                cloud_id
            )
        } else if !site_url.is_empty() {
            format!("{}/rest/api/3/search", site_url.trim_end_matches('/'))
        } else {
            // Fallback to ex/jira without cloud id will fail; surface a permanent error
            return Err(crate::connectors::trait_::SyncError::permanent(
                "Missing Jira cloud_id or site_url for search",
            )
            .into());
        };

        // Pagination parameters
        let max_results = 50u32;
        let mut start_at = 0u32;
        let mut all_signals: Vec<Signal> = Vec::new();
        let mut last_updated: Option<DateTime<Utc>> = None;
        let now = DateTime::from(Utc::now());

        loop {
            // JQL: updated >= since ordered ascending
            let jql = format!("updated >= \"{}\" ORDER BY updated ASC", since_rfc3339);
            let url = reqwest::Url::parse_with_params(
                &base_search,
                &[
                    ("jql", jql.as_str()),
                    ("startAt", &start_at.to_string()),
                    ("maxResults", &max_results.to_string()),
                    ("fields", "id,key,project,summary,status,assignee,updated"),
                ],
            )?;

            let resp = client
                .get(url)
                .bearer_auth(&access_token)
                .header("Accept", "application/json")
                .send()
                .await?;

            if resp.status() == StatusCode::UNAUTHORIZED {
                return Err(crate::connectors::trait_::SyncError::unauthorized("Jira token unauthorized").into());
            }
            if resp.status() == StatusCode::TOO_MANY_REQUESTS {
                let retry_after = resp
                    .headers()
                    .get("Retry-After")
                    .and_then(|h| h.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok());
                return Err(crate::connectors::trait_::SyncError::rate_limited(retry_after).into());
            }
            if resp.status().is_server_error() {
                return Err(crate::connectors::trait_::SyncError::transient(format!(
                    "Jira search failed: {}",
                    resp.status()
                ))
                .into());
            }
            if !resp.status().is_success() {
                return Err(crate::connectors::trait_::SyncError::permanent(format!(
                    "Jira search failed: {}",
                    resp.status()
                ))
                .into());
            }

            let body: serde_json::Value = resp.json().await?;
            let issues = body.get("issues").and_then(|v| v.as_array()).cloned().unwrap_or_default();

            // Map to Signals
            for issue in &issues {
                let fields = issue.get("fields").unwrap_or(&serde_json::Value::Null);
                let updated_str = if let Some(s) = fields.get("updated").and_then(|v| v.as_str()) {
                    s.to_string()
                } else {
                    now.to_rfc3339()
                };
                let updated_dt = DateTime::parse_from_rfc3339(&updated_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());
                last_updated = Some(last_updated.map_or(updated_dt, |prev| prev.max(updated_dt)));

                // Build a minimal webhook-like payload so existing normalizer works
                let payload = serde_json::json!({
                    "webhookEvent": "jira:issue_updated",
                    "issue": issue,
                });

                let normalized = extract_normalized_fields(&payload);
                let dedupe = generate_dedupe_key(&payload, "issue_updated");

                all_signals.push(Signal {
                    id: Uuid::new_v4(),
                    tenant_id: params.connection.tenant_id,
                    provider_slug: "jira".to_string(),
                    connection_id: params.connection.id,
                    kind: "issue_updated".to_string(),
                    occurred_at: updated_dt.into(),
                    received_at: now,
                    payload: normalized,
                    dedupe_key: Some(dedupe),
                    created_at: now,
                    updated_at: now,
                });
            }

            // Pagination advancement
            let fetched = issues.len() as u32;
            if fetched < max_results {
                break;
            }
            start_at += max_results;

            // Safety limit to avoid runaway loops
            if all_signals.len() >= 1000 {
                break;
            }
        }

        // Compute next cursor as greatest updated timestamp processed
        let next_cursor = last_updated.map(|dt| Cursor::from_string(dt.to_rfc3339()));
        let has_more = false; // We consumed all pages for this window

        let result = SyncResult {
            signals: all_signals,
            next_cursor,
            has_more,
        };

        debug!(
            tenant_id = %params.connection.tenant_id,
            connection_id = %params.connection.id,
            signals_generated = %result.signals.len(),
            has_more = %result.has_more,
            "Jira incremental sync completed"
        );

        Ok(result)
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

        debug!(
            tenant_id = %params.tenant_id,
            event_type = %event_type,
            "Processing Jira webhook"
        );

        let kind = match event_type {
            // Typical Jira webhook event keys
            "jira:issue_created" => Some("issue_created"),
            "jira:issue_updated" => Some("issue_updated"),
            _ => None,
        };

        if let Some(kind) = kind {
            info!(
                tenant_id = %params.tenant_id,
                event_type = %event_type,
                signal_kind = %kind,
                "Jira webhook mapped to signal"
            );

            // Extract normalized fields from Jira webhook payload
            let normalized_payload = extract_normalized_fields(&params.payload);

            Ok(vec![Signal {
                id: Uuid::new_v4(),
                tenant_id: params.tenant_id,
                provider_slug: "jira".to_string(),
                connection_id: Uuid::new_v4(),
                kind: kind.to_string(),
                occurred_at: now,
                received_at: now,
                payload: normalized_payload,
                dedupe_key: Some(generate_dedupe_key(&params.payload, kind)),
                created_at: now,
                updated_at: now,
            }])
        } else {
            debug!(
                tenant_id = %params.tenant_id,
                event_type = %event_type,
                "Jira webhook event ignored (not an issue event)"
            );
            Ok(vec![])
        }
    }
}

/// Initialize the Jira connector in the registry
pub fn register_jira_connector(registry: &mut Registry, connector: Arc<JiraConnector>) {
    let metadata = ProviderMetadata::new(
        "jira".to_string(),
        AuthType::OAuth2,
        vec!["read:jira-work".to_string(), "read:jira-user".to_string()],
        true, // webhooks supported
    );

    registry.register(connector, metadata);
}

/// Extract normalized fields from Jira webhook payload
fn extract_normalized_fields(payload: &serde_json::Value) -> serde_json::Value {
    let issue = payload.get("issue").unwrap_or(&serde_json::Value::Null);

    // Extract issue details
    let issue_id = issue.get("id").and_then(|v| v.as_str()).unwrap_or("");
    let issue_key = issue.get("key").and_then(|v| v.as_str()).unwrap_or("");

    // Extract project details
    let project_key = issue
        .get("fields")
        .and_then(|f| f.get("project"))
        .and_then(|p| p.get("key"))
        .and_then(|k| k.as_str())
        .unwrap_or("");

    // Extract issue fields
    let fields = issue.get("fields").unwrap_or(&serde_json::Value::Null);
    let summary = fields.get("summary").and_then(|s| s.as_str()).unwrap_or("");
    let status = fields
        .get("status")
        .and_then(|s| s.get("name"))
        .and_then(|n| n.as_str())
        .unwrap_or("");
    let assignee = fields
        .get("assignee")
        .and_then(|a| a.get("displayName"))
        .and_then(|n| n.as_str())
        .unwrap_or("");

    // Construct browser-friendly issue URL (browse link) instead of API self URL
    let base_url = issue
        .get("self")
        .and_then(|u| u.as_str())
        .and_then(|s| s.split("/rest/").next()) // Extract base URL before /rest/
        .unwrap_or("https://atlassian.net");

    let browse_url = if !issue_key.is_empty() {
        format!("{}/browse/{}", base_url.trim_end_matches("/"), issue_key)
    } else {
        format!("{}/browse/{}", base_url.trim_end_matches("/"), issue_id)
    };

    // Get occurred_at from webhook timestamp or current time
    let occurred_at = payload
        .get("timestamp")
        .and_then(|t| t.as_i64())
        .and_then(DateTime::from_timestamp_millis)
        .map(|dt| dt.to_rfc3339())
        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

    serde_json::json!({
        "issue_id": issue_id,
        "issue_key": issue_key,
        "project_key": project_key,
        "summary": summary,
        "status": status,
        "assignee": assignee,
        "url": browse_url,
        "occurred_at": occurred_at,
        "original_payload": payload
    })
}

/// Generate dedupe key for Jira webhook/sync signals
fn generate_dedupe_key(payload: &serde_json::Value, signal_kind: &str) -> String {
    let issue = payload.get("issue").unwrap_or(&serde_json::Value::Null);

    // Use issue ID and updated timestamp for deduplication
    let issue_id = issue.get("id").and_then(|v| v.as_str()).unwrap_or("");

    let updated = issue
        .get("fields")
        .and_then(|f| f.get("updated"))
        .and_then(|u| u.as_str())
        .unwrap_or("");

    format!("jira:{}:{}:{}", signal_kind, issue_id, updated)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connectors::trait_::{
        AuthorizeParams, ExchangeTokenParams, SyncParams, WebhookParams,
    };
    use uuid::Uuid;

    #[tokio::test]
    async fn test_jira_authorize_url_shape() {
        let connector = JiraConnector::new(
            "test-client-id".to_string(),
            "https://auth.atlassian.com".to_string(),
        );
        let tenant_id = Uuid::new_v4();

        let params = AuthorizeParams {
            tenant_id,
            redirect_uri: Some("https://example.com/callback".to_string()),
            state: Some("test_state_123".to_string()),
        };

        let result = connector.authorize(params).await.unwrap();

        // Verify URL structure
        assert_eq!(result.scheme(), "https");
        assert_eq!(result.host_str().unwrap(), "auth.atlassian.com");
        assert_eq!(result.path(), "/authorize");

        // Verify required query parameters
        let query_pairs: std::collections::HashMap<_, _> = result.query_pairs().collect();
        assert_eq!(query_pairs.get("audience").unwrap(), "api.atlassian.com");
        assert_eq!(query_pairs.get("client_id").unwrap(), "test-client-id");
        assert_eq!(
            query_pairs.get("redirect_uri").unwrap(),
            "https://example.com/callback"
        );
        assert_eq!(query_pairs.get("state").unwrap(), "test_state_123");
        assert_eq!(query_pairs.get("response_type").unwrap(), "code");
        assert_eq!(query_pairs.get("prompt").unwrap(), "consent");
        assert_eq!(query_pairs.get("access_type").unwrap(), "offline");
        assert!(query_pairs.get("scope").unwrap().contains("read:jira-work"));
        assert!(query_pairs.get("scope").unwrap().contains("read:jira-user"));
        assert!(query_pairs.get("scope").unwrap().contains("offline_access"));

        // Verify no fragment (OAuth 2.0 requirement)
        assert!(result.fragment().is_none());
    }

    #[tokio::test]
    async fn test_jira_webhook_mapping() {
        let connector = JiraConnector::new(
            "test-client-id".to_string(),
            "https://auth.atlassian.com".to_string(),
        );
        let tenant_id = Uuid::new_v4();

        // Test issue_created event
        let issue_created_payload = serde_json::json!({
            "webhookEvent": "jira:issue_created",
            "issue": {
                "id": "1001",
                "key": "TEST-123",
                "fields": {
                    "summary": "Test issue"
                }
            }
        });

        let params = WebhookParams {
            tenant_id,
            payload: issue_created_payload,
            db: None,
        };

        let signals = connector.handle_webhook(params).await.unwrap();
        assert_eq!(signals.len(), 1);
        assert_eq!(signals[0].kind, "issue_created");
        assert_eq!(signals[0].provider_slug, "jira");
        assert_eq!(signals[0].tenant_id, tenant_id);

        // Test issue_updated event
        let issue_updated_payload = serde_json::json!({
            "webhookEvent": "jira:issue_updated",
            "issue": {
                "id": "1001",
                "key": "TEST-123"
            }
        });

        let params = WebhookParams {
            tenant_id,
            payload: issue_updated_payload,
            db: None,
        };

        let signals = connector.handle_webhook(params).await.unwrap();
        assert_eq!(signals.len(), 1);
        assert_eq!(signals[0].kind, "issue_updated");

        // Test non-issue event (should be ignored)
        let non_issue_payload = serde_json::json!({
            "webhookEvent": "jira:project_created",
            "project": {
                "id": "1001",
                "key": "TEST"
            }
        });

        let params = WebhookParams {
            tenant_id,
            payload: non_issue_payload,
            db: None,
        };

        let signals = connector.handle_webhook(params).await.unwrap();
        assert_eq!(signals.len(), 0);

        // Test missing webhookEvent (should be ignored)
        let missing_event_payload = serde_json::json!({
            "issue": {
                "id": "1001"
            }
        });

        let params = WebhookParams {
            tenant_id,
            payload: missing_event_payload,
            db: None,
        };

        let signals = connector.handle_webhook(params).await.unwrap();
        assert_eq!(signals.len(), 0);
    }

    #[tokio::test]
    async fn test_jira_sync_with_cursor() {
        let connector = JiraConnector::new(
            "test-client-id".to_string(),
            "https://auth.atlassian.com".to_string(),
        );
        let tenant_id = Uuid::new_v4();
        let connection_id = Uuid::new_v4();

        // Create mock connection
        let connection = Connection {
            id: connection_id,
            tenant_id,
            provider_slug: "jira".to_string(),
            external_id: "jira-user-123".to_string(),
            status: "active".to_string(),
            display_name: Some("Jira Connection".to_string()),
            access_token_ciphertext: Some(b"mock_token".to_vec()),
            refresh_token_ciphertext: None,
            expires_at: None,
            scopes: None,
            metadata: None,
            created_at: chrono::Utc::now().fixed_offset(),
            updated_at: chrono::Utc::now().fixed_offset(),
        };

        // Test sync without cursor
        let params = SyncParams {
            connection: connection.clone(),
            cursor: None,
        };

        let result = connector.sync(params).await.unwrap();
        assert_eq!(result.signals.len(), 1);
        assert_eq!(result.signals[0].kind, "issue_updated");
        assert!(result.next_cursor.is_some());

        // Test sync with cursor
        let cursor = Cursor::from_string("1234567890".to_string());
        let params = SyncParams {
            connection,
            cursor: Some(cursor),
        };

        let result = connector.sync(params).await.unwrap();
        assert_eq!(result.signals.len(), 1);

        // Verify normalized fields are present in signal payload
        let payload = &result.signals[0].payload;
        assert!(payload.get("issue_id").is_some());
        assert!(payload.get("issue_key").is_some());
        assert!(payload.get("project_key").is_some());
        assert!(payload.get("summary").is_some());
        assert!(payload.get("status").is_some());
        assert!(payload.get("occurred_at").is_some());
    }

    #[tokio::test]
    async fn test_jira_exchange_token_stub() {
        let connector = JiraConnector::new(
            "test-client-id".to_string(),
            "https://auth.atlassian.com".to_string(),
        );
        let tenant_id = Uuid::new_v4();

        let params = ExchangeTokenParams {
            code: "test_authorization_code".to_string(),
            redirect_uri: Some("https://example.com/callback".to_string()),
            tenant_id,
        };

        let connection = connector.exchange_token(params).await.unwrap();

        assert_eq!(connection.provider_slug, "jira");
        assert_eq!(connection.tenant_id, tenant_id);
        assert_eq!(connection.status, "active");
        assert!(connection.access_token_ciphertext.is_some());
        assert!(connection.refresh_token_ciphertext.is_some());
        assert!(connection.expires_at.is_some());
        assert!(connection.scopes.is_some());
    }

    #[tokio::test]
    async fn test_jira_refresh_token_stub() {
        let connector = JiraConnector::new(
            "test-client-id".to_string(),
            "https://auth.atlassian.com".to_string(),
        );
        let tenant_id = Uuid::new_v4();

        let connection = Connection {
            id: Uuid::new_v4(),
            tenant_id,
            provider_slug: "jira".to_string(),
            external_id: "jira-user-123".to_string(),
            status: "active".to_string(),
            display_name: Some("Jira Connection".to_string()),
            access_token_ciphertext: Some(b"old_token".to_vec()),
            refresh_token_ciphertext: Some(b"old_refresh".to_vec()),
            expires_at: Some(chrono::Utc::now().fixed_offset()),
            scopes: None,
            metadata: None,
            created_at: chrono::Utc::now().fixed_offset(),
            updated_at: chrono::Utc::now().fixed_offset(),
        };

        let refreshed = connector.refresh_token(connection).await.unwrap();

        assert_eq!(refreshed.provider_slug, "jira");
        assert_eq!(refreshed.tenant_id, tenant_id);
        assert!(refreshed.access_token_ciphertext.is_some());
        assert!(refreshed.refresh_token_ciphertext.is_some());
        assert!(refreshed.expires_at.is_some());

        // Verify tokens were actually updated (new token should be different)
        let old_token = b"old_token".to_vec();
        let new_token = refreshed.access_token_ciphertext.unwrap();
        assert_ne!(old_token, new_token);
    }
}
