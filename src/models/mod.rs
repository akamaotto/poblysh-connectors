//! # Data Models
//!
//! This module contains all the data models used throughout the Connectors API.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub mod connection;
pub mod oauth_state;
pub mod provider;
pub mod signal;
pub mod sync_job;

pub use connection::Entity as Connection;
pub use oauth_state::Entity as OAuthState;
pub use provider::Entity as Provider;
pub use signal::Entity as Signal;
pub use sync_job::Entity as SyncJob;

/// Basic service information response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ServiceInfo {
    /// The name of the service
    pub service: String,
    /// The version of the service
    pub version: String,
}

impl Default for ServiceInfo {
    fn default() -> Self {
        Self {
            service: "poblysh-connectors".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}
