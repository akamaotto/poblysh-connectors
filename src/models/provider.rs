//! Provider entity model
//!
//! This module contains the SeaORM entity model for the providers table,
//! which serves as a global catalog of external service providers.

use sea_orm::ActiveModelBehavior;
use sea_orm::entity::prelude::*;
use sea_orm::prelude::DateTimeWithTimeZone;

/// Provider entity representing external service providers
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "providers")]
pub struct Model {
    /// Unique slug identifier for the provider (primary key)
    #[sea_orm(primary_key)]
    pub slug: String,

    /// Display name of the provider (spec)
    pub display_name: String,

    /// Auth type (spec) e.g., oauth2, webhook-only
    pub auth_type: String,

    /// Timestamp when the provider was created
    pub created_at: DateTimeWithTimeZone,

    /// Timestamp when the provider was last updated
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
