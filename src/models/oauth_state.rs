//! # OAuth State Model
//!
//! This module contains the OAuth state entity for storing OAuth flow state tokens.

use sea_orm::ActiveModelBehavior;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use uuid::Uuid;

/// OAuth State entity for storing OAuth flow state tokens
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "oauth_states")]
pub struct Model {
    /// Primary key UUID
    #[sea_orm(primary_key)]
    pub id: Uuid,

    /// Tenant ID that owns this OAuth state
    pub tenant_id: Uuid,

    /// Provider name (e.g., "github", "google_drive")
    pub provider: String,

    /// State token generated for CSRF protection
    pub state: String,

    /// PKCE code verifier (optional, for enhanced security)
    pub code_verifier: Option<String>,

    /// Expiration timestamp
    pub expires_at: chrono::DateTime<chrono::Utc>,

    /// When the state was created
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// When the state was last updated
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

/// OAuth state creation response for API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthStateResponse {
    /// State token ID
    pub id: Uuid,
    /// State token value
    pub state: String,
    /// Provider name
    pub provider: String,
    /// Expiration timestamp
    pub expires_at: String,
}

impl From<Model> for OAuthStateResponse {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            state: model.state,
            provider: model.provider,
            expires_at: model.expires_at.to_rfc3339(),
        }
    }
}

/// OAuth state lookup for callback validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthStateLookup {
    /// Tenant ID that owns the state
    pub tenant_id: Uuid,
    /// Provider name
    pub provider: String,
    /// State token value
    pub state: String,
    /// Optional PKCE code verifier
    pub code_verifier: Option<String>,
}

impl From<Model> for OAuthStateLookup {
    fn from(model: Model) -> Self {
        Self {
            tenant_id: model.tenant_id,
            provider: model.provider,
            state: model.state,
            code_verifier: model.code_verifier,
        }
    }
}
