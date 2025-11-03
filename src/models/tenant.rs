//! Tenant entity model
//!
//! This module contains the SeaORM entity model for the tenants table,
//! which stores tenant information for multi-tenancy.

use sea_orm::ActiveModelBehavior;
use sea_orm::entity::prelude::*;
use sea_orm::prelude::DateTimeWithTimeZone;

/// Tenant entity representing multi-tenant isolation
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "tenants")]
pub struct Model {
    /// Unique identifier for the tenant (primary key)
    #[sea_orm(primary_key)]
    pub id: Uuid,

    /// Display name for the tenant (optional)
    pub name: Option<String>,

    /// Timestamp when the tenant was created
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
