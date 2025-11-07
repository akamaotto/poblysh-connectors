//! # Grounded Signal Repository
//!
//! This module contains the repository implementation for GroundedSignal entities,
//! providing tenant-scoped data access methods with filtering and pagination.

use crate::error::RepositoryError;
use crate::models::grounded_signal::{
    ActiveModel as GroundedSignalActiveModel, Entity as GroundedSignal, GroundedSignalResponse,
    GroundedSignalStatus, Model as GroundedSignalModel,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, ModelTrait, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect, Set,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Query parameters for listing grounded signals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListGroundedSignalsQuery {
    pub tenant_id: Uuid,
    pub status: Option<GroundedSignalStatus>,
    pub min_score: Option<f32>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Response with pagination metadata
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ListGroundedSignalsResponse {
    pub data: Vec<GroundedSignalResponse>,
    pub pagination: PaginationInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PaginationInfo {
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
    pub has_more: bool,
}

/// Repository for GroundedSignal database operations
pub struct GroundedSignalRepository<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> GroundedSignalRepository<'a> {
    /// Create a new GroundedSignalRepository with the given database connection
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    /// Create a new grounded signal.
    ///
    /// This method is idempotent with respect to the provided idempotency key when supplied:
    /// - If `idempotency_key` is Some, it will first check for an existing grounded signal
    ///   with the same tenant_id and idempotency_key and return it if found.
    /// - Otherwise, it will insert a new grounded signal.
    pub async fn create(
        &self,
        signal_id: Uuid,
        tenant_id: Uuid,
        scores: &crate::models::SignalScores,
        status: GroundedSignalStatus,
        evidence: serde_json::Value,
        recommendation: Option<String>,
        idempotency_key: Option<String>,
    ) -> Result<GroundedSignalResponse, RepositoryError> {
        // If an idempotency key is provided, attempt to reuse an existing grounded signal
        if let Some(ref key) = idempotency_key {
            if let Some(existing) = GroundedSignal::find()
                .filter(crate::models::grounded_signal::Column::TenantId.eq(tenant_id))
                .filter(crate::models::grounded_signal::Column::IdempotencyKey.eq(key.clone()))
                .one(self.db)
                .await
                .map_err(RepositoryError::database_error)?
            {
                return Ok(existing.into());
            }
        }

        let grounded_signal = GroundedSignalActiveModel {
            id: Set(Uuid::new_v4()),
            signal_id: Set(signal_id),
            tenant_id: Set(tenant_id),
            idempotency_key: Set(idempotency_key),
            score_relevance: Set(scores.relevance),
            score_novelty: Set(scores.novelty),
            score_timeliness: Set(scores.timeliness),
            score_impact: Set(scores.impact),
            score_alignment: Set(scores.alignment),
            score_credibility: Set(scores.credibility),
            total_score: Set(scores.total),
            status: Set(status),
            evidence: Set(serde_json::Value::from(evidence)),
            recommendation: Set(recommendation),
            created_at: Set(chrono::Utc::now().into()),
            updated_at: Set(chrono::Utc::now().into()),
        };

        let result = grounded_signal
            .insert(self.db)
            .await
            .map_err(RepositoryError::database_error)?;

        Ok(result.into())
    }

    /// Update grounded signal status and recommendation
    pub async fn update_status(
        &self,
        id: Uuid,
        status: GroundedSignalStatus,
        recommendation: Option<String>,
    ) -> Result<GroundedSignalResponse, RepositoryError> {
        let grounded_signal = GroundedSignal::find_by_id(id)
            .one(self.db)
            .await
            .map_err(RepositoryError::database_error)?
            .ok_or_else(|| RepositoryError::NotFound("GroundedSignal not found".to_string()))?;

        let mut active_model: GroundedSignalActiveModel = grounded_signal.into();
        active_model.status = Set(status);
        active_model.updated_at = Set(chrono::Utc::now().into());

        if let Some(rec) = recommendation {
            active_model.recommendation = Set(Some(rec));
        }

        let result = active_model
            .update(self.db)
            .await
            .map_err(RepositoryError::database_error)?;

        Ok(result.into())
    }

    /// List grounded signals with filters and pagination
    pub async fn list(
        &self,
        query: ListGroundedSignalsQuery,
    ) -> Result<ListGroundedSignalsResponse, RepositoryError> {
        let limit = query.limit.unwrap_or(50).min(200); // Default 50, max 200
        let offset = query.offset.unwrap_or(0);

        let mut db_query = GroundedSignal::find()
            .filter(crate::models::grounded_signal::Column::TenantId.eq(query.tenant_id));

        // Apply optional filters
        if let Some(status) = query.status {
            db_query = db_query.filter(crate::models::grounded_signal::Column::Status.eq(status));
        }

        if let Some(min_score) = query.min_score {
            db_query =
                db_query.filter(crate::models::grounded_signal::Column::TotalScore.gte(min_score));
        }

        // Get total count for pagination
        let total = db_query
            .clone()
            .count(self.db)
            .await
            .map_err(RepositoryError::database_error)?;

        // Apply ordering and pagination
        let results = db_query
            .order_by_desc(crate::models::grounded_signal::Column::CreatedAt)
            .offset(offset as u64)
            .limit(limit as u64)
            .all(self.db)
            .await
            .map_err(RepositoryError::database_error)?;

        let data: Vec<GroundedSignalResponse> =
            results.into_iter().map(|model| model.into()).collect();
        let has_more = (offset + limit) < total as i64;

        Ok(ListGroundedSignalsResponse {
            data,
            pagination: PaginationInfo {
                total: total as i64,
                limit,
                offset,
                has_more,
            },
        })
    }

    /// Get grounded signal by ID
    pub async fn get_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<GroundedSignalResponse>, RepositoryError> {
        let grounded_signal = GroundedSignal::find_by_id(id)
            .one(self.db)
            .await
            .map_err(RepositoryError::database_error)?;

        Ok(grounded_signal.map(|model| model.into()))
    }

    /// Get grounded signals by original signal ID
    pub async fn get_by_signal_id(
        &self,
        signal_id: Uuid,
    ) -> Result<Vec<GroundedSignalResponse>, RepositoryError> {
        let results = GroundedSignal::find()
            .filter(crate::models::grounded_signal::Column::SignalId.eq(signal_id))
            .all(self.db)
            .await
            .map_err(RepositoryError::database_error)?;

        Ok(results.into_iter().map(|model| model.into()).collect())
    }

    /// Get pending grounded signals for background processing
    pub async fn get_pending_signals(
        &self,
        limit: i64,
    ) -> Result<Vec<GroundedSignalModel>, RepositoryError> {
        let results = GroundedSignal::find()
            .filter(crate::models::grounded_signal::Column::Status.eq(GroundedSignalStatus::Draft))
            .order_by_asc(crate::models::grounded_signal::Column::CreatedAt)
            .limit(limit as u64)
            .all(self.db)
            .await
            .map_err(RepositoryError::database_error)?;

        Ok(results)
    }

    /// Delete grounded signal by ID
    pub async fn delete(&self, id: Uuid) -> Result<(), RepositoryError> {
        let grounded_signal = GroundedSignal::find_by_id(id)
            .one(self.db)
            .await
            .map_err(RepositoryError::database_error)?
            .ok_or_else(|| RepositoryError::NotFound("GroundedSignal not found".to_string()))?;

        grounded_signal
            .delete(self.db)
            .await
            .map_err(RepositoryError::database_error)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;
    use crate::db::init_pool;
    use crate::models::grounded_signal::SignalScores;
    use crate::models::signal::ActiveModel as SignalActiveModel;
    use crate::models::tenant::ActiveModel as TenantActiveModel;
    use chrono::Utc;
    use sea_orm::ActiveModelTrait;
    use uuid::Uuid;

    async fn setup_test_data() -> (DatabaseConnection, Uuid, Uuid) {
        let config = AppConfig {
            profile: "test".to_string(),
            ..Default::default()
        };

        let db = init_pool(&config).await.expect("Failed to init test DB");

        // Create tenant
        let tenant_id = Uuid::new_v4();
        let tenant = TenantActiveModel {
            id: sea_orm::Set(tenant_id),
            ..Default::default()
        };
        tenant.insert(&db).await.unwrap();

        // Create signal
        let signal_id = Uuid::new_v4();
        let signal = SignalActiveModel {
            id: sea_orm::Set(signal_id),
            tenant_id: sea_orm::Set(tenant_id),
            provider_slug: sea_orm::Set("test-provider".to_string()),
            kind: sea_orm::Set("test_event".to_string()),
            occurred_at: sea_orm::Set(Utc::now().into()),
            received_at: sea_orm::Set(Utc::now().into()),
            payload: sea_orm::Set(serde_json::json!({"test": "data"})),
            ..Default::default()
        };
        signal.insert(&db).await.unwrap();

        (db, tenant_id, signal_id)
    }

    #[tokio::test]
    async fn test_create_grounded_signal() {
        let (db, tenant_id, signal_id) = setup_test_data().await;
        let repo = GroundedSignalRepository::new(&db);

        let scores = SignalScores {
            relevance: 0.8,
            novelty: 0.6,
            timeliness: 0.9,
            impact: 0.7,
            alignment: 0.8,
            credibility: 0.75,
            total: 0.77,
        };

        let evidence = serde_json::json!({
            "keywords": ["test", "event"],
            "sources": ["test-provider"]
        });

        let result = repo
            .create(
                signal_id,
                tenant_id,
                &scores,
                GroundedSignalStatus::Draft,
                evidence,
                Some("Test recommendation".to_string()),
                None,
            )
            .await
            .unwrap();

        assert_eq!(result.signal_id, signal_id);
        assert_eq!(result.tenant_id, tenant_id);
        assert_eq!(result.status, GroundedSignalStatus::Draft);
        assert_eq!(result.scores.total, 0.77);
        assert_eq!(
            result.recommendation,
            Some("Test recommendation".to_string())
        );
    }

    #[tokio::test]
    async fn test_list_grounded_signals_empty() {
        let (db, tenant_id, _) = setup_test_data().await;
        let repo = GroundedSignalRepository::new(&db);

        let query = ListGroundedSignalsQuery {
            tenant_id,
            status: None,
            min_score: None,
            limit: None,
            offset: None,
        };

        let result = repo.list(query).await.unwrap();
        assert_eq!(result.data.len(), 0);
        assert_eq!(result.pagination.total, 0);
        assert!(!result.pagination.has_more);
    }

    #[tokio::test]
    async fn test_list_grounded_signals_with_data() {
        let (db, tenant_id, signal_id) = setup_test_data().await;
        let repo = GroundedSignalRepository::new(&db);

        // Create test grounded signals
        for i in 0..5 {
            let scores = SignalScores {
                relevance: 0.8,
                novelty: 0.6,
                timeliness: 0.9,
                impact: 0.7 + (i as f32 * 0.05), // Vary impact score
                alignment: 0.8,
                credibility: 0.75,
                total: 0.77 + (i as f32 * 0.05),
            };

            let evidence = serde_json::json!({"index": i});

            let _ = repo
                .create(
                    signal_id,
                    tenant_id,
                    &scores,
                    GroundedSignalStatus::Recommended,
                    evidence,
                    None,
                    None,
                )
                .await
                .unwrap();
        }

        let query = ListGroundedSignalsQuery {
            tenant_id,
            status: None,
            min_score: None,
            limit: None,
            offset: None,
        };

        let result = repo.list(query).await.unwrap();
        assert_eq!(result.data.len(), 5);
        assert_eq!(result.pagination.total, 5);
        assert!(!result.pagination.has_more);

        // Should be ordered by created_at DESC (newest first)
        for item in &result.data {
            assert_eq!(item.status, GroundedSignalStatus::Recommended);
        }
    }

    #[tokio::test]
    async fn test_list_grounded_signals_with_filters() {
        let (db, tenant_id, signal_id) = setup_test_data().await;
        let repo = GroundedSignalRepository::new(&db);

        let scores = SignalScores {
            relevance: 0.8,
            novelty: 0.6,
            timeliness: 0.9,
            impact: 0.7,
            alignment: 0.8,
            credibility: 0.75,
            total: 0.77,
        };

        // Create signals with different statuses
        let _ = repo
            .create(
                signal_id,
                tenant_id,
                &scores,
                GroundedSignalStatus::Draft,
                serde_json::json!({"type": "draft"}),
                None,
                None,
            )
            .await
            .unwrap();

        let _ = repo
            .create(
                signal_id,
                tenant_id,
                &scores,
                GroundedSignalStatus::Recommended,
                serde_json::json!({"type": "recommended"}),
                None,
                None,
            )
            .await
            .unwrap();

        let _ = repo
            .create(
                signal_id,
                tenant_id,
                &scores,
                GroundedSignalStatus::Actioned,
                serde_json::json!({"type": "actioned"}),
                None,
                None,
            )
            .await
            .unwrap();

        // Filter by status
        let query = ListGroundedSignalsQuery {
            tenant_id,
            status: Some(GroundedSignalStatus::Recommended),
            min_score: None,
            limit: None,
            offset: None,
        };

        let result = repo.list(query).await.unwrap();
        assert_eq!(result.data.len(), 1);
        assert!(
            result
                .data
                .iter()
                .all(|gs| gs.status == GroundedSignalStatus::Recommended)
        );
    }

    #[tokio::test]
    async fn test_update_grounded_signal_status() {
        let (db, tenant_id, signal_id) = setup_test_data().await;
        let repo = GroundedSignalRepository::new(&db);

        let scores = SignalScores {
            relevance: 0.8,
            novelty: 0.6,
            timeliness: 0.9,
            impact: 0.7,
            alignment: 0.8,
            credibility: 0.75,
            total: 0.77,
        };

        let created = repo
            .create(
                signal_id,
                tenant_id,
                &scores,
                GroundedSignalStatus::Draft,
                serde_json::json!({}),
                None,
                None,
            )
            .await
            .unwrap();

        assert_eq!(created.status, GroundedSignalStatus::Draft);

        // Update status
        let updated = repo
            .update_status(
                created.id,
                GroundedSignalStatus::Actioned,
                Some("Action taken".to_string()),
            )
            .await
            .unwrap();

        assert_eq!(updated.status, GroundedSignalStatus::Actioned);
        assert_eq!(updated.recommendation, Some("Action taken".to_string()));
        assert!(updated.updated_at > created.updated_at);
    }
}
