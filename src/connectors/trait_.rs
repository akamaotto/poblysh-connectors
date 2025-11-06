//! Connector trait definition
//!
//! Defines the standard interface that all connector implementations must follow.

use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use url::Url;
use uuid::Uuid;

use crate::models::{connection::Model as Connection, signal::Model as Signal};

/// Connector-specific error types for structured error handling
#[derive(Debug, Clone)]
pub enum ConnectorError {
    /// HTTP error from upstream provider
    HttpError {
        status: u16,
        body: Option<String>,
        headers: Vec<(String, String)>,
    },
    /// Malformed response from provider
    MalformedResponse {
        details: String,
        partial_data: Option<String>,
    },
    /// Network or connectivity error
    NetworkError { details: String, retryable: bool },
    /// Authentication/authorization error
    AuthenticationError {
        details: String,
        error_code: Option<String>,
    },
    /// Rate limiting error
    RateLimitError {
        retry_after: Option<u64>,
        limit: Option<u32>,
    },
    /// Configuration or setup error
    ConfigurationError { details: String },
    /// Unknown error
    Unknown { details: String },
}

/// Sync-specific error types for structured error handling during sync operations
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SyncError {
    #[serde(flatten)]
    pub kind: SyncErrorKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SyncErrorKind {
    /// Authentication/authorization failure
    Unauthorized,
    /// Rate limited with optional retry after hint
    RateLimited {
        #[serde(skip_serializing_if = "Option::is_none")]
        retry_after_secs: Option<u64>,
    },
    /// Transient/retryable error
    Transient,
    /// Permanent/non-retryable error
    Permanent,
}

impl SyncError {
    pub fn unauthorized<S: Into<String>>(message: S) -> Self {
        Self {
            kind: SyncErrorKind::Unauthorized,
            message: Some(message.into()),
            details: None,
        }
    }

    pub fn rate_limited(retry_after_secs: Option<u64>) -> Self {
        Self {
            kind: SyncErrorKind::RateLimited { retry_after_secs },
            message: None,
            details: None,
        }
    }

    pub fn rate_limited_with_message<S: Into<String>>(
        retry_after_secs: Option<u64>,
        message: S,
    ) -> Self {
        Self {
            kind: SyncErrorKind::RateLimited { retry_after_secs },
            message: Some(message.into()),
            details: None,
        }
    }

    pub fn transient<S: Into<String>>(message: S) -> Self {
        Self {
            kind: SyncErrorKind::Transient,
            message: Some(message.into()),
            details: None,
        }
    }

    pub fn permanent<S: Into<String>>(message: S) -> Self {
        Self {
            kind: SyncErrorKind::Permanent,
            message: Some(message.into()),
            details: None,
        }
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}

impl std::fmt::Display for ConnectorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectorError::HttpError { status, body, .. } => {
                write!(
                    f,
                    "HTTP error {}: {}",
                    status,
                    body.as_deref().unwrap_or("No body")
                )
            }
            ConnectorError::MalformedResponse { details, .. } => {
                write!(f, "Malformed response: {}", details)
            }
            ConnectorError::NetworkError { details, .. } => {
                write!(f, "Network error: {}", details)
            }
            ConnectorError::AuthenticationError { details, .. } => {
                write!(f, "Authentication error: {}", details)
            }
            ConnectorError::RateLimitError {
                retry_after, limit, ..
            } => {
                write!(f, "Rate limit exceeded")?;
                if let Some(limit) = limit {
                    write!(f, " (limit: {})", limit)?;
                }
                if let Some(after) = retry_after {
                    write!(f, " (retry after: {}s)", after)?;
                }
                Ok(())
            }
            ConnectorError::ConfigurationError { details } => {
                write!(f, "Configuration error: {}", details)
            }
            ConnectorError::Unknown { details } => {
                write!(f, "Unknown error: {}", details)
            }
        }
    }
}

impl std::error::Error for ConnectorError {}

impl std::fmt::Display for SyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            SyncErrorKind::Unauthorized => {
                write!(f, "Unauthorized")?;
                if let Some(msg) = &self.message {
                    write!(f, ": {}", msg)?;
                }
            }
            SyncErrorKind::RateLimited { retry_after_secs } => {
                write!(f, "Rate limited")?;
                if let Some(after) = retry_after_secs {
                    write!(f, " (retry after: {}s)", after)?;
                }
                if let Some(msg) = &self.message {
                    write!(f, ": {}", msg)?;
                }
            }
            SyncErrorKind::Transient => {
                write!(f, "Transient error")?;
                if let Some(msg) = &self.message {
                    write!(f, ": {}", msg)?;
                }
            }
            SyncErrorKind::Permanent => {
                write!(f, "Permanent error")?;
                if let Some(msg) = &self.message {
                    write!(f, ": {}", msg)?;
                }
            }
        }
        Ok(())
    }
}

impl std::error::Error for SyncError {}

impl From<ConnectorError> for SyncError {
    fn from(connector_error: ConnectorError) -> Self {
        match connector_error {
            ConnectorError::RateLimitError {
                retry_after,
                limit: _,
            } => SyncError::rate_limited(retry_after),
            ConnectorError::AuthenticationError {
                details,
                error_code: _,
            } => SyncError::unauthorized(details),
            ConnectorError::NetworkError { details, retryable } => {
                if retryable {
                    SyncError::transient(details)
                } else {
                    SyncError::permanent(details)
                }
            }
            ConnectorError::HttpError {
                status,
                body,
                headers: _,
            } => {
                if status == 429 {
                    // Try to extract retry_after from body if available
                    let retry_after = body
                        .as_ref()
                        .and_then(|b| b.strip_prefix("Retry-After: "))
                        .and_then(|s| s.split_whitespace().next())
                        .and_then(|s| s.parse().ok());
                    SyncError::rate_limited(retry_after)
                } else if (400..500).contains(&status) {
                    SyncError::permanent(format!(
                        "HTTP error {}: {}",
                        status,
                        body.unwrap_or_default()
                    ))
                } else {
                    SyncError::transient(format!(
                        "HTTP error {}: {}",
                        status,
                        body.unwrap_or_default()
                    ))
                }
            }
            ConnectorError::MalformedResponse {
                details,
                partial_data: _,
            } => SyncError::transient(format!("Malformed response: {}", details)),
            ConnectorError::ConfigurationError { details } => {
                SyncError::permanent(format!("Configuration error: {}", details))
            }
            ConnectorError::Unknown { details } => {
                SyncError::transient(format!("Unknown error: {}", details))
            }
        }
    }
}

/// Cursor for pagination in sync operations.
///
/// Wraps an opaque JSON payload returned by connectors. The payload may be a
/// primitive or structured object and must round-trip without alteration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(transparent)]
pub struct Cursor(pub serde_json::Value);

impl Cursor {
    /// Construct a cursor from any JSON value.
    pub fn from_json(value: serde_json::Value) -> Self {
        Self(value)
    }

    /// Convenience helper to build a string cursor.
    pub fn from_string<S: Into<String>>(value: S) -> Self {
        Self(serde_json::Value::String(value.into()))
    }

    /// Borrow the underlying JSON value.
    pub fn as_json(&self) -> &serde_json::Value {
        &self.0
    }

    /// Attempt to access the cursor as a string.
    pub fn as_str(&self) -> Option<&str> {
        self.0.as_str()
    }
}

impl From<Cursor> for serde_json::Value {
    fn from(cursor: Cursor) -> Self {
        cursor.0
    }
}

impl From<serde_json::Value> for Cursor {
    fn from(value: serde_json::Value) -> Self {
        Cursor::from_json(value)
    }
}

/// Parameters for authorization flow
#[derive(Debug, Clone)]
pub struct AuthorizeParams {
    pub tenant_id: Uuid,
    pub redirect_uri: Option<String>,
    pub state: Option<String>,
}

/// Parameters for token exchange
#[derive(Debug, Clone)]
pub struct ExchangeTokenParams {
    pub code: String,
    pub redirect_uri: Option<String>,
    pub tenant_id: Uuid,
}

/// Parameters for sync operation
#[derive(Debug, Clone)]
pub struct SyncParams {
    pub connection: Connection,
    pub cursor: Option<Cursor>,
}

/// Result from a sync operation
#[derive(Debug, Clone)]
pub struct SyncResult {
    pub signals: Vec<Signal>,
    pub next_cursor: Option<Cursor>,
    pub has_more: bool,
}

/// Parameters for webhook handling
#[derive(Debug, Clone)]
pub struct WebhookParams {
    pub payload: serde_json::Value,
    pub tenant_id: Uuid,
    pub db: Option<DatabaseConnection>,
}

#[async_trait]
pub trait Connector: Send + Sync {
    /// Begin the authorization flow for this provider.
    /// Returns an authorization URL for the user to visit.
    async fn authorize(
        &self,
        params: AuthorizeParams,
    ) -> Result<Url, Box<dyn std::error::Error + Send + Sync>>;

    /// Exchange an authorization code for a connection.
    async fn exchange_token(
        &self,
        params: ExchangeTokenParams,
    ) -> Result<Connection, Box<dyn std::error::Error + Send + Sync>>;

    /// Refresh an expired access token for an existing connection.
    async fn refresh_token(
        &self,
        connection: Connection,
    ) -> Result<Connection, Box<dyn std::error::Error + Send + Sync>>;

    /// Perform a sync operation for this provider.
    /// Returns signals and pagination information from the provider.
    async fn sync(
        &self,
        params: SyncParams,
    ) -> Result<SyncResult, Box<dyn std::error::Error + Send + Sync>>;

    /// Handle an incoming webhook from this provider.
    /// Returns a collection of signals generated from the webhook.
    async fn handle_webhook(
        &self,
        params: WebhookParams,
    ) -> Result<Vec<Signal>, Box<dyn std::error::Error + Send + Sync>>;
}
