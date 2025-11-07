//! Gmail connector
//!
//! Implements the Connector trait for Gmail integration with Pub/Sub push notifications
//! and incremental sync using Gmail History API.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use base64::{Engine as _, engine::general_purpose};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use lru::LruCache;
use reqwest::Client;
use serde::Deserialize;
use thiserror::Error;
use tokio::sync::RwLock;
use url::Url;
use uuid::Uuid;

use crate::connectors::{
    AuthorizeParams, Connector, ExchangeTokenParams, Registry, SyncParams, SyncResult,
    WebhookParams,
    metadata::{AuthType, ProviderMetadata},
};
use crate::models::{connection::Model as Connection, signal::Model as Signal};

/// Default Gmail OAuth scopes
pub const DEFAULT_GMAIL_SCOPES: &[&str] = &["https://www.googleapis.com/auth/gmail.readonly"];

/// Gmail OAuth endpoints
const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_USERINFO_URL: &str = "https://www.googleapis.com/oauth2/v2/userinfo";

/// Gmail API endpoints
const GMAIL_USERS_ENDPOINT: &str = "https://gmail.googleapis.com/gmail/v1/users";

/// Gmail connector errors
#[derive(Debug, Error)]
pub enum GmailError {
    #[error("Invalid Pub/Sub message format: {0}")]
    InvalidMessageFormat(String),

    #[error("History API error: {0}")]
    HistoryApiError(String),

    #[error("Rate limit exceeded: retry after {0}s")]
    RateLimitExceeded(u64),

    #[error("Authentication failed: {0}")]
    Authentication(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Userinfo error: {0}")]
    UserinfoError(String),

    #[error("Token exchange failed: {0}")]
    TokenExchange(String),

    #[error("Token refresh failed: {0}")]
    TokenRefresh(String),

    #[error("OIDC verification failed: {0}")]
    OidcVerification(String),

    #[error("JWKS fetch failed: {0}")]
    JwksFetch(String),
}

/// Google OAuth token response
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GoogleTokenResponse {
    access_token: String,
    token_type: String,
    expires_in: Option<u64>,
    refresh_token: Option<String>,
    scope: Option<String>,
}

/// Google userinfo response
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GoogleUserinfo {
    email: String,
    name: Option<String>,
    picture: Option<String>,
}

/// Pub/Sub message envelope
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct PubSubMessage {
    #[serde(rename = "messageId")]
    message_id: String,
    data: String, // base64 encoded
    attributes: Option<HashMap<String, String>>,
    #[serde(rename = "publishTime")]
    publish_time: String,
}

/// Pub/Sub webhook payload
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct PubSubPayload {
    message: PubSubMessage,
    subscription: String,
}

/// Gmail push notification data (decoded from PubSubMessage.data)
#[derive(Debug, Deserialize)]
struct GmailPushData {
    #[serde(rename = "emailAddress")]
    email_address: String,
    #[serde(rename = "historyId")]
    history_id: u64,
}

/// Gmail history response
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GmailHistoryResponse {
    #[serde(rename = "historyId")]
    history_id: u64,
    history: Option<Vec<GmailHistoryRecord>>,
    #[serde(rename = "nextPageToken")]
    next_page_token: Option<String>,
}

/// Gmail history record
#[derive(Debug, Deserialize)]
struct GmailHistoryRecord {
    id: String,
    #[serde(rename = "historyId")]
    history_id: u64,
    messages: Option<Vec<GmailMessage>>,
    #[serde(rename = "messagesAdded")]
    messages_added: Option<Vec<GmailMessage>>,
    #[serde(rename = "messagesDeleted")]
    messages_deleted: Option<Vec<GmailMessage>>,
}

/// Gmail message
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GmailMessage {
    id: String,
    #[serde(rename = "threadId")]
    thread_id: Option<String>,
    #[serde(rename = "historyId")]
    history_id: Option<String>,
    #[serde(rename = "internalDate")]
    internal_date: Option<String>,
    #[serde(rename = "sizeEstimate")]
    size_estimate: Option<u64>,
    snippet: Option<String>,
    #[serde(rename = "labelIds")]
    label_ids: Vec<String>,
}

/// JWKS (JSON Web Key Set) response from Google
#[derive(Debug, Deserialize)]
struct JwksResponse {
    keys: Vec<JsonWebKey>,
}

/// JSON Web Key for JWT signature verification
#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
struct JsonWebKey {
    kty: String,
    kid: Option<String>,
    alg: Option<String>,
    n: Option<String>,
    e: Option<String>,
    r#use: Option<String>,
}

/// OIDC token verifier with caching
struct OidcVerifier {
    http_client: Client,
    jwks_cache: Arc<RwLock<LruCache<String, JsonWebKey>>>,
    audience: String,
    issuers: Vec<String>,
}

impl OidcVerifier {
    /// Create a new OIDC verifier
    fn new(http_client: Client, audience: String, issuers: Vec<String>) -> Self {
        Self {
            http_client,
            jwks_cache: Arc::new(RwLock::new(LruCache::new(
                std::num::NonZero::new(100).unwrap(),
            ))), // Cache 100 keys
            audience,
            issuers,
        }
    }

    /// Verify JWT from Authorization header
    async fn verify_jwt(&self, auth_header: &str) -> Result<(), GmailError> {
        // Extract Bearer token
        let token = auth_header.strip_prefix("Bearer ").ok_or_else(|| {
            GmailError::OidcVerification("Invalid Authorization header format".to_string())
        })?;

        // Decode token header to get kid
        let header = jsonwebtoken::decode_header(token).map_err(|e| {
            GmailError::OidcVerification(format!("Failed to decode JWT header: {}", e))
        })?;

        let kid = header.kid.ok_or_else(|| {
            GmailError::OidcVerification("JWT missing 'kid' header parameter".to_string())
        })?;

        // Get the verification key
        let jwk = self.get_verification_key(&kid).await?;

        // Create decoding key from JWK
        let decoding_key = self.create_decoding_key(&jwk)?;

        // Set up validation
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&[&self.audience]);
        validation.set_issuer(&self.issuers);
        validation.validate_exp = true;
        validation.leeway = 60; // Allow 60 seconds clock skew

        // Verify token
        decode::<serde_json::Value>(token, &decoding_key, &validation)
            .map_err(|e| GmailError::OidcVerification(format!("JWT verification failed: {}", e)))?;

        Ok(())
    }

    /// Get verification key by kid, fetching from JWKS if not cached
    async fn get_verification_key(&self, kid: &str) -> Result<JsonWebKey, GmailError> {
        // Check cache first
        {
            let mut cache = self.jwks_cache.write().await;
            if let Some(jwk) = cache.get(kid) {
                return Ok(jwk.clone());
            }
        }

        // Not in cache, fetch JWKS
        let jwks = self.fetch_jwks().await?;

        // Find the key with matching kid
        let jwk = jwks
            .keys
            .into_iter()
            .find(|key| key.kid.as_ref() == Some(&kid.to_string()))
            .ok_or_else(|| {
                GmailError::OidcVerification(format!("Key with kid '{}' not found in JWKS", kid))
            })?;

        // Cache the key
        {
            let mut cache = self.jwks_cache.write().await;
            cache.put(kid.to_string(), jwk.clone());
        }

        Ok(jwk)
    }

    /// Fetch JWKS from Google's OAuth2 endpoint
    async fn fetch_jwks(&self) -> Result<JwksResponse, GmailError> {
        const GOOGLE_JWKS_URL: &str = "https://www.googleapis.com/oauth2/v3/certs";

        let response = self
            .http_client
            .get(GOOGLE_JWKS_URL)
            .send()
            .await
            .map_err(|e| GmailError::JwksFetch(format!("Failed to fetch JWKS: {}", e)))?;

        if !response.status().is_success() {
            return Err(GmailError::JwksFetch(format!(
                "JWKS request failed with status {}: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            )));
        }

        response
            .json()
            .await
            .map_err(|e| GmailError::JwksFetch(format!("Failed to parse JWKS response: {}", e)))
    }

    /// Create DecodingKey from JsonWebKey
    fn create_decoding_key(&self, jwk: &JsonWebKey) -> Result<DecodingKey, GmailError> {
        if jwk.kty != "RSA" {
            return Err(GmailError::OidcVerification(
                "Only RSA keys are supported for OIDC verification".to_string(),
            ));
        }

        let n = jwk.n.as_ref().ok_or_else(|| {
            GmailError::OidcVerification("Missing 'n' (modulus) in JWK".to_string())
        })?;

        let e = jwk.e.as_ref().ok_or_else(|| {
            GmailError::OidcVerification("Missing 'e' (exponent) in JWK".to_string())
        })?;

        DecodingKey::from_rsa_components(n, e).map_err(|e| {
            GmailError::OidcVerification(format!("Failed to create decoding key: {}", e))
        })
    }
}

/// Gmail connector implementation
pub struct GmailConnector {
    /// OAuth configuration
    client_id: String,
    client_secret: String,
    /// OAuth scopes (configurable)
    scopes: Vec<String>,
    /// HTTP client for API calls
    http_client: Client,
    /// OIDC verifier for Pub/Sub webhooks
    oidc_verifier: Option<OidcVerifier>,
    /// Gmail History API base endpoint (overridable for tests)
    gmail_users_endpoint: String,
}

impl GmailConnector {
    fn build_http_client() -> Client {
        Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client")
    }

    fn new_with_options(
        client_id: String,
        client_secret: String,
        scopes: Vec<String>,
        oidc_audience: Option<String>,
        oidc_issuers: Option<Vec<String>>,
        gmail_users_endpoint: String,
    ) -> Self {
        let http_client = Self::build_http_client();

        // Create OIDC verifier if audience and issuers are provided
        let oidc_verifier = if let (Some(audience), Some(issuers)) = (oidc_audience, oidc_issuers) {
            Some(OidcVerifier::new(http_client.clone(), audience, issuers))
        } else {
            None
        };

        Self {
            client_id,
            client_secret,
            scopes,
            http_client,
            oidc_verifier,
            gmail_users_endpoint,
        }
    }

    /// Create a new Gmail connector
    pub fn new(client_id: String, client_secret: String) -> Self {
        Self::new_with_options(
            client_id,
            client_secret,
            DEFAULT_GMAIL_SCOPES.iter().map(|s| s.to_string()).collect(),
            None,
            None,
            GMAIL_USERS_ENDPOINT.to_string(),
        )
    }

    /// Create a new Gmail connector with OIDC verification and configurable scopes
    pub fn new_with_oidc(
        client_id: String,
        client_secret: String,
        oidc_audience: Option<String>,
        oidc_issuers: Option<Vec<String>>,
    ) -> Self {
        Self::new_with_oidc_and_scopes(
            client_id,
            client_secret,
            oidc_audience,
            oidc_issuers,
            DEFAULT_GMAIL_SCOPES.iter().map(|s| s.to_string()).collect(),
        )
    }

    /// Create a new Gmail connector with OIDC verification and custom scopes
    pub fn new_with_oidc_and_scopes(
        client_id: String,
        client_secret: String,
        oidc_audience: Option<String>,
        oidc_issuers: Option<Vec<String>>,
        scopes: Vec<String>,
    ) -> Self {
        Self::new_with_options(
            client_id,
            client_secret,
            scopes,
            oidc_audience,
            oidc_issuers,
            GMAIL_USERS_ENDPOINT.to_string(),
        )
    }

    #[cfg(test)]
    fn new_with_history_endpoint_for_tests(
        client_id: String,
        client_secret: String,
        gmail_users_endpoint: String,
    ) -> Self {
        Self::new_with_options(
            client_id,
            client_secret,
            DEFAULT_GMAIL_SCOPES.iter().map(|s| s.to_string()).collect(),
            None,
            None,
            gmail_users_endpoint,
        )
    }

    /// Get Gmail scopes as a space-separated string
    fn get_scopes(&self) -> String {
        self.scopes.join(" ")
    }

    /// Build Gmail OAuth authorization URL
    fn build_authorize_url(&self, params: &AuthorizeParams) -> Result<Url, GmailError> {
        let mut url = Url::parse(GOOGLE_AUTH_URL)
            .map_err(|e| GmailError::Configuration(format!("Invalid auth URL: {}", e)))?;

        url.query_pairs_mut()
            .append_pair("client_id", &self.client_id)
            .append_pair(
                "redirect_uri",
                params
                    .redirect_uri
                    .as_ref()
                    .unwrap_or(&"http://localhost:3000/callback".to_string()),
            )
            .append_pair("response_type", "code")
            .append_pair("scope", &self.get_scopes())
            .append_pair("access_type", "offline") // Important for refresh tokens
            .append_pair("prompt", "consent");

        // Add state if provided
        if let Some(state) = &params.state {
            url.query_pairs_mut().append_pair("state", state);
        }

        Ok(url)
    }

    /// Exchange authorization code for access token
    async fn exchange_code_for_token(
        &self,
        code: &str,
        redirect_uri: &str,
    ) -> Result<GoogleTokenResponse, GmailError> {
        let mut params = HashMap::new();
        params.insert("client_id".to_string(), self.client_id.clone());
        params.insert("client_secret".to_string(), self.client_secret.clone());
        params.insert("code".to_string(), code.to_string());
        params.insert("grant_type".to_string(), "authorization_code".to_string());
        params.insert("redirect_uri".to_string(), redirect_uri.to_string());

        let response = self
            .http_client
            .post(GOOGLE_TOKEN_URL)
            .header("Accept", "application/json")
            .form(&params)
            .send()
            .await
            .map_err(|e| GmailError::Network(format!("Token request failed: {}", e)))?;

        let status = response.status();

        if status == 429 {
            let retry_after = response
                .headers()
                .get("Retry-After")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse().ok())
                .unwrap_or(60);
            return Err(GmailError::RateLimitExceeded(retry_after));
        }

        if status == 403 {
            // Extract headers before consuming response
            let retry_after_header = response
                .headers()
                .get("Retry-After")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse().ok());

            // Check for quota-related errors in 403 responses
            let body = response.text().await.unwrap_or_default();
            let body_lower = body.to_lowercase();

            // Common quota error patterns
            let quota_error_patterns = [
                "userratelimitexceeded",
                "ratelimitexceeded",
                "quotaexceeded",
                "servicelimit",
                "daily limit",
                "billing limit",
            ];

            if quota_error_patterns
                .iter()
                .any(|pattern| body_lower.contains(pattern))
            {
                let retry_after = retry_after_header.unwrap_or(60);
                return Err(GmailError::RateLimitExceeded(retry_after));
            }

            return Err(GmailError::TokenExchange(format!(
                "Token exchange failed with status {}: {}",
                status, body
            )));
        }

        if !status.is_success() {
            return Err(GmailError::TokenExchange(format!(
                "Token exchange failed with status {}: {}",
                status,
                response.text().await.unwrap_or_default()
            )));
        }

        response.json::<GoogleTokenResponse>().await.map_err(|e| {
            GmailError::InvalidResponse(format!("Failed to parse token response: {}", e))
        })
    }

    /// Refresh access token
    async fn refresh_access_token(
        &self,
        refresh_token: &str,
    ) -> Result<GoogleTokenResponse, GmailError> {
        let mut params = HashMap::new();
        params.insert("client_id".to_string(), self.client_id.clone());
        params.insert("client_secret".to_string(), self.client_secret.clone());
        params.insert("refresh_token".to_string(), refresh_token.to_string());
        params.insert("grant_type".to_string(), "refresh_token".to_string());

        let response = self
            .http_client
            .post(GOOGLE_TOKEN_URL)
            .header("Accept", "application/json")
            .form(&params)
            .send()
            .await
            .map_err(|e| GmailError::Network(format!("Token refresh request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(GmailError::TokenRefresh(format!(
                "Token refresh failed with status {}: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            )));
        }

        response.json::<GoogleTokenResponse>().await.map_err(|e| {
            GmailError::InvalidResponse(format!("Failed to parse refresh response: {}", e))
        })
    }

    /// Get user email from Google userinfo endpoint
    async fn get_user_email(&self, access_token: &str) -> Result<String, GmailError> {
        let response = self
            .http_client
            .get(GOOGLE_USERINFO_URL)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await
            .map_err(|e| GmailError::Network(format!("Failed to fetch userinfo: {}", e)))?;

        if !response.status().is_success() {
            return Err(GmailError::UserinfoError(format!(
                "Userinfo request failed with status {}: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            )));
        }

        let userinfo: GoogleUserinfo = response.json().await.map_err(|e| {
            GmailError::InvalidResponse(format!("Failed to parse userinfo response: {}", e))
        })?;

        Ok(userinfo.email)
    }

    /// Verify OIDC token from Authorization header
    pub async fn verify_oidc_token(&self, auth_header: Option<&str>) -> Result<(), GmailError> {
        // OIDC verification is now mandatory for Gmail
        let verifier = self.oidc_verifier.as_ref().ok_or_else(|| {
            GmailError::Configuration("Gmail OIDC verification is not configured".to_string())
        })?;

        let auth_header = auth_header.ok_or_else(|| {
            GmailError::OidcVerification(
                "Missing Authorization header for OIDC verification".to_string(),
            )
        })?;

        verifier.verify_jwt(auth_header).await?;

        Ok(())
    }

    /// Decode and parse Pub/Sub webhook payload with idempotency
    fn parse_webhook_payload(
        &self,
        payload: &serde_json::Value,
    ) -> Result<(String, u64, String), GmailError> {
        let pubsub: PubSubPayload = serde_json::from_value(payload.clone()).map_err(|e| {
            GmailError::InvalidMessageFormat(format!("Invalid Pub/Sub payload: {}", e))
        })?;

        // Decode base64 message data
        let decoded_data = general_purpose::STANDARD
            .decode(pubsub.message.data)
            .map_err(|e| GmailError::InvalidMessageFormat(format!("Invalid base64 data: {}", e)))?;

        let decoded_str = String::from_utf8(decoded_data).map_err(|e| {
            GmailError::InvalidMessageFormat(format!("Invalid UTF-8 in data: {}", e))
        })?;

        let gmail_data: GmailPushData = serde_json::from_str(&decoded_str).map_err(|e| {
            GmailError::InvalidMessageFormat(format!("Invalid Gmail push data: {}", e))
        })?;

        Ok((
            gmail_data.email_address,
            gmail_data.history_id,
            pubsub.message.message_id,
        ))
    }

    /// Fetch Gmail history starting from a specific history ID
    async fn fetch_history(
        &self,
        access_token: &str,
        start_history_id: u64,
    ) -> Result<GmailHistoryResponse, GmailError> {
        let url = format!(
            "{}/me/history?startHistoryId={}",
            self.gmail_users_endpoint, start_history_id
        );

        let response = self
            .http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await
            .map_err(|e| GmailError::Network(format!("Failed to fetch history: {}", e)))?;

        let status = response.status();

        if status == 429 {
            let retry_after = response
                .headers()
                .get("Retry-After")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse().ok())
                .unwrap_or(60);
            return Err(GmailError::RateLimitExceeded(retry_after));
        }

        if status == 403 {
            // Extract headers before consuming response
            let retry_after_header = response
                .headers()
                .get("Retry-After")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse().ok());

            // Check for quota-related errors in 403 responses
            let body = response.text().await.unwrap_or_default();
            let body_lower = body.to_lowercase();

            // Common Gmail quota error reasons that should be treated as rate limits
            let quota_error_patterns = [
                "userratelimitexceeded",
                "ratelimitexceeded",
                "quotaexceeded",
                "servicelimit",
                "daily limit",
                "billing limit",
            ];

            if quota_error_patterns
                .iter()
                .any(|pattern| body_lower.contains(pattern))
            {
                let retry_after = retry_after_header.unwrap_or(60); // Default 60 seconds for quota errors
                return Err(GmailError::RateLimitExceeded(retry_after));
            }

            return Err(GmailError::HistoryApiError(format!(
                "Access forbidden: {}",
                body
            )));
        }

        if status == 401 {
            return Err(GmailError::Authentication(
                "Invalid or expired access token".to_string(),
            ));
        }

        if status == 404 {
            return Err(GmailError::HistoryApiError(
                "History ID not found or too old".to_string(),
            ));
        }

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(GmailError::HistoryApiError(format!(
                "History API request failed with status {}: {}",
                status, body
            )));
        }

        response.json().await.map_err(|e| {
            GmailError::InvalidResponse(format!("Failed to parse history response: {}", e))
        })
    }

    /// Process a Gmail history record and generate normalized email signals
    async fn process_history_record(
        &self,
        connection: &Connection,
        record: GmailHistoryRecord,
        _access_token: &str,
    ) -> Result<Vec<Signal>, GmailError> {
        let mut signals = Vec::new();

        let deleted = record
            .messages_deleted
            .as_ref()
            .map(|msgs| !msgs.is_empty())
            .unwrap_or(false);

        let signal_type = if deleted {
            "email_deleted"
        } else {
            "email_updated"
        };

        signals.push(self.create_email_signal_from_record(connection, signal_type, &record)?);

        Ok(signals)
    }

    /// Create a normalized email signal from Gmail history record
    fn create_email_signal_from_record(
        &self,
        connection: &Connection,
        signal_type: &str,
        record: &GmailHistoryRecord,
    ) -> Result<Signal, GmailError> {
        let dedupe_key = format!("gmail:{}:{}", connection.id, record.history_id);

        let mut payload = serde_json::Map::new();
        payload.insert(
            "signal_type".to_string(),
            serde_json::Value::String(signal_type.to_string()),
        );
        payload.insert(
            "history_id".to_string(),
            serde_json::Value::Number(serde_json::Number::from(record.history_id)),
        );
        payload.insert(
            "record_id".to_string(),
            serde_json::Value::String(record.id.clone()),
        );

        // Add message counts if available
        if let Some(ref messages) = record.messages {
            payload.insert(
                "total_messages".to_string(),
                serde_json::Value::Number(serde_json::Number::from(messages.len())),
            );
        }
        if let Some(ref messages_added) = record.messages_added {
            payload.insert(
                "messages_added".to_string(),
                serde_json::Value::Number(serde_json::Number::from(messages_added.len())),
            );
        }
        if let Some(ref messages_deleted) = record.messages_deleted {
            payload.insert(
                "messages_deleted".to_string(),
                serde_json::Value::Number(serde_json::Number::from(messages_deleted.len())),
            );
        }

        Ok(Signal {
            id: Uuid::new_v4(),
            tenant_id: connection.tenant_id,
            provider_slug: "gmail".to_string(),
            connection_id: connection.id,
            kind: signal_type.to_string(),
            occurred_at: chrono::Utc::now().into(),
            received_at: chrono::Utc::now().into(),
            payload: serde_json::to_value(payload).unwrap(),
            dedupe_key: Some(dedupe_key),
            created_at: chrono::Utc::now().into(),
            updated_at: chrono::Utc::now().into(),
        })
    }

    /// Check for duplicate Pub/Sub message delivery using message ID and history ID
    async fn is_duplicate_delivery(
        &self,
        db: &sea_orm::DatabaseConnection,
        connection_id: &uuid::Uuid,
        _message_id: &str,
        history_id: u64,
    ) -> Result<bool, GmailError> {
        use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

        // Check if we already have a signal with the same dedupe key
        // The dedupe key format is: gmail:{connection_id}:{history_id}
        let expected_dedupe_key = format!("gmail:{}:{}", connection_id, history_id);

        let existing_signal = crate::models::signal::Entity::find()
            .filter(crate::models::signal::Column::ProviderSlug.eq("gmail"))
            .filter(crate::models::signal::Column::ConnectionId.eq(*connection_id))
            .filter(crate::models::signal::Column::DedupeKey.eq(&expected_dedupe_key))
            .one(db)
            .await
            .map_err(|e| {
                GmailError::Network(format!("Failed to check for duplicate signals: {}", e))
            })?;

        Ok(existing_signal.is_some())
    }

    /// Create a normalized email signal from Gmail message with Pub/Sub message tracking
    fn create_email_signal_with_message_id(
        &self,
        connection: &Connection,
        signal_type: &str,
        email_address: &str,
        history_id: u64,
        message_id: &str,
    ) -> Signal {
        let dedupe_key = format!("gmail:{}:{}", connection.id, history_id);

        let mut payload = serde_json::Map::new();
        payload.insert(
            "signal_type".to_string(),
            serde_json::Value::String(signal_type.to_string()),
        );
        payload.insert(
            "email_address".to_string(),
            serde_json::Value::String(email_address.to_string()),
        );
        payload.insert(
            "history_id".to_string(),
            serde_json::Value::Number(serde_json::Number::from(history_id)),
        );
        payload.insert(
            "pubsub_message_id".to_string(),
            serde_json::Value::String(message_id.to_string()),
        );

        Signal {
            id: Uuid::new_v4(),
            tenant_id: connection.tenant_id,
            provider_slug: "gmail".to_string(),
            connection_id: connection.id,
            kind: signal_type.to_string(),
            occurred_at: chrono::Utc::now().into(),
            received_at: chrono::Utc::now().into(),
            payload: serde_json::to_value(payload).unwrap(),
            dedupe_key: Some(dedupe_key),
            created_at: chrono::Utc::now().into(),
            updated_at: chrono::Utc::now().into(),
        }
    }

    /// Create a normalized email signal from Gmail message
    #[allow(dead_code)]
    fn create_email_signal(
        &self,
        connection: &Connection,
        signal_type: &str,
        email_address: &str,
        history_id: u64,
    ) -> Signal {
        let dedupe_key = format!("gmail:{}:{}", connection.id, history_id);

        let mut payload = serde_json::Map::new();
        payload.insert(
            "signal_type".to_string(),
            serde_json::Value::String(signal_type.to_string()),
        );
        payload.insert(
            "email_address".to_string(),
            serde_json::Value::String(email_address.to_string()),
        );
        payload.insert(
            "history_id".to_string(),
            serde_json::Value::Number(serde_json::Number::from(history_id)),
        );

        Signal {
            id: Uuid::new_v4(),
            tenant_id: connection.tenant_id,
            provider_slug: "gmail".to_string(),
            connection_id: connection.id,
            kind: signal_type.to_string(),
            occurred_at: chrono::Utc::now().into(),
            received_at: chrono::Utc::now().into(),
            payload: serde_json::to_value(payload).unwrap(),
            dedupe_key: Some(dedupe_key),
            created_at: chrono::Utc::now().into(),
            updated_at: chrono::Utc::now().into(),
        }
    }
}

#[async_trait]
impl Connector for GmailConnector {
    /// Generate Gmail OAuth authorization URL
    async fn authorize(
        &self,
        params: AuthorizeParams,
    ) -> Result<Url, Box<dyn std::error::Error + Send + Sync>> {
        let url = self.build_authorize_url(&params)?;
        tracing::debug!("Generated Gmail OAuth authorize URL: {}", url);
        Ok(url)
    }

    /// Exchange OAuth code for access/refresh tokens
    async fn exchange_token(
        &self,
        params: ExchangeTokenParams,
    ) -> Result<Connection, Box<dyn std::error::Error + Send + Sync>> {
        // Exchange code for access token
        let redirect_uri = params.redirect_uri.as_ref().ok_or_else(|| {
            Box::new(GmailError::Configuration(
                "Redirect URI is required for OAuth token exchange".to_string(),
            )) as Box<dyn std::error::Error + Send + Sync>
        })?;

        let token_response = self
            .exchange_code_for_token(&params.code, redirect_uri)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        // Get user profile information to extract email address
        let email = self
            .get_user_email(&token_response.access_token)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        tracing::info!("Successfully authenticated Gmail user: {}", email);

        // Create connection record
        let connection = Connection {
            id: Uuid::new_v4(),
            tenant_id: params.tenant_id,
            provider_slug: "gmail".to_string(),
            external_id: email.to_string(), // Use email as external_id for webhook resolution
            status: "active".to_string(),
            display_name: Some(format!("Gmail ({})", email)),
            access_token_ciphertext: Some(token_response.access_token.into_bytes()),
            refresh_token_ciphertext: token_response.refresh_token.map(|t| t.into_bytes()),
            expires_at: token_response.expires_in.map(|seconds| {
                (chrono::Utc::now() + chrono::Duration::seconds(seconds as i64)).into()
            }),
            scopes: Some(serde_json::to_value(vec![self.get_scopes()]).unwrap()),
            metadata: Some(serde_json::json!({
                "token_type": token_response.token_type,
                "userinfo": {
                    "email": email,
                }
            })),
            created_at: chrono::Utc::now().into(),
            updated_at: chrono::Utc::now().into(),
        };

        Ok(connection)
    }

    /// Refresh expired access token
    async fn refresh_token(
        &self,
        mut connection: Connection,
    ) -> Result<Connection, Box<dyn std::error::Error + Send + Sync>> {
        let refresh_token_bytes =
            connection
                .refresh_token_ciphertext
                .as_ref()
                .ok_or_else(|| {
                    Box::new(GmailError::Authentication(
                        "No refresh token available".to_string(),
                    )) as Box<dyn std::error::Error + Send + Sync>
                })?;

        let refresh_token_str = String::from_utf8(refresh_token_bytes.clone()).map_err(|e| {
            Box::new(GmailError::Authentication(format!(
                "Invalid refresh token encoding: {}",
                e
            ))) as Box<dyn std::error::Error + Send + Sync>
        })?;

        let token_response = self
            .refresh_access_token(&refresh_token_str)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        tracing::info!(
            "Successfully refreshed Gmail token for connection: {}",
            connection.id
        );

        // Update connection with new tokens
        connection.access_token_ciphertext = Some(token_response.access_token.into_bytes());
        // Note: Google doesn't always return a new refresh token, so we only update if we get one
        if let Some(new_refresh_token) = token_response.refresh_token {
            connection.refresh_token_ciphertext = Some(new_refresh_token.into_bytes());
        }
        connection.expires_at = token_response
            .expires_in
            .map(|seconds| (chrono::Utc::now() + chrono::Duration::seconds(seconds as i64)).into());
        connection.updated_at = chrono::Utc::now().into();

        Ok(connection)
    }

    /// Perform incremental sync using Gmail History API
    async fn sync(
        &self,
        params: SyncParams,
    ) -> Result<SyncResult, Box<dyn std::error::Error + Send + Sync>> {
        let connection = params.connection;
        let start_cursor = params
            .cursor
            .and_then(|c| c.as_str().and_then(|s| s.parse().ok()));

        // Extract and decode access token
        let access_token_bytes = connection.access_token_ciphertext.as_ref().ok_or_else(|| {
            Box::new(GmailError::Authentication(
                "No access token available".to_string(),
            )) as Box<dyn std::error::Error + Send + Sync>
        })?;

        let access_token = String::from_utf8(access_token_bytes.clone()).map_err(|e| {
            Box::new(GmailError::Authentication(format!(
                "Invalid access token encoding: {}",
                e
            ))) as Box<dyn std::error::Error + Send + Sync>
        })?;

        // Start from cursor or fetch current history ID
        let mut current_history_id = start_cursor.unwrap_or({
            // Start from a very early history ID ( Gmail history IDs start from 1)
            1
        });

        let mut all_signals = Vec::new();
        let _has_more = false;

        // Fetch history in a single batch for now
        match self.fetch_history(&access_token, current_history_id).await {
            Ok(history_response) => {
                let history_records = history_response.history.unwrap_or_default();

                tracing::debug!(
                    "Gmail sync: fetched {} history records starting from ID {}",
                    history_records.len(),
                    current_history_id
                );

                for record in history_records {
                    let record_signals = self
                        .process_history_record(&connection, record, &access_token)
                        .await?;
                    all_signals.extend(record_signals);
                }

                // Update cursor to the latest history ID
                current_history_id = history_response.history_id;

                // Note: Gmail History API doesn't provide pagination in the same way,
                // so we consider sync complete for this batch
            }
            Err(GmailError::RateLimitExceeded(retry_after)) => {
                return Err(
                    Box::new(crate::connectors::trait_::SyncError::rate_limited(Some(
                        retry_after,
                    ))) as Box<dyn std::error::Error + Send + Sync>,
                );
            }
            Err(GmailError::Authentication(msg)) => {
                return Err(
                    Box::new(crate::connectors::trait_::SyncError::unauthorized(msg))
                        as Box<dyn std::error::Error + Send + Sync>,
                );
            }
            Err(GmailError::HistoryApiError(msg))
                if msg.contains("not found") || msg.contains("too old") =>
            {
                // History ID is invalid/expired, signal need for full re-sync
                tracing::warn!(
                    "History ID {} is invalid/expired for connection {}: {}",
                    current_history_id,
                    connection.id,
                    msg
                );
                return Err(
                    Box::new(crate::connectors::trait_::SyncError::transient(format!(
                        "History sync error: {}",
                        msg
                    ))) as Box<dyn std::error::Error + Send + Sync>,
                );
            }
            Err(e) => {
                tracing::error!(
                    "Error fetching Gmail history for connection {}: {}",
                    connection.id,
                    e
                );
                return Err(Box::new(e) as Box<dyn std::error::Error + Send + Sync>);
            }
        }

        Ok(SyncResult {
            signals: all_signals,
            next_cursor: Some(crate::connectors::Cursor::from_string(
                current_history_id.to_string(),
            )),
            has_more: false,
        })
    }

    /// Handle Pub/Sub webhook for Gmail push notifications
    async fn handle_webhook(
        &self,
        params: WebhookParams,
    ) -> Result<Vec<Signal>, Box<dyn std::error::Error + Send + Sync>> {
        // Note: OIDC verification and body size validation are now done synchronously
        // in the HTTP handler before this method is called

        // Parse the Pub/Sub payload and extract message ID for idempotency
        let (email_address, history_id, message_id) =
            self.parse_webhook_payload(&params.payload)?;

        // Find connection by email address, provider, and tenant
        let connection = if let Some(db) = &params.db {
            use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

            let connections = crate::models::connection::Entity::find()
                .filter(crate::models::connection::Column::TenantId.eq(params.tenant_id))
                .filter(crate::models::connection::Column::ProviderSlug.eq("gmail"))
                .filter(crate::models::connection::Column::ExternalId.eq(&email_address))
                .filter(crate::models::connection::Column::Status.eq("active"))
                .all(db)
                .await
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

            match connections.len() {
                0 => {
                    // No connection found - log and return empty signals
                    tracing::warn!(
                        tenant_id = %params.tenant_id,
                        email_address = %email_address,
                        "No active Gmail connection found for email address"
                    );
                    return Ok(vec![]);
                }
                1 => {
                    // Found exactly one connection
                    connections.into_iter().next().unwrap()
                }
                _ => {
                    // Multiple connections found - this is an error condition
                    tracing::error!(
                        tenant_id = %params.tenant_id,
                        email_address = %email_address,
                        count = connections.len(),
                        "Multiple active Gmail connections found for email address"
                    );
                    return Err(Box::new(GmailError::Configuration(format!(
                        "Multiple connections found for email address: {}",
                        email_address
                    )))
                        as Box<dyn std::error::Error + Send + Sync>);
                }
            }
        } else {
            // No database provided - log and return empty signals
            tracing::warn!(
                tenant_id = %params.tenant_id,
                email_address = %email_address,
                "No database connection provided for Gmail webhook processing"
            );
            return Ok(vec![]);
        };

        // Check for duplicate delivery using message ID
        if let Some(db) = &params.db
            && self
                .is_duplicate_delivery(db, &connection.id, &message_id, history_id)
                .await
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?
        {
            tracing::debug!(
                connection_id = %connection.id,
                message_id = %message_id,
                history_id = history_id,
                "Duplicate Pub/Sub message detected, ignoring"
            );
            return Ok(vec![]);
        }

        // Create a signal indicating an email update with Pub/Sub message tracking
        let signal = self.create_email_signal_with_message_id(
            &connection,
            "email_updated",
            &email_address,
            history_id,
            &message_id,
        );

        tracing::debug!("Created email signal for Gmail webhook: {}", signal.id);

        Ok(vec![signal])
    }
}

/// Register the Gmail connector with the registry
pub fn register_gmail_connector(registry: &mut Registry, connector: Arc<GmailConnector>) {
    let metadata = ProviderMetadata::new(
        "gmail".to_string(),
        AuthType::OAuth2,
        DEFAULT_GMAIL_SCOPES.iter().map(|s| s.to_string()).collect(),
        true, // Supports webhooks
    );

    registry.register(connector, metadata);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connectors::trait_::{Cursor, SyncError, SyncErrorKind, SyncParams};
    use serde_json::json;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn build_test_connection() -> Connection {
        Connection {
            id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            provider_slug: "gmail".to_string(),
            external_id: "test@example.com".to_string(),
            status: "active".to_string(),
            display_name: Some("Gmail (test@example.com)".to_string()),
            access_token_ciphertext: Some(b"test-access-token".to_vec()),
            refresh_token_ciphertext: None,
            expires_at: None,
            scopes: None,
            metadata: None,
            created_at: chrono::Utc::now().into(),
            updated_at: chrono::Utc::now().into(),
        }
    }

    #[test]
    fn test_parse_webhook_payload() {
        let connector = GmailConnector::new(
            "test-client-id".to_string(),
            "test-client-secret".to_string(),
        );

        // Create a mock Pub/Sub payload
        let gmail_data = json!({
            "emailAddress": "test@example.com",
            "historyId": 12345
        });

        let encoded_data = general_purpose::STANDARD.encode(gmail_data.to_string().as_bytes());

        let payload = json!({
            "message": {
                "messageId": "msg-123",
                "data": encoded_data,
                "publishTime": "2025-01-01T00:00:00Z"
            },
            "subscription": "projects/test/subscriptions/gmail-sub"
        });

        let (email_address, history_id, message_id) =
            connector.parse_webhook_payload(&payload).unwrap();
        assert_eq!(email_address, "test@example.com");
        assert_eq!(history_id, 12345);
        assert_eq!(message_id, "msg-123");
    }

    #[test]
    fn test_parse_webhook_payload_invalid_format() {
        let connector = GmailConnector::new(
            "test-client-id".to_string(),
            "test-client-secret".to_string(),
        );

        let payload = json!({
            "invalid": "payload"
        });

        let result = connector.parse_webhook_payload(&payload);
        assert!(result.is_err());
    }

    #[test]
    fn test_gmail_connector_with_oidc_verification() {
        // Test that OIDC-enabled connector can be created
        let connector = GmailConnector::new_with_oidc(
            "test-client-id".to_string(),
            "test-client-secret".to_string(),
            Some("test-audience".to_string()),
            Some(vec!["https://accounts.google.com".to_string()]),
        );

        // Verify OIDC verifier is configured
        assert!(connector.oidc_verifier.is_some());
    }

    #[test]
    fn test_gmail_connector_without_oidc_verification() {
        // Test that regular connector can be created without OIDC
        let connector = GmailConnector::new(
            "test-client-id".to_string(),
            "test-client-secret".to_string(),
        );

        // Verify OIDC verifier is not configured
        assert!(connector.oidc_verifier.is_none());
    }

    #[tokio::test]
    async fn test_verify_oidc_token_no_verifier() {
        let connector = GmailConnector::new(
            "test-client-id".to_string(),
            "test-client-secret".to_string(),
        );

        // When OIDC verifier is not configured, verification should fail
        let result = connector.verify_oidc_token(None).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GmailError::Configuration(_)));
    }

    #[test]
    fn test_create_email_signal() {
        let connector = GmailConnector::new(
            "test-client-id".to_string(),
            "test-client-secret".to_string(),
        );

        let connection = build_test_connection();
        let signal =
            connector.create_email_signal(&connection, "email_updated", "test@example.com", 12345);

        assert_eq!(signal.provider_slug, "gmail");
        assert_eq!(signal.kind, "email_updated");
        assert_eq!(
            signal.dedupe_key.unwrap(),
            format!("gmail:{}:12345", connection.id)
        );

        let payload = signal.payload.as_object().unwrap();
        assert_eq!(payload.get("signal_type").unwrap(), "email_updated");
        assert_eq!(payload.get("email_address").unwrap(), "test@example.com");
        assert_eq!(payload.get("history_id").unwrap(), 12345);
    }

    #[test]
    fn test_build_authorize_url() {
        let connector = GmailConnector::new(
            "test-client-id".to_string(),
            "test-client-secret".to_string(),
        );

        let params = AuthorizeParams {
            tenant_id: Uuid::new_v4(),
            redirect_uri: Some("https://test.com/callback".to_string()),
            state: Some("test_state".to_string()),
        };

        let url = connector.build_authorize_url(&params).unwrap();
        assert_eq!(url.scheme(), "https");
        assert_eq!(url.host_str().unwrap(), "accounts.google.com");
        assert_eq!(url.path(), "/o/oauth2/v2/auth");

        let query_pairs: HashMap<_, _> = url.query_pairs().collect();
        assert_eq!(query_pairs.get("client_id").unwrap(), "test-client-id");
        assert_eq!(
            query_pairs.get("redirect_uri").unwrap(),
            "https://test.com/callback"
        );
        assert_eq!(query_pairs.get("state").unwrap(), "test_state");
        assert_eq!(
            query_pairs.get("scope").unwrap(),
            "https://www.googleapis.com/auth/gmail.readonly"
        );
        assert_eq!(query_pairs.get("access_type").unwrap(), "offline");
        assert_eq!(query_pairs.get("prompt").unwrap(), "consent");
        assert_eq!(query_pairs.get("response_type").unwrap(), "code");
    }

    #[tokio::test]
    async fn test_sync_advances_cursor_from_history_response() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/gmail/v1/users/me/history"))
            .and(query_param("startHistoryId", "42"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "historyId": 99,
                "history": [{
                    "id": "history-99",
                    "historyId": 99,
                    "messages": [{
                        "id": "message-1",
                        "threadId": "thread-1",
                        "historyId": "99",
                        "internalDate": "1700000000000",
                        "sizeEstimate": 1024,
                        "snippet": "hello world",
                        "labelIds": ["INBOX"]
                    }],
                    "messagesAdded": [{
                        "id": "message-1",
                        "threadId": "thread-1",
                        "historyId": "99",
                        "internalDate": "1700000000000",
                        "sizeEstimate": 1024,
                        "snippet": "hello world",
                        "labelIds": ["INBOX"]
                    }]
                }]
            })))
            .mount(&server)
            .await;

        let connector = GmailConnector::new_with_history_endpoint_for_tests(
            "test-client-id".to_string(),
            "test-client-secret".to_string(),
            format!("{}/gmail/v1/users", server.uri()),
        );

        let connection = build_test_connection();
        let connection_id = connection.id;
        let params = SyncParams {
            connection,
            cursor: Some(Cursor::from_string("42")),
        };

        let result = connector.sync(params).await.expect("sync should succeed");
        assert_eq!(result.next_cursor.unwrap().as_str(), Some("99"));
        assert!(!result.has_more);
        assert_eq!(result.signals.len(), 1);
        let signal = &result.signals[0];
        assert_eq!(signal.kind, "email_updated");
        assert_eq!(signal.connection_id, connection_id);
        let payload = signal.payload.as_object().unwrap();
        assert_eq!(payload.get("signal_type").unwrap(), "email_updated");
        assert_eq!(payload.get("history_id").and_then(|v| v.as_u64()), Some(99));
        assert_eq!(
            payload.get("messages_added").and_then(|v| v.as_u64()),
            Some(1)
        );
        assert_eq!(payload.get("record_id").unwrap(), "history-99");
    }

    #[tokio::test]
    async fn test_process_history_record_maps_update_counts() {
        let connector = GmailConnector::new(
            "test-client-id".to_string(),
            "test-client-secret".to_string(),
        );
        let connection = build_test_connection();
        let record = GmailHistoryRecord {
            id: "record-1".to_string(),
            history_id: 77,
            messages: Some(vec![GmailMessage {
                id: "message-1".to_string(),
                thread_id: Some("thread-1".to_string()),
                history_id: Some("77".to_string()),
                internal_date: Some("1700000000000".to_string()),
                size_estimate: Some(1024),
                snippet: Some("hello world".to_string()),
                label_ids: vec!["INBOX".to_string()],
            }]),
            messages_added: Some(vec![GmailMessage {
                id: "message-2".to_string(),
                thread_id: Some("thread-1".to_string()),
                history_id: Some("77".to_string()),
                internal_date: Some("1700000000001".to_string()),
                size_estimate: Some(2048),
                snippet: Some("hello again".to_string()),
                label_ids: vec!["INBOX".to_string()],
            }]),
            messages_deleted: None,
        };

        let signals = connector
            .process_history_record(&connection, record, "")
            .await
            .expect("process should succeed");

        assert_eq!(signals.len(), 1);
        let signal = &signals[0];
        assert_eq!(signal.kind, "email_updated");
        let payload = signal.payload.as_object().unwrap();
        assert_eq!(
            payload.get("total_messages").and_then(|v| v.as_u64()),
            Some(1)
        );
        assert_eq!(
            payload.get("messages_added").and_then(|v| v.as_u64()),
            Some(1)
        );
        assert!(!payload.contains_key("messages_deleted"));
    }

    #[tokio::test]
    async fn test_process_history_record_maps_deletes() {
        let connector = GmailConnector::new(
            "test-client-id".to_string(),
            "test-client-secret".to_string(),
        );
        let connection = build_test_connection();
        let record = GmailHistoryRecord {
            id: "record-2".to_string(),
            history_id: 88,
            messages: None,
            messages_added: None,
            messages_deleted: Some(vec![GmailMessage {
                id: "message-removed".to_string(),
                thread_id: Some("thread-removed".to_string()),
                history_id: Some("88".to_string()),
                internal_date: Some("1700000000100".to_string()),
                size_estimate: Some(128),
                snippet: Some("removed".to_string()),
                label_ids: vec!["TRASH".to_string()],
            }]),
        };

        let signals = connector
            .process_history_record(&connection, record, "")
            .await
            .expect("process should succeed");

        assert_eq!(signals[0].kind, "email_deleted");
        let payload = signals[0].payload.as_object().unwrap();
        assert_eq!(
            payload.get("messages_deleted").and_then(|v| v.as_u64()),
            Some(1)
        );
        assert!(!payload.contains_key("messages_added"));
    }

    #[tokio::test]
    async fn test_sync_rate_limits_on_429() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/gmail/v1/users/me/history"))
            .and(query_param("startHistoryId", "1"))
            .respond_with(ResponseTemplate::new(429).insert_header("Retry-After", "120"))
            .mount(&server)
            .await;

        let connector = GmailConnector::new_with_history_endpoint_for_tests(
            "test-client-id".to_string(),
            "test-client-secret".to_string(),
            format!("{}/gmail/v1/users", server.uri()),
        );

        let params = SyncParams {
            connection: build_test_connection(),
            cursor: None,
        };

        let err = connector
            .sync(params)
            .await
            .expect_err("expected rate limit");
        let sync_error = err
            .downcast::<SyncError>()
            .expect("expected SyncError on rate limit");
        assert_eq!(
            sync_error.kind,
            SyncErrorKind::RateLimited {
                retry_after_secs: Some(120)
            }
        );
    }

    #[tokio::test]
    async fn test_sync_rate_limits_on_quota_403() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/gmail/v1/users/me/history"))
            .and(query_param("startHistoryId", "1"))
            .respond_with(
                ResponseTemplate::new(403)
                    .set_body_string(r#"{"error":{"status":"userRateLimitExceeded"}}"#),
            )
            .mount(&server)
            .await;

        let connector = GmailConnector::new_with_history_endpoint_for_tests(
            "test-client-id".to_string(),
            "test-client-secret".to_string(),
            format!("{}/gmail/v1/users", server.uri()),
        );

        let params = SyncParams {
            connection: build_test_connection(),
            cursor: None,
        };

        let err = connector
            .sync(params)
            .await
            .expect_err("expected rate limit");
        let sync_error = err
            .downcast::<SyncError>()
            .expect("expected SyncError on quota rate limit");
        assert_eq!(
            sync_error.kind,
            SyncErrorKind::RateLimited {
                retry_after_secs: Some(60)
            }
        );
    }
}
