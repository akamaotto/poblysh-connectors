//! Sync Executor
//!
//! Background executor responsible for claiming due sync jobs, invoking provider
//! connectors, persisting signals, and managing cursor advancement with backoff
//! and retry logic.

use chrono::Utc;
use rand::{Rng, thread_rng};
use sea_orm::prelude::*;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect, QueryTrait, Set, TransactionTrait,
};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

use crate::connectors::{SyncParams, SyncResult, registry::Registry};
use crate::models::{
    connection::{ActiveModel as ConnectionActiveModel, Entity as ConnectionEntity},
    signal::ActiveModel as SignalActiveModel,
    sync_job::{self, ActiveModel as SyncJobActiveModel, Entity as SyncJobEntity},
};
use crate::repositories::sync_metadata::{ConnectionSyncMetadata, cursor_from_json};

/// Configuration for the sync executor
#[derive(Debug, Clone)]
pub struct ExecutorConfig {
    /// Milliseconds between executor ticks
    pub tick_ms: u64,
    /// Maximum number of concurrent jobs
    pub concurrency: usize,
    /// Maximum number of jobs to claim in one batch
    pub claim_batch: usize,
    /// Maximum number of seconds a job can run before being timed out
    pub max_run_seconds: u64,
    /// Maximum number of items to process per run
    pub max_items_per_run: usize,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            tick_ms: 5000, // 5 seconds
            concurrency: 10,
            claim_batch: 50,
            max_run_seconds: 300, // 5 minutes
            max_items_per_run: 1000,
        }
    }
}

/// Sync executor responsible for running background sync jobs
pub struct SyncExecutor {
    pub db: std::sync::Arc<DatabaseConnection>,
    pub registry: std::sync::Arc<Registry>,
    config: ExecutorConfig,
}

impl SyncExecutor {
    /// Create a new sync executor
    pub fn new(db: DatabaseConnection, registry: Registry, config: ExecutorConfig) -> Self {
        Self {
            db: std::sync::Arc::new(db),
            registry: std::sync::Arc::new(registry),
            config,
        }
    }

    /// Get the executor configuration
    pub fn config(&self) -> &ExecutorConfig {
        &self.config
    }

    /// Run the executor loop
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting sync executor with config: {:?}", self.config);

        loop {
            let start = std::time::Instant::now();

            match self.claim_and_run_jobs().await {
                Ok(count) => {
                    if count > 0 {
                        debug!("Executed {} sync jobs", count);
                    }
                }
                Err(e) => {
                    error!("Error executing sync jobs: {}", e);
                }
            }

            // Sleep for remaining tick time
            let elapsed = start.elapsed();
            let tick_duration = Duration::from_millis(self.config.tick_ms);
            if elapsed < tick_duration {
                sleep(tick_duration - elapsed).await;
            }
        }
    }

    /// Claim due jobs and execute them
    #[instrument(skip(self), fields(batch_size = self.config.claim_batch))]
    pub async fn claim_and_run_jobs(
        &self,
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let timer = std::time::Instant::now();
        let jobs = self.claim_jobs().await?;
        let count = jobs.len();

        if jobs.is_empty() {
            debug!("No due jobs found to claim");
            return Ok(0);
        }

        info!("Claimed {} jobs for execution", count);

        // Create a bounded semaphore to limit concurrent jobs
        let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(self.config.concurrency));

        // Spawn all jobs with concurrency control
        let mut handles = Vec::new();
        for job in jobs {
            let executor = self.clone();
            let permit = semaphore
                .clone()
                .acquire_owned()
                .await
                .map_err(|_| "Failed to acquire semaphore permit")?;

            let handle = tokio::spawn(async move {
                let _permit = permit; // Holds the permit until job completes
                if let Err(e) = executor.run_single_job(job).await {
                    error!("Error running job: {}", e);
                }
            });
            handles.push(handle);
        }

        // Wait for all jobs to complete
        for handle in handles {
            let _ = handle.await;
        }

        let elapsed = timer.elapsed();
        info!(
            "Completed {} jobs in {:.2}s (avg: {:.2}s/job)",
            count,
            elapsed.as_secs_f64(),
            elapsed.as_secs_f64() / count as f64
        );

        Ok(count)
    }

    /// Claim due jobs from the database using truly atomic approach
    async fn claim_jobs(
        &self,
    ) -> Result<Vec<sync_job::Model>, Box<dyn std::error::Error + Send + Sync>> {
        let now = Utc::now();
        let txn = self.db.begin().await?;

        // First, find eligible jobs with single-flight constraint
        let eligible_jobs = SyncJobEntity::find()
            .select_only()
            .column(sync_job::Column::Id)
            .filter(
                sync_job::Column::Status
                    .eq("queued")
                    .and(sync_job::Column::ScheduledAt.lte(now))
                    .and(
                        sync_job::Column::RetryAfter
                            .is_null()
                            .or(sync_job::Column::RetryAfter.lte(now)),
                    ),
            )
            .filter(
                sync_job::Column::ConnectionId.not_in_subquery(
                    SyncJobEntity::find()
                        .select_only()
                        .column(sync_job::Column::ConnectionId)
                        .filter(sync_job::Column::Status.eq("running"))
                        .into_query(),
                ),
            )
            .order_by_desc(sync_job::Column::Priority)
            .order_by_asc(sync_job::Column::ScheduledAt)
            .limit(Some(self.config.claim_batch as u64))
            .into_tuple::<Uuid>()
            .all(&txn)
            .await?;

        // Atomically claim the jobs in a single UPDATE statement
        let update_result = if !eligible_jobs.is_empty() {
            SyncJobEntity::update_many()
                .col_expr(sync_job::Column::Status, Expr::value("running"))
                .col_expr(sync_job::Column::StartedAt, Expr::value(now))
                .col_expr(
                    sync_job::Column::Attempts,
                    Expr::value(Expr::col(sync_job::Column::Attempts).add(1)),
                )
                .filter(sync_job::Column::Id.is_in(eligible_jobs))
                .filter(sync_job::Column::Status.eq("queued")) // Double-check they're still queued
                .exec(&txn)
                .await?
        } else {
            txn.commit().await?;
            return Ok(Vec::new());
        };

        // Fetch only the jobs that were actually claimed (those affected by the UPDATE)
        // This ensures we only return jobs that we successfully transitioned to "running" status
        let claimed_jobs = if update_result.rows_affected > 0 {
            SyncJobEntity::find()
                .filter(sync_job::Column::Status.eq("running"))
                .filter(sync_job::Column::StartedAt.eq(now))
                .all(&txn)
                .await?
        } else {
            Vec::new()
        };

        txn.commit().await?;
        Ok(claimed_jobs)
    }

    /// Run a single sync job
    #[instrument(skip(self), fields(job_id = %job.id, connection_id = %job.connection_id, provider_slug = %job.provider_slug))]
    pub async fn run_single_job(
        &self,
        job: sync_job::Model,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let start_time = std::time::Instant::now();
        info!("Starting sync job {} (attempt {})", job.id, job.attempts);

        match self.execute_job(&job).await {
            Ok(sync_result) => {
                let execution_time = start_time.elapsed();
                debug!("Job {} executed in {:?}", job.id, execution_time);

                match self.handle_success(&job, sync_result).await {
                    Ok(()) => {
                        let total_time = start_time.elapsed();
                        info!(
                            "Successfully completed job {} in {:?} (execution: {:?}, total: {:?})",
                            job.id, total_time, execution_time, total_time
                        );
                        Ok(())
                    }
                    Err(e) => {
                        error!("Error handling success for job {}: {}", job.id, e);
                        self.handle_failure(&job, &format!("Success handling failed: {}", e))
                            .await?;
                        Err(e)
                    }
                }
            }
            Err(e) => {
                let execution_time = start_time.elapsed();
                warn!("Job {} failed after {:?}: {}", job.id, execution_time, e);
                self.handle_failure(&job, &format!("{}", e)).await?;
                Err(e)
            }
        }
    }

    /// Execute the actual sync job
    async fn execute_job(
        &self,
        job: &sync_job::Model,
    ) -> Result<SyncResult, Box<dyn std::error::Error + Send + Sync>> {
        // Get connection
        let connection = ConnectionEntity::find_by_id(job.connection_id)
            .one(&*self.db)
            .await?
            .ok_or("Connection not found")?;

        // Get connector
        let connector = self.registry.get(&job.provider_slug)?;

        // Resolve cursor: prefer job cursor, then connection metadata cursor
        let cursor = job
            .cursor
            .clone()
            .and_then(|cursor| cursor_from_json(Some(&cursor)))
            .or_else(|| {
                let sync_metadata =
                    ConnectionSyncMetadata::from_connection_metadata(connection.metadata.as_ref());
                sync_metadata.cursor
            });

        // Create sync params
        let sync_params = SyncParams { connection, cursor };

        // Execute sync with timeout
        let sync_result = tokio::time::timeout(
            Duration::from_secs(self.config.max_run_seconds),
            connector.sync(sync_params),
        )
        .await
        .map_err(|_| "Job timed out")??;

        Ok(sync_result)
    }

    /// Handle successful job completion
    async fn handle_success(
        &self,
        job: &sync_job::Model,
        sync_result: SyncResult,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let txn = self.db.begin().await?;
        let now = Utc::now();

        // Persist signals
        if !sync_result.signals.is_empty() {
            for signal in &sync_result.signals {
                let active_signal: SignalActiveModel = signal.clone().into();
                active_signal.insert(&txn).await?;
            }
        }

        // Update connection cursor if provided
        if sync_result.next_cursor.is_some() {
            let connection = ConnectionEntity::find_by_id(job.connection_id)
                .one(&txn)
                .await?
                .ok_or("Connection not found")?;

            if let Some(next_cursor) = &sync_result.next_cursor {
                let mut sync_metadata =
                    ConnectionSyncMetadata::from_connection_metadata(connection.metadata.as_ref());
                sync_metadata.cursor = Some(next_cursor.clone());

                let updated_metadata =
                    sync_metadata.into_connection_metadata(connection.metadata.as_ref());

                let mut active_connection: ConnectionActiveModel = connection.into();
                active_connection.metadata = Set(Some(updated_metadata));
                active_connection.updated_at = Set(now.into());
                active_connection.update(&txn).await?;
            }
        }

        // Update job status to succeeded
        let mut active_job: SyncJobActiveModel = job.clone().into();
        active_job.status = Set("succeeded".to_string());
        active_job.finished_at = Set(Some(now.into()));
        active_job.updated_at = Set(now.into());
        active_job.update(&txn).await?;

        // Store the signal count before moving sync_result
        let signal_count = sync_result.signals.len();

        // If has_more, create follow-up incremental job
        if sync_result.has_more
            && sync_result.next_cursor.is_some()
            && let Some(next_cursor) = sync_result.next_cursor
        {
            let cursor_json = serde_json::to_value(next_cursor)?;
            let follow_up_job = SyncJobActiveModel {
                id: Set(Uuid::new_v4()),
                tenant_id: Set(job.tenant_id),
                provider_slug: Set(job.provider_slug.clone()),
                connection_id: Set(job.connection_id),
                job_type: Set("incremental".to_string()),
                status: Set("queued".to_string()),
                priority: Set(job.priority),
                attempts: Set(0),
                scheduled_at: Set(now.into()),
                retry_after: Set(None),
                started_at: Set(None),
                finished_at: Set(None),
                cursor: Set(Some(cursor_json)),
                error: Set(None),
                created_at: Set(now.into()),
                updated_at: Set(now.into()),
            };
            follow_up_job.insert(&txn).await?;
        }

        txn.commit().await?;

        info!(
            "Successfully completed job {} with {} signals{}",
            job.id,
            signal_count,
            if sync_result.has_more {
                " (has_more=true)"
            } else {
                ""
            }
        );

        Ok(())
    }

    /// Handle job failure
    async fn handle_failure(
        &self,
        job: &sync_job::Model,
        error_msg: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let txn = self.db.begin().await?;
        let now = Utc::now();

        // Calculate backoff with jitter
        let base_seconds = 5;
        let max_seconds = 900; // 15 minutes
        let jitter_factor = 0.1;

        // job.attempts already includes the current attempt (incremented during claim)
        let attempts_completed = job.attempts.max(0);
        let prior_failures = attempts_completed.saturating_sub(1).max(0);
        let exp_backoff =
            (base_seconds as f64 * 2_f64.powi(prior_failures)).min(max_seconds as f64);
        let jitter = thread_rng().gen_range(0.0..(jitter_factor * exp_backoff));
        let backoff_seconds = exp_backoff + jitter;

        let retry_after = now + chrono::Duration::seconds(backoff_seconds as i64);

        // Update job status back to queued with retry_after
        let mut active_job: SyncJobActiveModel = job.clone().into();
        active_job.status = Set("queued".to_string());
        active_job.attempts = Set(attempts_completed);
        active_job.retry_after = Set(Some(retry_after.into()));
        active_job.error = Set(Some(serde_json::json!({
            "message": error_msg,
            "attempts": attempts_completed,
            "backoff_seconds": backoff_seconds,
            "timestamp": now.to_rfc3339(),
        })));
        active_job.updated_at = Set(now.into());
        active_job.update(&txn).await?;

        txn.commit().await?;

        warn!(
            "Job {} failed (attempt {}), retrying after {:.1}s: {}",
            job.id, attempts_completed, backoff_seconds, error_msg
        );

        Ok(())
    }
}

// Implement Clone for the executor to allow it to be used in spawned tasks
impl Clone for SyncExecutor {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            registry: self.registry.clone(),
            config: self.config.clone(),
        }
    }
}
