//! # SyncJob Repository
//!
//! This module provides repository operations for the sync_jobs table,
//! encapsulating SeaORM operations with tenant-aware access patterns.

use crate::models::sync_job::Column;
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::error::ApiError;
use crate::models::sync_job::{ActiveModel, Entity, Model};
use chrono::DateTime;

/// Configuration for listing jobs with filters
#[derive(Debug, Default)]
pub struct ListJobsConfig {
    pub status: Option<String>,
    pub provider: Option<String>,
    pub connection_id: Option<Uuid>,
    pub job_type: Option<String>,
    pub started_after: Option<DateTime<Utc>>,
    pub finished_after: Option<DateTime<Utc>>,
}

/// Result of listing jobs with cursor pagination
#[derive(Debug)]
pub struct ListJobsResult {
    /// Jobs for the current page
    pub jobs: Vec<Model>,
    /// Cursor for the next page (None if no more pages)
    pub next_cursor: Option<(sea_orm::prelude::DateTimeWithTimeZone, Uuid)>,
}

/// Repository for sync job database operations
pub struct SyncJobRepository {
    db: DatabaseConnection,
}

impl SyncJobRepository {
    /// Create a new SyncJobRepository with the given database connection
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Enqueue a new webhook sync job
    pub async fn enqueue_webhook_job(
        &self,
        tenant_id: Uuid,
        provider_slug: &str,
        connection_id: Uuid,
        cursor: Option<JsonValue>,
    ) -> Result<Model, ApiError> {
        let now = Utc::now().fixed_offset();

        let job = ActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            provider_slug: Set(provider_slug.to_string()),
            connection_id: Set(connection_id),
            job_type: Set("webhook".to_string()),
            status: Set("queued".to_string()),
            priority: Set(50), // Default priority for webhook jobs
            attempts: Set(0),
            scheduled_at: Set(now),
            retry_after: Set(None),
            started_at: Set(None),
            finished_at: Set(None),
            cursor: Set(cursor),
            error: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let result = job.insert(&self.db).await.map_err(|e| {
            tracing::error!("Failed to create webhook sync job: {}", e);
            ApiError::new(
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_SERVER_ERROR",
                "Failed to create sync job",
            )
        })?;

        tracing::info!(
            tenant_id = %tenant_id,
            provider_slug = %result.provider_slug,
            connection_id = %connection_id,
            job_id = %result.id,
            "Webhook sync job enqueued"
        );

        Ok(result)
    }

    /// Find a sync job by ID, ensuring it belongs to the specified tenant
    pub async fn find_by_tenant(
        &self,
        tenant_id: Uuid,
        job_id: Uuid,
    ) -> Result<Option<Model>, ApiError> {
        let job = Entity::find_by_id(job_id)
            .filter(Column::TenantId.eq(tenant_id))
            .one(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to find sync job: {}", e);
                ApiError::new(
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_SERVER_ERROR",
                    "Failed to find sync job",
                )
            })?;

        Ok(job)
    }

    /// List sync jobs for a tenant with optional filtering
    pub async fn list_by_tenant(
        &self,
        tenant_id: Uuid,
        provider_slug: Option<String>,
        status: Option<String>,
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> Result<Vec<Model>, ApiError> {
        let mut query = Entity::find()
            .filter(Column::TenantId.eq(tenant_id))
            .order_by_asc(Column::CreatedAt);

        if let Some(provider) = provider_slug {
            query = query.filter(Column::ProviderSlug.eq(provider));
        }

        if let Some(status_filter) = status {
            query = query.filter(Column::Status.eq(status_filter));
        }

        let results = if let Some(limit_value) = limit {
            query
                .offset(offset.unwrap_or(0))
                .limit(limit_value)
                .all(&self.db)
                .await
        } else {
            query.all(&self.db).await
        }
        .map_err(|e| {
            tracing::error!("Failed to list sync jobs: {}", e);
            ApiError::new(
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_SERVER_ERROR",
                "Failed to list sync jobs",
            )
        })?;

        Ok(results)
    }

    /// Update the status of a sync job
    pub async fn update_status(
        &self,
        tenant_id: Uuid,
        job_id: Uuid,
        status: String,
        error: Option<JsonValue>,
    ) -> Result<Model, ApiError> {
        let job = Entity::find_by_id(job_id)
            .filter(Column::TenantId.eq(tenant_id))
            .one(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to find sync job for status update: {}", e);
                ApiError::new(
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_SERVER_ERROR",
                    "Failed to find sync job",
                )
            })?
            .ok_or_else(|| {
                tracing::error!(job_id = %job_id, tenant_id = %tenant_id, "Sync job not found for status update");
                ApiError::new(
                    axum::http::StatusCode::NOT_FOUND,
                    "NOT_FOUND",
                    "Sync job not found",
                )
            })?;

        let mut active_job: ActiveModel = job.into();
        active_job.status = Set(status);
        active_job.updated_at = Set(Utc::now().fixed_offset());

        if let Some(err) = error {
            active_job.error = Set(Some(err));
        }

        let updated_job = active_job.update(&self.db).await.map_err(|e| {
            tracing::error!("Failed to update sync job status: {}", e);
            ApiError::new(
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_SERVER_ERROR",
                "Failed to update sync job",
            )
        })?;

        Ok(updated_job)
    }

    /// List jobs with cursor pagination and comprehensive filtering
    /// Jobs are ordered by scheduled_at DESC, id DESC for consistent pagination
    pub async fn list_jobs(
        &self,
        tenant_id: Uuid,
        limit: i64,
        cursor: Option<(sea_orm::prelude::DateTimeWithTimeZone, Uuid)>,
        config: ListJobsConfig,
    ) -> Result<ListJobsResult, ApiError> {
        use sea_orm::{Condition, QuerySelect};

        let mut query = Entity::find()
            .filter(Column::TenantId.eq(tenant_id))
            .order_by_desc(Column::ScheduledAt)
            .order_by_desc(Column::Id);

        // Apply cursor if provided for pagination
        if let Some((scheduled_at, id)) = cursor {
            query = query.filter(
                Condition::any()
                    .add(Column::ScheduledAt.lt(scheduled_at))
                    .add(
                        Condition::all()
                            .add(Column::ScheduledAt.eq(scheduled_at))
                            .add(Column::Id.lt(id)),
                    ),
            );
        }

        // Apply filters
        if let Some(ref status_filter) = config.status {
            query = query.filter(Column::Status.eq(status_filter));
        }

        if let Some(ref provider_filter) = config.provider {
            query = query.filter(Column::ProviderSlug.eq(provider_filter));
        }

        if let Some(connection_id_filter) = config.connection_id {
            query = query.filter(Column::ConnectionId.eq(connection_id_filter));
        }

        if let Some(ref job_type_filter) = config.job_type {
            query = query.filter(Column::JobType.eq(job_type_filter));
        }

        if let Some(started_after_filter) = config.started_after {
            query = query.filter(Column::StartedAt.gte(started_after_filter.fixed_offset()));
        }

        if let Some(finished_after_filter) = config.finished_after {
            query = query.filter(Column::FinishedAt.gte(finished_after_filter.fixed_offset()));
        }

        // Fetch one extra record to determine if there are more pages
        let results = query
            .limit(Some(limit as u64 + 1))
            .all(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to list jobs with cursor: {}", e);
                ApiError::new(
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_SERVER_ERROR",
                    "Failed to list jobs",
                )
            })?;

        // Determine if there are more pages and extract the jobs for the current page
        let has_more = results.len() > limit as usize;
        let jobs = if has_more {
            results.into_iter().take(limit as usize).collect()
        } else {
            results
        };

        // Determine next cursor
        let next_cursor = if has_more {
            jobs.last()
                .map(|last_job| (last_job.scheduled_at, last_job.id))
        } else {
            None
        };

        Ok(ListJobsResult { jobs, next_cursor })
    }
}
