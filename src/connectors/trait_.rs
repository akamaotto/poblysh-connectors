//! Connector trait definition
//!
//! Defines the standard interface that all connector implementations must follow.

use async_trait::async_trait;
use url::Url;
use uuid::Uuid;

use crate::models::{connection::Model as Connection, signal::Model as Signal};

/// Cursor for pagination in sync operations
#[derive(Debug, Clone)]
pub struct Cursor {
    pub value: String,
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

/// Parameters for webhook handling
#[derive(Debug, Clone)]
pub struct WebhookParams {
    pub payload: serde_json::Value,
    pub tenant_id: Uuid,
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
    /// Returns a collection of signals from the provider.
    async fn sync(
        &self,
        params: SyncParams,
    ) -> Result<Vec<Signal>, Box<dyn std::error::Error + Send + Sync>>;

    /// Handle an incoming webhook from this provider.
    /// Returns a collection of signals generated from the webhook.
    async fn handle_webhook(
        &self,
        params: WebhookParams,
    ) -> Result<Vec<Signal>, Box<dyn std::error::Error + Send + Sync>>;
}
