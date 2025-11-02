//! SyncJob entity model
//!
//! This module contains the SeaORM entity model for the sync_jobs table,
//! which represents scheduled or webhook-triggered units of work for connectors.

use super::connection::Entity as Connection;
use sea_orm::ActiveModelBehavior;
use sea_orm::entity::prelude::*;
use sea_orm::prelude::DateTimeWithTimeZone;
use serde_json::Value as JsonValue;
use uuid::Uuid;

/// SyncJob entity representing scheduled or webhook-triggered work units
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "sync_jobs")]
pub struct Model {
    /// Unique identifier for the sync job (primary key)
    #[sea_orm(primary_key)]
    pub id: Uuid,

    /// Tenant identifier for multi-tenancy
    pub tenant_id: Uuid,

    /// Slug of the provider this job is for
    pub provider_slug: String,

    /// Connection identifier this job is associated with
    pub connection_id: Uuid,

    /// Type of job (e.g., full, incremental, webhook)
    pub job_type: String,

    /// Current status of the job (e.g., queued, running, succeeded, failed)
    pub status: String,

    /// Job priority for scheduling (higher values = higher priority)
    pub priority: i16,

    /// Number of attempts made for this job
    pub attempts: i32,

    /// Timestamp when the job is scheduled to run
    pub scheduled_at: DateTimeWithTimeZone,

    /// Timestamp when the job becomes eligible for retry after backoff
    pub retry_after: Option<DateTimeWithTimeZone>,

    /// Timestamp when the job started execution
    pub started_at: Option<DateTimeWithTimeZone>,

    /// Timestamp when the job finished execution
    pub finished_at: Option<DateTimeWithTimeZone>,

    /// Opaque provider cursor for incremental sync state
    #[sea_orm(column_type = "JsonBinary")]
    pub cursor: Option<JsonValue>,

    /// Structured error details if the job failed
    #[sea_orm(column_type = "JsonBinary")]
    pub error: Option<JsonValue>,

    /// Timestamp when the sync job was created
    pub created_at: DateTimeWithTimeZone,

    /// Timestamp when the sync job was last updated
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
