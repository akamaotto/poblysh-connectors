//! # Tenant Signal Configuration Model
//!
//! Per-tenant configuration for signal processing thresholds and settings.

use sea_orm::{
    ActiveModelBehavior, DeriveEntityModel, EntityTrait, RelationDef, entity::prelude::*,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "tenant_signal_configs")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub tenant_id: Uuid,

    pub weak_signal_threshold: f32,

    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub scoring_weights: Option<Json>,

    #[sea_orm(column_type = "Text", nullable)]
    pub webhook_url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTimeWithTimeZone>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::tenant::Entity",
        from = "Column::TenantId",
        to = "super::tenant::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Tenant,
}

impl Related<super::tenant::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tenant.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Default for Model {
    fn default() -> Self {
        Self {
            tenant_id: Uuid::new_v4(),
            weak_signal_threshold: 0.7,
            scoring_weights: None,
            webhook_url: None,
            created_at: None,
            updated_at: None,
        }
    }
}

/// Scoring weights configuration
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ScoringWeights {
    pub impact: f32,
    pub relevance: f32,
    pub novelty: f32,
    pub alignment: f32,
    pub timeliness: f32,
    pub credibility: f32,
}

impl Default for ScoringWeights {
    fn default() -> Self {
        Self {
            impact: 0.25,
            relevance: 0.20,
            novelty: 0.15,
            alignment: 0.15,
            timeliness: 0.15,
            credibility: 0.10,
        }
    }
}

impl Model {
    /// Get scoring weights, falling back to defaults if not configured
    pub fn get_scoring_weights(&self) -> ScoringWeights {
        self.scoring_weights
            .as_ref()
            .and_then(|json| serde_json::from_value(json.clone()).ok())
            .unwrap_or_default()
    }

    /// Validate that weights sum to approximately 1.0
    pub fn validate_weights(weights: &ScoringWeights) -> bool {
        let total = weights.impact
            + weights.relevance
            + weights.novelty
            + weights.alignment
            + weights.timeliness
            + weights.credibility;

        (total - 1.0).abs() < 0.001 // Allow small floating point errors
    }
}
