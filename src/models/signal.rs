//! Signal entity model
//!
//! This module contains the SeaORM entity model for the signals table,
//! which stores normalized events emitted by connectors.

use super::connection::Entity as Connection;
use sea_orm::ActiveModelBehavior;
use sea_orm::entity::prelude::*;
use sea_orm::prelude::DateTimeWithTimeZone;
use serde_json::Value as JsonValue;
use uuid::Uuid;

/// Signal entity representing normalized events emitted by connectors
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "signals")]
pub struct Model {
    /// Unique identifier for the signal (primary key)
    #[sea_orm(primary_key)]
    pub id: Uuid,

    /// Tenant identifier for multi-tenancy
    pub tenant_id: Uuid,

    /// Slug of the provider that emitted this signal
    pub provider_slug: String,

    /// Connection identifier that this signal originated from
    pub connection_id: Uuid,

    /// Normalized event kind (e.g., issue_created, pr_merged, message_posted)
    pub kind: String,

    /// Timestamp when the event occurred in the provider system
    pub occurred_at: DateTimeWithTimeZone,

    /// Timestamp when the signal was processed by the system
    pub received_at: DateTimeWithTimeZone,

    /// Normalized event payload
    #[sea_orm(column_type = "JsonBinary")]
    pub payload: JsonValue,

    /// Optional deduplication key for future idempotency logic
    pub dedupe_key: Option<String>,

    /// Timestamp when the signal was created
    pub created_at: DateTimeWithTimeZone,

    /// Timestamp when the signal was last updated
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "Connection",
        from = "Column::ConnectionId",
        to = "super::connection::Column::Id"
    )]
    Connection,
}

impl Related<Connection> for Entity {
    fn to() -> RelationDef {
        Relation::Connection.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
