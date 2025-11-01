//! Connection entity model
//!
//! This module contains the SeaORM entity model for the connections table,
//! which stores tenant-scoped authorizations to external providers.

use super::provider::Entity as Provider;
use sea_orm::entity::prelude::*;
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::ActiveModelBehavior;
use serde_json::Value as JsonValue;
use uuid::Uuid;

/// Connection entity representing tenant-scoped authorizations to external providers
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "connections")]
pub struct Model {
    /// Unique identifier for the connection (primary key)
    #[sea_orm(primary_key)]
    pub id: Uuid,

    /// Tenant identifier for multi-tenancy
    pub tenant_id: Uuid,

    /// Slug of the provider this connection belongs to
    pub provider_slug: String,

    /// External identifier for the connection (unique per tenant & provider)
    pub external_id: String,

    /// Status of the connection (spec: active|revoked|error)
    pub status: String,

    /// Display name for the connection (optional)
    pub display_name: Option<String>,

    /// Encrypted access token ciphertext (spec; placeholder type)
    pub access_token_ciphertext: Option<Vec<u8>>,

    /// Encrypted refresh token ciphertext (spec; placeholder type)
    pub refresh_token_ciphertext: Option<Vec<u8>>,

    /// Expiration timestamp (spec-aligned)
    pub expires_at: Option<DateTimeWithTimeZone>,

    /// OAuth scopes (optional, stored as JSON array)
    #[sea_orm(column_type = "JsonBinary")]
    pub scopes: Option<JsonValue>,

    /// Provider-specific opaque metadata (spec: JSONB)
    #[sea_orm(column_type = "JsonBinary")]
    pub metadata: Option<JsonValue>,

    /// Timestamp when the connection was created
    pub created_at: DateTimeWithTimeZone,

    /// Timestamp when the connection was last updated
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "Provider",
        from = "Column::ProviderSlug",
        to = "super::provider::Column::Slug"
    )]
    Provider,
}

impl Related<Provider> for Entity {
    fn to() -> RelationDef {
        Relation::Provider.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
