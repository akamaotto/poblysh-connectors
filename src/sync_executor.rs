//! Sync Executor
//!
//! Background executor responsible for claiming due sync jobs, invoking provider
//! connectors, persisting signals, and managing cursor advancement with backoff
//! and retry logic.

use chrono::Utc;
use metrics::{counter, histogram};
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

use crate::connectors::{
    ConnectorError, SyncError, SyncErrorKind, SyncParams, SyncResult, registry::Registry,
};
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
    rate_limit_policy: crate::config::RateLimitPolicyConfig,
}

impl SyncExecutor {
    /// Create a new sync executor
    pub fn new(
        db: DatabaseConnection,
        registry: Registry,
        config: ExecutorConfig,
        rate_limit_policy: crate::config::RateLimitPolicyConfig,
    ) -> Self {
        Self {
            db: std::sync::Arc::new(db),
            registry: std::sync::Arc::new(registry),
            config,
            rate_limit_policy,
        }
    }

    /// Get the executor configuration
    pub fn config(&self) -> &ExecutorConfig {
        &self.config
    }

    /// Calculate retry backoff based on rate limit policy and error
    fn calculate_backoff(
        &self,
        sync_error: &SyncError,
        attempts_completed: i32,
        provider_slug: &str,
    ) -> (f64, bool) {
        // Get provider-specific override or use default policy
        let policy = self.rate_limit_policy.provider_overrides.get(provider_slug);

        let base_seconds = policy
            .and_then(|p| p.base_seconds)
            .unwrap_or(self.rate_limit_policy.base_seconds) as f64;
        let max_seconds = policy
            .and_then(|p| p.max_seconds)
            .unwrap_or(self.rate_limit_policy.max_seconds) as f64;
        let jitter_factor = policy
            .and_then(|p| p.jitter_factor)
            .unwrap_or(self.rate_limit_policy.jitter_factor);

        let mut backoff = (base_seconds * 2_f64.powi(attempts_completed)).min(max_seconds);

        // If the error provides retry_after_secs, prefer the max of that and our calculated backoff
        if let SyncErrorKind::RateLimited { retry_after_secs } = &sync_error.kind {
            if let Some(retry_after) = retry_after_secs {
                backoff = backoff.max(*retry_after as f64);
            }
        }

        // Apply jitter
        let jitter = thread_rng().gen_range(0.0..(jitter_factor * backoff));
        let final_backoff = backoff + jitter;

        let is_rate_limited = matches!(sync_error.kind, SyncErrorKind::RateLimited { .. });

        (final_backoff, is_rate_limited)
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
                        self.handle_failure(&job, &e.to_string(), None).await?;
                        Err(e)
                    }
                }
            }
            Err(e) => {
                let execution_time = start_time.elapsed();
                warn!("Job {} failed after {:?}: {}", job.id, execution_time, e);

                // Try to extract SyncError from the error or convert from ConnectorError
                let sync_error = e.downcast_ref::<SyncError>().cloned().or_else(|| {
                    // Try to convert from ConnectorError
                    e.downcast_ref::<ConnectorError>()
                        .map(|connector_err| SyncError::from(connector_err.clone()))
                });

                self.handle_failure(&job, &e.to_string(), sync_error.as_ref())
                    .await?;
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
        sync_error: Option<&SyncError>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let txn = self.db.begin().await?;
        let now = Utc::now();

        // job.attempts already includes the current attempt (incremented during claim)
        let attempts_completed = job.attempts.max(0);
        let prior_failures = attempts_completed.saturating_sub(1).max(0);

        // Calculate backoff using rate limit policy if we have a SyncError
        let (backoff_seconds, is_rate_limited) = if let Some(sync_err) = sync_error {
            self.calculate_backoff(sync_err, prior_failures, &job.provider_slug)
        } else {
            // Fallback to old logic for non-SyncError cases
            let base_seconds = 5.0;
            let max_seconds = 900.0; // 15 minutes
            let jitter_factor = 0.1;
            let exp_backoff = (base_seconds * 2_f64.powi(prior_failures)).min(max_seconds);
            let jitter = thread_rng().gen_range(0.0..(jitter_factor * exp_backoff));
            (exp_backoff + jitter, false)
        };

        let retry_after = now + chrono::Duration::seconds(backoff_seconds as i64);

        // Build error details
        let mut error_details = serde_json::json!({
            "message": error_msg,
            "attempts": attempts_completed,
            "backoff_seconds": backoff_seconds,
            "timestamp": now.to_rfc3339(),
        });

        // Add sync error details if available
        if let Some(sync_err) = sync_error {
            error_details["sync_error"] = serde_json::to_value(sync_err)?;
            if is_rate_limited {
                error_details["is_rate_limited"] = serde_json::Value::Bool(true);

                // Record rate limit metrics
                let metric_labels = vec![("provider", job.provider_slug.clone())];
                counter!("rate_limited_total", &metric_labels).increment(1);
                histogram!("rate_limited_backoff_seconds", &metric_labels).record(backoff_seconds);
            }
        }

        // Update job status back to queued with retry_after
        let mut active_job: SyncJobActiveModel = job.clone().into();
        active_job.status = Set("queued".to_string());
        active_job.attempts = Set(attempts_completed);
        active_job.retry_after = Set(Some(retry_after.into()));
        active_job.error = Set(Some(error_details));
        active_job.updated_at = Set(now.into());
        active_job.update(&txn).await?;

        txn.commit().await?;

        if is_rate_limited {
            warn!(
                "Job {} rate limited (attempt {}), retrying after {:.1}s: {}",
                job.id, attempts_completed, backoff_seconds, error_msg
            );
        } else {
            warn!(
                "Job {} failed (attempt {}), retrying after {:.1}s: {}",
                job.id, attempts_completed, backoff_seconds, error_msg
            );
        }

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
            rate_limit_policy: self.rate_limit_policy.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RateLimitProviderOverride;
    use std::collections::BTreeMap;

    fn create_test_rate_limit_policy() -> crate::config::RateLimitPolicyConfig {
        crate::config::RateLimitPolicyConfig {
            base_seconds: 5,
            max_seconds: 900,
            jitter_factor: 0.1,
            provider_overrides: BTreeMap::new(),
        }
    }

    async fn create_test_executor(policy: crate::config::RateLimitPolicyConfig) -> SyncExecutor {
        let db = sea_orm::Database::connect("sqlite::memory:")
            .await
            .expect("Failed to create in-memory database");
        let registry = Registry::new();
        let config = ExecutorConfig::default();
        SyncExecutor::new(db, registry, config, policy)
    }

    #[tokio::test]
    async fn test_calculate_backoff_default_policy() {
        let policy = create_test_rate_limit_policy();
        let executor = create_test_executor(policy).await;

        let sync_error = SyncError::rate_limited(None);

        // Test exponential backoff with attempts
        let (backoff1, is_rate_limited) =
            executor.calculate_backoff(&sync_error, 0, "test_provider");
        assert!(is_rate_limited);
        assert!((backoff1 >= 5.0 && backoff1 <= 5.5)); // base * 2^0 = 5, jitter may add 0-0.5

        let (backoff2, _) = executor.calculate_backoff(&sync_error, 1, "test_provider");
        assert!((backoff2 >= 10.0 && backoff2 <= 11.0)); // base * 2^1 = 10, jitter may add 0-1

        let (backoff3, _) = executor.calculate_backoff(&sync_error, 2, "test_provider");
        assert!((backoff3 >= 20.0 && backoff3 <= 22.0)); // base * 2^2 = 20, jitter may add 0-2
    }

    #[tokio::test]
    async fn test_calculate_backoff_with_provider_override() {
        let mut provider_overrides = BTreeMap::new();
        provider_overrides.insert(
            "github".to_string(),
            RateLimitProviderOverride {
                base_seconds: Some(10),
                max_seconds: Some(1800),
                jitter_factor: Some(0.2),
            },
        );

        let policy = crate::config::RateLimitPolicyConfig {
            base_seconds: 5,
            max_seconds: 900,
            jitter_factor: 0.1,
            provider_overrides,
        };

        let executor = create_test_executor(policy).await;
        let sync_error = SyncError::rate_limited(None);

        // Test that github provider gets override settings
        let (backoff, _) = executor.calculate_backoff(&sync_error, 0, "github");
        assert!((backoff >= 10.0 && backoff <= 12.0)); // override base = 10, jitter 0-2

        // Test that non-override provider gets default settings
        let (backoff, _) = executor.calculate_backoff(&sync_error, 0, "jira");
        assert!((backoff >= 5.0 && backoff <= 5.5)); // default base = 5, jitter 0-0.5
    }

    #[tokio::test]
    async fn test_calculate_backoff_retry_after_precedence() {
        let policy = create_test_rate_limit_policy();
        let executor = create_test_executor(policy).await;

        // Test that retry_after_secs takes precedence over calculated backoff when larger
        let sync_error = SyncError::rate_limited(Some(300)); // 5 minutes
        let (backoff, _) = executor.calculate_backoff(&sync_error, 0, "test_provider");
        assert!((backoff >= 300.0 && backoff <= 330.0)); // Should use retry_after (300) not calculated (5), jitter up to 30

        // Test that retry_after_secs takes precedence over calculated backoff when smaller
        let sync_error = SyncError::rate_limited(Some(2)); // 2 seconds
        let (backoff, _) = executor.calculate_backoff(&sync_error, 3, "test_provider"); // 3 attempts = 5*2^3 = 40
        assert!((backoff >= 40.0 && backoff <= 44.0)); // Should use calculated (40) not retry_after (2), jitter up to 4
    }

    #[tokio::test]
    async fn test_calculate_backoff_max_capping() {
        let policy = create_test_rate_limit_policy();
        let executor = create_test_executor(policy).await;

        let sync_error = SyncError::rate_limited(None);

        // Test with high attempts to exceed max
        let (backoff, _) = executor.calculate_backoff(&sync_error, 10, "test_provider");
        assert!(backoff <= 900.0 + (900.0 * 0.1)); // Should not exceed max + jitter
        assert!(backoff >= 900.0); // Should be at least max
    }

    #[tokio::test]
    async fn test_sync_error_creation() {
        let unauthorized = SyncError::unauthorized("Invalid token");
        matches!(unauthorized.kind, SyncErrorKind::Unauthorized);

        let rate_limited = SyncError::rate_limited(Some(60));
        if let SyncErrorKind::RateLimited { retry_after_secs } = rate_limited.kind {
            assert_eq!(retry_after_secs, Some(60));
        } else {
            panic!("Expected RateLimited variant");
        }

        let transient = SyncError::transient("Network error");
        matches!(transient.kind, SyncErrorKind::Transient);

        let permanent = SyncError::permanent("Invalid configuration");
        matches!(permanent.kind, SyncErrorKind::Permanent);
    }

    #[tokio::test]
    async fn test_sync_error_with_details() {
        let details = serde_json::json!({"status_code": 429, "reset_time": "2024-01-01T00:00:00Z"});
        let error = SyncError::rate_limited_with_message(Some(60), "API rate limit exceeded")
            .with_details(details.clone());

        assert!(error.details.as_ref().unwrap().get("status_code").is_some());
        if let SyncErrorKind::RateLimited { retry_after_secs } = error.kind {
            assert_eq!(retry_after_secs, Some(60));
        } else {
            panic!("Expected RateLimited variant");
        }
    }

    #[tokio::test]
    async fn test_connector_error_to_sync_error_conversion() {
        // Test RateLimitError conversion
        let rate_limit_error = ConnectorError::RateLimitError {
            retry_after: Some(300),
            limit: Some(100),
        };
        let sync_error = SyncError::from(rate_limit_error);

        if let SyncErrorKind::RateLimited { retry_after_secs } = sync_error.kind {
            assert_eq!(retry_after_secs, Some(300));
        } else {
            panic!("Expected RateLimited variant");
        }

        // Test AuthenticationError conversion
        let auth_error = ConnectorError::AuthenticationError {
            details: "Invalid token".to_string(),
            error_code: Some("AUTH_001".to_string()),
        };
        let sync_error = SyncError::from(auth_error);
        matches!(sync_error.kind, SyncErrorKind::Unauthorized);

        // Test retryable NetworkError conversion
        let network_error = ConnectorError::NetworkError {
            details: "Connection timeout".to_string(),
            retryable: true,
        };
        let sync_error = SyncError::from(network_error);
        matches!(sync_error.kind, SyncErrorKind::Transient);

        // Test non-retryable NetworkError conversion
        let network_error = ConnectorError::NetworkError {
            details: "Invalid endpoint".to_string(),
            retryable: false,
        };
        let sync_error = SyncError::from(network_error);
        matches!(sync_error.kind, SyncErrorKind::Permanent);
    }
}
