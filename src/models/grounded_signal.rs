//! # Grounded Signal Model
//!
//! Represents signals that have been analyzed and promoted to grounded signals
//! with scoring evidence and recommendations.

use sea_orm::{
    ActiveModelBehavior, DeriveEntityModel, EntityTrait, EnumIter, RelationDef, entity::prelude::*,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "grounded_signals")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,

    pub signal_id: Uuid,
    pub tenant_id: Uuid,

    /// Idempotency key to prevent duplicate grounded signals for the same cluster
    pub idempotency_key: Option<String>,

    // Individual dimension scores
    pub score_relevance: f32,
    pub score_novelty: f32,
    pub score_timeliness: f32,
    pub score_impact: f32,
    pub score_alignment: f32,
    pub score_credibility: f32,
    pub total_score: f32,

    pub status: GroundedSignalStatus,

    #[sea_orm(column_type = "JsonBinary")]
    pub evidence: Json,

    pub recommendation: Option<String>,

    pub created_at: DateTimeWithTimeZone,

    pub updated_at: DateTimeWithTimeZone,
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    EnumIter,
    DeriveActiveEnum,
    Serialize,
    Deserialize,
    ToSchema,
    Default,
)]
#[sea_orm(rs_type = "String", db_type = "Text")]
pub enum GroundedSignalStatus {
    #[sea_orm(string_value = "draft")]
    #[serde(rename = "draft")]
    #[default]
    Draft,

    #[sea_orm(string_value = "recommended")]
    #[serde(rename = "recommended")]
    Recommended,

    #[sea_orm(string_value = "actioned")]
    #[serde(rename = "actioned")]
    Actioned,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::signal::Entity",
        from = "Column::SignalId",
        to = "super::signal::Column::Id"
    )]
    Signal,

    #[sea_orm(
        belongs_to = "super::tenant::Entity",
        from = "Column::TenantId",
        to = "super::tenant::Column::Id"
    )]
    Tenant,
}

impl Related<super::signal::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Signal.def()
    }
}

impl Related<super::tenant::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tenant.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

/// Public representation for API responses (excluding internal fields)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GroundedSignalResponse {
    pub id: Uuid,
    pub signal_id: Uuid,
    pub tenant_id: Uuid,
    pub scores: SignalScores,
    pub status: GroundedSignalStatus,
    pub evidence: serde_json::Value,
    pub recommendation: Option<String>,
    #[schema(value_type = String, example = "2025-01-01T12:00:00Z")]
    pub created_at: DateTimeWithTimeZone,
    #[schema(value_type = String, example = "2025-01-01T12:05:00Z")]
    pub updated_at: DateTimeWithTimeZone,
}

/// Score breakdown for API responses
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SignalScores {
    pub relevance: f32,
    pub novelty: f32,
    pub timeliness: f32,
    pub impact: f32,
    pub alignment: f32,
    pub credibility: f32,
    pub total: f32,
}

impl From<Model> for GroundedSignalResponse {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            signal_id: model.signal_id,
            tenant_id: model.tenant_id,
            scores: SignalScores {
                relevance: model.score_relevance,
                novelty: model.score_novelty,
                timeliness: model.score_timeliness,
                impact: model.score_impact,
                alignment: model.score_alignment,
                credibility: model.score_credibility,
                total: model.total_score,
            },
            status: model.status,
            evidence: model.evidence,
            recommendation: model.recommendation,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}
