//! # Sync Scheduler
//!
//! Background task that evaluates active connections, applies jittered intervals,
//! and enqueues incremental sync jobs while maintaining at-most-once semantics
//! per connection. The scheduler persists cadence metadata alongside job rows
//! so multiple instances may coordinate safely.

use std::sync::Arc;

use axum::http::StatusCode;
use chrono::{DateTime, Duration, FixedOffset, Utc};
use metrics::{counter, gauge, histogram};
use rand::Rng;
use sea_orm::sea_query::{LockBehavior, LockType};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, DatabaseTransaction, DbErr,
    EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, RuntimeErr, Set,
    TransactionTrait,
};
use tokio::time::{Duration as TokioDuration, Instant, sleep};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

use crate::config::{AppConfig, SchedulerConfig};
use crate::error::ApiError;
use crate::models::connection::{
    ActiveModel as ConnectionActiveModel, Column as ConnectionColumn, Entity as Connection,
    Model as ConnectionModel,
};
use crate::models::sync_job::{
    ActiveModel as SyncJobActiveModel, Column as SyncJobColumn, Entity as SyncJob,
};
use crate::repositories::sync_metadata::{ConnectionSyncMetadata, MIN_SYNC_INTERVAL_SECONDS};

/// Default number of connections evaluated per tick.
const DEFAULT_BATCH_SIZE: usize = 128;

/// Index name for the interval uniqueness guard.
const INTERVAL_UNIQUE_INDEX: &str = "idx_sync_jobs_incremental_pending";

/// Background scheduler service.
pub struct SyncScheduler {
    config: Arc<AppConfig>,
    db: Arc<DatabaseConnection>,
    batch_size: usize,
}

#[derive(Debug, Default)]
struct TickStats {
    connections_polled: u64,
    jobs_enqueued: u64,
    jobs_skipped_pending: u64,
    jobs_skipped_not_due: u64,
    backlog_connections: u64,
    connections_with_errors: u64,
}

#[derive(Debug, Clone)]
struct DueComputation {
    job_due: DateTime<Utc>,
    next_run_at: DateTime<Utc>,
    is_overdue: bool,
}

impl SyncScheduler {
    /// Create a new scheduler instance.
    pub fn new(config: Arc<AppConfig>, db: Arc<DatabaseConnection>) -> Self {
        Self {
            config,
            db,
            batch_size: DEFAULT_BATCH_SIZE,
        }
    }

    /// Override the number of connections processed per tick (primarily for tests).
    #[allow(dead_code)]
    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = batch_size.max(1);
        self
    }

    /// Run the scheduler loop until the provided shutdown token fires.
    #[instrument(skip_all)]
    pub async fn run(self, shutdown: CancellationToken) -> Result<(), ApiError> {
        info!("Starting sync scheduler");
        let tick_interval = TokioDuration::from_secs(self.config.scheduler.tick_interval_seconds);

        loop {
            tokio::select! {
                _ = shutdown.cancelled() => {
                    info!("Sync scheduler shutdown requested");
                    break;
                }
                _ = sleep(tick_interval) => {
                    let tick_started = Instant::now();
                    if let Err(err) = self.tick().await {
                        error!(error = ?err, "Scheduler tick failed");
                    }
                    let elapsed = tick_started.elapsed();
                    histogram!("sync_scheduler_tick_duration_ms")
                        .record(elapsed.as_secs_f64() * 1_000.0);
                }
            }
        }

        info!("Sync scheduler stopped");
        Ok(())
    }

    async fn tick(&self) -> Result<(), ApiError> {
        let now = Utc::now();
        let mut stats = TickStats::default();

        let candidate_ids = self.load_candidate_ids().await?;

        for connection_id in candidate_ids {
            match self
                .process_connection(connection_id, now, &mut stats)
                .await
            {
                Ok(()) => {}
                Err(err) => {
                    stats.connections_with_errors += 1;
                    error!(
                        error = ?err,
                        connection_id = %connection_id,
                        "Failed to process connection for scheduling"
                    );
                }
            }
        }

        gauge!("sync_scheduler_backlog_gauge").set(stats.backlog_connections as f64);

        debug!(
            polled = stats.connections_polled,
            enqueued = stats.jobs_enqueued,
            skipped_pending = stats.jobs_skipped_pending,
            skipped_not_due = stats.jobs_skipped_not_due,
            errors = stats.connections_with_errors,
            backlog = stats.backlog_connections,
            "Scheduler tick completed"
        );

        Ok(())
    }

    async fn load_candidate_ids(&self) -> Result<Vec<Uuid>, ApiError> {
        let mut models = Connection::find()
            .filter(ConnectionColumn::Status.eq("active"))
            .order_by_asc(ConnectionColumn::CreatedAt)
            .limit((self.batch_size as u64).saturating_mul(4))
            .all(self.db.as_ref())
            .await
            .map_err(|err| map_db_err("failed to load active connections", err))?;

        models.sort_by_key(|connection| {
            let metadata =
                ConnectionSyncMetadata::from_connection_metadata(connection.metadata.as_ref());
            metadata
                .next_run_at
                .or(metadata.first_activated_at)
                .unwrap_or_else(|| connection.created_at.with_timezone(&Utc))
        });

        Ok(models
            .into_iter()
            .take(self.batch_size)
            .map(|connection| connection.id)
            .collect())
    }

    async fn process_connection(
        &self,
        connection_id: Uuid,
        now: DateTime<Utc>,
        stats: &mut TickStats,
    ) -> Result<(), ApiError> {
        let mut txn = self
            .db
            .begin()
            .await
            .map_err(|err| map_db_err("failed to start scheduler transaction", err))?;

        let Some(connection) = Connection::find()
            .filter(ConnectionColumn::Id.eq(connection_id))
            .filter(ConnectionColumn::Status.eq("active"))
            .lock_with_behavior(LockType::Update, LockBehavior::SkipLocked)
            .one(&txn)
            .await
            .map_err(|err| map_db_err("failed to load connection for scheduling", err))?
        else {
            txn.rollback()
                .await
                .map_err(|err| map_db_err("failed to rollback scheduler transaction", err))?;
            return Ok(());
        };

        stats.connections_polled += 1;

        let mut metadata =
            ConnectionSyncMetadata::from_connection_metadata(connection.metadata.as_ref());
        let mut metadata_dirty = metadata.sanitize_interval(&self.config.scheduler);

        if metadata.first_activated_at.is_none() {
            metadata.first_activated_at = Some(connection.created_at.with_timezone(&Utc));
            metadata_dirty = true;
        }

        let base_interval = metadata.effective_interval_seconds(&self.config.scheduler);
        if base_interval < MIN_SYNC_INTERVAL_SECONDS {
            warn!(
                connection_id = %connection.id,
                "Base interval smaller than minimum; using scheduler default"
            );
        }

        let last_finished = self
            .last_incremental_finished_at(connection.id, &mut txn)
            .await?;

        let due = compute_due_times(
            &metadata,
            base_interval,
            last_finished,
            metadata
                .first_activated_at
                .unwrap_or_else(|| connection.created_at.with_timezone(&Utc)),
            now,
        );

        let pending_exists = SyncJob::find()
            .filter(SyncJobColumn::ConnectionId.eq(connection.id))
            .filter(SyncJobColumn::JobType.eq("incremental"))
            .filter(SyncJobColumn::Status.is_in(vec!["queued", "running"]))
            .count(&txn)
            .await
            .map_err(|err| map_db_err("failed to check pending jobs", err))?
            > 0;

        if pending_exists {
            stats.jobs_skipped_pending += 1;
            debug!(
                connection_id = %connection.id,
                "Skipping scheduling; pending incremental job exists"
            );
            if metadata_dirty {
                self.persist_metadata(&mut txn, &connection, &metadata, now)
                    .await?;
            }
            txn.commit()
                .await
                .map_err(|err| map_db_err("failed to commit scheduler transaction", err))?;
            return Ok(());
        }

        if now < due.job_due {
            stats.jobs_skipped_not_due += 1;
            debug!(
                connection_id = %connection.id,
                due_at = %due.job_due,
                "Connection not yet due for scheduling"
            );
            if metadata_dirty {
                self.persist_metadata(&mut txn, &connection, &metadata, now)
                    .await?;
            }
            txn.commit()
                .await
                .map_err(|err| map_db_err("failed to commit scheduler transaction", err))?;
            return Ok(());
        }

        let jitter_seconds = sample_jitter_seconds(&self.config.scheduler, base_interval);
        let scheduled_at = due
            .job_due
            .checked_add_signed(Duration::seconds(jitter_seconds as i64))
            .unwrap_or(now);

        metadata.next_run_at = Some(due.next_run_at);
        metadata.last_jitter_seconds = Some(jitter_seconds);
        metadata_dirty = true;

        let job_model = SyncJobActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(connection.tenant_id),
            provider_slug: Set(connection.provider_slug.clone()),
            connection_id: Set(connection.id),
            job_type: Set("incremental".to_string()),
            status: Set("queued".to_string()),
            priority: Set(30),
            attempts: Set(0),
            scheduled_at: Set(to_db_time(scheduled_at)),
            retry_after: Set(None),
            started_at: Set(None),
            finished_at: Set(None),
            cursor: Set(None),
            error: Set(None),
            created_at: Set(to_db_time(now)),
            updated_at: Set(to_db_time(now)),
        };

        match job_model.insert(&txn).await {
            Ok(_) | Err(DbErr::UnpackInsertId) => {
                stats.jobs_enqueued += 1;
                if due.is_overdue {
                    stats.backlog_connections += 1;
                }
                info!(
                    connection_id = %connection.id,
                    provider_slug = %connection.provider_slug,
                    tenant_id = %connection.tenant_id,
                    base_interval_seconds = base_interval,
                    jitter_seconds = jitter_seconds,
                    scheduled_at = %scheduled_at,
                    next_run_at = %due.next_run_at,
                    "Enqueued incremental sync job"
                );

                let metric_labels = vec![
                    ("provider_slug", connection.provider_slug.clone()),
                    ("tenant_id", connection.tenant_id.to_string()),
                ];
                counter!("sync_scheduler_jobs_scheduled_total", &metric_labels).increment(1);
                histogram!("sync_scheduler_jitter_seconds", &metric_labels)
                    .record(jitter_seconds as f64);
            }
            Err(err) if is_unique_violation(&err) => {
                stats.jobs_skipped_pending += 1;
                debug!(
                    connection_id = %connection.id,
                    "Interval job already exists; skipping enqueue"
                );
            }
            Err(err) => return Err(map_db_err("failed to insert incremental sync job", err)),
        }

        if metadata_dirty {
            self.persist_metadata(&mut txn, &connection, &metadata, now)
                .await?;
        }

        txn.commit()
            .await
            .map_err(|err| map_db_err("failed to commit scheduler transaction", err))?;

        Ok(())
    }

    async fn last_incremental_finished_at<C>(
        &self,
        connection_id: Uuid,
        executor: &mut C,
    ) -> Result<Option<DateTime<Utc>>, ApiError>
    where
        C: ConnectionTrait + Send,
    {
        let last_job = SyncJob::find()
            .filter(SyncJobColumn::ConnectionId.eq(connection_id))
            .filter(SyncJobColumn::JobType.eq("incremental"))
            .filter(SyncJobColumn::Status.eq("succeeded"))
            .order_by_desc(SyncJobColumn::FinishedAt)
            .limit(1)
            .one(executor)
            .await
            .map_err(|err| map_db_err("failed to load last incremental job", err))?;

        Ok(last_job
            .and_then(|job| job.finished_at)
            .map(|dt| dt.with_timezone(&Utc)))
    }

    async fn persist_metadata(
        &self,
        txn: &mut DatabaseTransaction,
        connection: &ConnectionModel,
        metadata: &ConnectionSyncMetadata,
        now: DateTime<Utc>,
    ) -> Result<(), ApiError> {
        let metadata_json = metadata.into_connection_metadata(connection.metadata.as_ref());
        let metadata_option = match metadata_json {
            serde_json::Value::Object(ref map) if map.is_empty() => None,
            value => Some(value),
        };

        let active = ConnectionActiveModel {
            id: Set(connection.id),
            metadata: Set(metadata_option),
            updated_at: Set(to_db_time(now)),
            ..Default::default()
        };

        active
            .update(txn)
            .await
            .map_err(|err| map_db_err("failed to persist connection metadata", err))?;

        Ok(())
    }
}

fn compute_due_times(
    metadata: &ConnectionSyncMetadata,
    base_interval_seconds: u64,
    last_finished: Option<DateTime<Utc>>,
    activation_reference: DateTime<Utc>,
    now: DateTime<Utc>,
) -> DueComputation {
    let base_interval = Duration::seconds(base_interval_seconds as i64);

    let mut next_due = metadata
        .next_run_at
        .or_else(|| last_finished.map(|finished| finished + base_interval))
        .unwrap_or(activation_reference + base_interval);

    let mut advanced = false;
    while next_due <= now {
        next_due += base_interval;
        advanced = true;
    }

    let job_due = if advanced {
        next_due - base_interval
    } else {
        next_due
    };

    let next_run_at = if advanced {
        next_due
    } else {
        next_due + base_interval
    };

    DueComputation {
        job_due,
        next_run_at,
        is_overdue: now > job_due,
    }
}

fn sample_jitter_seconds(config: &SchedulerConfig, base_interval_seconds: u64) -> u64 {
    let mut rng = rand::thread_rng();
    compute_jitter_seconds(config, base_interval_seconds, &mut rng)
}

fn compute_jitter_seconds<R: Rng + ?Sized>(
    config: &SchedulerConfig,
    base_interval_seconds: u64,
    rng: &mut R,
) -> u64 {
    let min = config.jitter_pct_min.max(0.0);
    let max = config.jitter_pct_max.max(min);

    if min == 0.0 && max == 0.0 {
        return 0;
    }

    let jitter_pct = if (max - min).abs() < f64::EPSILON {
        min
    } else {
        rng.gen_range(min..=max)
    };

    (base_interval_seconds as f64 * jitter_pct).round() as u64
}

fn is_unique_violation(err: &DbErr) -> bool {
    match err {
        DbErr::Exec(RuntimeErr::SqlxError(sea_orm::SqlxError::Database(db_err))) => {
            let code = db_err.code();
            let constraint = db_err.constraint();
            matches!(constraint, Some(INTERVAL_UNIQUE_INDEX))
                || matches!(code.as_deref(), Some("23505") | Some("2067"))
        }
        _ => false,
    }
}

fn to_db_time(dt: DateTime<Utc>) -> DateTime<FixedOffset> {
    DateTime::from_naive_utc_and_offset(
        dt.naive_utc(),
        FixedOffset::east_opt(0).expect("UTC offset"),
    )
}

fn map_db_err(context: &'static str, err: DbErr) -> ApiError {
    error!(error = ?err, context, "Database operation failed");
    ApiError::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        "INTERNAL_SERVER_ERROR",
        context,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{SeedableRng, rngs::mock::StepRng};
    use std::sync::Arc;

    use migration::{Migrator, MigratorTrait};
    use sea_orm::{Database, PaginatorTrait, Statement, Value};

    fn scheduler_config() -> SchedulerConfig {
        SchedulerConfig {
            tick_interval_seconds: 60,
            default_interval_seconds: 900,
            jitter_pct_min: 0.0,
            jitter_pct_max: 0.2,
            max_overridden_interval_seconds: 86400,
        }
    }

    #[test]
    fn jitter_respects_bounds() {
        let config = scheduler_config();
        let base_interval = 900;
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);

        for _ in 0..100 {
            let jitter = compute_jitter_seconds(&config, base_interval, &mut rng);
            assert!(jitter <= (base_interval as f64 * config.jitter_pct_max).round() as u64);
            assert!(jitter >= (base_interval as f64 * config.jitter_pct_min).round() as u64);
        }
    }

    #[test]
    fn compute_due_bootstrap() {
        let metadata = ConnectionSyncMetadata::default();
        let activation = DateTime::parse_from_rfc3339("2025-01-01T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let now = activation;
        let due = compute_due_times(&metadata, 900, None, activation, now);

        assert_eq!(due.job_due, activation + Duration::seconds(900));
        assert_eq!(due.next_run_at, activation + Duration::seconds(1800));
        assert!(!due.is_overdue);
    }

    #[test]
    fn compute_due_catch_up_advances_until_future() {
        let metadata = ConnectionSyncMetadata::default();
        let activation = DateTime::parse_from_rfc3339("2025-01-01T10:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let last_finished = Some(activation);
        let now = activation + Duration::minutes(20);
        let due = compute_due_times(&metadata, 900, last_finished, activation, now);

        assert_eq!(due.job_due, activation + Duration::minutes(15));
        assert_eq!(due.next_run_at, activation + Duration::minutes(30));
        assert!(due.is_overdue);
    }

    #[test]
    fn compute_due_steady_state_rolls_forward() {
        let metadata = ConnectionSyncMetadata {
            next_run_at: Some(
                DateTime::parse_from_rfc3339("2025-01-01T10:15:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
            ..Default::default()
        };
        let activation = DateTime::parse_from_rfc3339("2025-01-01T10:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let now = DateTime::parse_from_rfc3339("2025-01-01T10:16:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let due = compute_due_times(&metadata, 900, None, activation, now);

        assert_eq!(
            due.job_due,
            DateTime::parse_from_rfc3339("2025-01-01T10:15:00Z")
                .unwrap()
                .with_timezone(&Utc)
        );
        assert_eq!(
            due.next_run_at,
            DateTime::parse_from_rfc3339("2025-01-01T10:30:00Z")
                .unwrap()
                .with_timezone(&Utc)
        );
        assert!(due.is_overdue);
    }

    #[test]
    fn jitter_zero_when_bounds_zero() {
        let config = SchedulerConfig {
            jitter_pct_min: 0.0,
            jitter_pct_max: 0.0,
            ..scheduler_config()
        };
        let mut rng = StepRng::new(0, 1);
        let jitter = compute_jitter_seconds(&config, 600, &mut rng);
        assert_eq!(jitter, 0);
    }

    #[tokio::test]
    async fn catch_up_enqueues_job_and_updates_metadata() {
        let _ = tracing_subscriber::fmt::try_init();
        let db = Database::connect("sqlite::memory:")
            .await
            .expect("create in-memory db");
        Migrator::up(&db, None).await.expect("apply migrations");

        let backend = db.get_database_backend();
        let tenant_id = Uuid::new_v4();
        let provider_slug = "github";
        let connection_id = Uuid::new_v4();
        let now_anchor = Utc::now();

        db.execute(Statement::from_sql_and_values(
            backend,
            "INSERT INTO tenants (id, name) VALUES (?, ?)",
            vec![tenant_id.into(), "Test Tenant".into()],
        ))
        .await
        .expect("insert tenant");

        db.execute(Statement::from_sql_and_values(
            backend,
            "INSERT INTO providers (slug, display_name, auth_type) VALUES (?, ?, ?)",
            vec![provider_slug.into(), "GitHub".into(), "oauth2".into()],
        ))
        .await
        .expect("insert provider");

        let activation = now_anchor - Duration::minutes(45);
        let metadata = serde_json::json!({
            "sync": {
                "first_activated_at": activation.to_rfc3339(),
                "interval_seconds": 900
            }
        })
        .to_string();

        db.execute(Statement::from_sql_and_values(
            backend,
            "INSERT INTO connections (id, tenant_id, provider_slug, external_id, status, metadata) \
             VALUES (?, ?, ?, ?, ?, ?)",
            vec![
                Value::from(connection_id),
                Value::from(tenant_id),
                Value::from(provider_slug),
                Value::from("external-1"),
                Value::from("active"),
                Value::from(metadata.clone()),
            ],
        ))
        .await
        .expect("insert connection");

        let last_finished = now_anchor - Duration::minutes(30);
        let timestamp = last_finished.to_rfc3339();
        db.execute(Statement::from_sql_and_values(
            backend,
            "INSERT INTO sync_jobs (id, tenant_id, provider_slug, connection_id, job_type, status, priority, attempts, scheduled_at, finished_at, created_at, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            vec![
                Value::from(Uuid::new_v4()),
                Value::from(tenant_id),
                Value::from(provider_slug),
                Value::from(connection_id),
                Value::from("incremental"),
                Value::from("succeeded"),
                Value::from(30),
                Value::from(1),
                Value::from(timestamp.clone()),
                Value::from(timestamp.clone()),
                Value::from(timestamp.clone()),
                Value::from(timestamp.clone()),
            ],
        ))
        .await
        .expect("insert historical job");

        let connection_count = Connection::find()
            .count(&db)
            .await
            .expect("connection count");
        assert_eq!(connection_count, 1, "connection not inserted");

        let mut config = AppConfig::default();
        config.scheduler.jitter_pct_min = 0.0;
        config.scheduler.jitter_pct_max = 0.0;

        let scheduler = SyncScheduler::new(Arc::new(config), Arc::new(db.clone()));
        scheduler.tick().await.expect("first tick succeeds");

        let queued_jobs = SyncJob::find()
            .filter(SyncJobColumn::ConnectionId.eq(connection_id))
            .filter(SyncJobColumn::Status.eq("queued"))
            .all(&db)
            .await
            .expect("fetch queued jobs");
        assert_eq!(queued_jobs.len(), 1);
        let scheduled_at = queued_jobs[0].scheduled_at.with_timezone(&Utc);
        assert!(
            scheduled_at <= Utc::now() + Duration::seconds(1),
            "scheduled_at too far in future: {}",
            scheduled_at
        );

        let connection = Connection::find_by_id(connection_id)
            .one(&db)
            .await
            .expect("fetch connection")
            .expect("connection exists");
        let metadata =
            ConnectionSyncMetadata::from_connection_metadata(connection.metadata.as_ref());
        let next_run_at = metadata
            .next_run_at
            .expect("next_run_at should be recorded");
        let diff = next_run_at - scheduled_at;
        assert!(
            (diff.num_seconds() - 900).abs() <= 1,
            "unexpected interval advancement: {} seconds",
            diff.num_seconds()
        );
        assert_eq!(metadata.last_jitter_seconds, Some(0));

        scheduler.tick().await.expect("second tick succeeds");
        let queued_jobs_after = SyncJob::find()
            .filter(SyncJobColumn::ConnectionId.eq(connection_id))
            .filter(SyncJobColumn::Status.eq("queued"))
            .all(&db)
            .await
            .expect("fetch queued jobs after second tick");
        assert_eq!(queued_jobs_after.len(), 1, "no duplicate interval jobs");
    }
}
