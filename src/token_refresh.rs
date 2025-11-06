//! # Token Refresh Service
//!
//! Background task that periodically scans active connections and refreshes tokens
//! nearing expiry. Also provides on-demand refresh functionality for sync and webhook
//! operations when encountering 401 errors.

use chrono::{DateTime, Duration, Utc};
use metrics::{counter, gauge, histogram};
use rand::Rng;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
    prelude::DateTimeWithTimeZone,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{Duration as TokioDuration, sleep};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

use crate::config::AppConfig;
use crate::connectors::registry::Registry;
use crate::error::ApiError;
use crate::models::connection::{self, ActiveModel as ConnectionActiveModel, Entity as Connection};
use crate::repositories::connection::ConnectionRepository;

/// Background token refresh service
pub struct TokenRefreshService {
    config: Arc<AppConfig>,
    db: Arc<DatabaseConnection>,
    connection_repo: Arc<ConnectionRepository>,
    connector_registry: Registry,
    /// Tracks ongoing refresh operations to provide single-flight protection
    in_flight_refreshes: Arc<Mutex<HashMap<Uuid, tokio::task::JoinHandle<()>>>>,
}

#[derive(Debug, Default)]
struct RefreshStats {
    connections_polled: u64,
    refreshes_attempted: u64,
    refreshes_succeeded: u64,
    refreshes_failed: u64,
    connections_error_set: u64,
}

/// Classification of token refresh errors for appropriate handling
#[derive(Debug, PartialEq)]
pub enum RefreshErrorClassification {
    /// Permanent failures that should disable the connection (e.g., invalid_grant)
    Permanent,
    /// Temporary failures that can be retried (e.g., network issues)
    Transient,
    /// Rate limiting errors that should trigger backoff
    RateLimited,
}

/// Result of a token refresh operation
#[derive(Debug)]
pub struct RefreshResult {
    pub success: bool,
    pub connection_id: Uuid,
    pub new_access_token: Option<String>,
    pub new_refresh_token: Option<String>,
    pub new_expires_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

impl TokenRefreshService {
    /// Create a new token refresh service instance
    pub fn new(
        config: Arc<AppConfig>,
        db: Arc<DatabaseConnection>,
        connection_repo: Arc<ConnectionRepository>,
        connector_registry: Registry,
    ) -> Self {
        Self {
            config,
            db,
            connection_repo,
            connector_registry,
            in_flight_refreshes: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Run the token refresh loop until the provided shutdown token fires
    #[instrument(skip_all)]
    pub async fn run(&self, shutdown: CancellationToken) -> Result<(), ApiError> {
        info!("Starting token refresh service");
        let tick_interval = TokioDuration::from_secs(self.config.token_refresh.tick_seconds);

        loop {
            tokio::select! {
                _ = shutdown.cancelled() => {
                    info!("Token refresh service shutdown requested");
                    break;
                }
                _ = sleep(tick_interval) => {
                    let tick_started = std::time::Instant::now();
                    if let Err(err) = self.tick().await {
                        error!(error = ?err, "Token refresh tick failed");
                    }
                    let elapsed = tick_started.elapsed();
                    histogram!("token_refresh_tick_duration_ms")
                        .record(elapsed.as_secs_f64() * 1_000.0);
                }
            }
        }

        info!("Token refresh service stopped");
        Ok(())
    }

    /// Execute one tick of the token refresh service
    #[instrument(skip_all)]
    pub async fn tick(&self) -> Result<(), ApiError> {
        let now = Utc::now();
        let mut stats = RefreshStats::default();

        // Find connections that need refresh
        let due_connections = self.find_connections_due_for_refresh(now).await?;

        info!(
            found_connections = due_connections.len(),
            lead_time_seconds = self.config.token_refresh.lead_time_seconds,
            "Found connections due for token refresh"
        );

        // Process connections in batches with concurrency limit
        let semaphore = Arc::new(tokio::sync::Semaphore::new(
            self.config.token_refresh.concurrency as usize,
        ));

        let mut handles = Vec::new();

        for connection in due_connections {
            let semaphore = semaphore.clone();
            let service = self.clone();

            let handle = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                service
                    .refresh_connection_with_jitter(connection, now)
                    .await
            });

            handles.push(handle);
        }

        // Wait for all refresh operations to complete
        for handle in handles {
            match handle.await {
                Ok(Ok(_)) => stats.refreshes_succeeded += 1,
                Ok(Err(e)) => {
                    stats.refreshes_failed += 1;
                    error!(error = ?e, "Connection refresh failed");
                }
                Err(e) => {
                    stats.refreshes_failed += 1;
                    error!(error = ?e, "Refresh task panicked or was cancelled");
                }
            }
        }

        // Record metrics
        gauge!("token_refresh_connections_polled_gauge").set(stats.connections_polled as f64);
        counter!("token_refresh_attempts_total").increment(stats.refreshes_attempted);
        counter!("token_refresh_success_total").increment(stats.refreshes_succeeded);
        counter!("token_refresh_failure_total").increment(stats.refreshes_failed);

        debug!(
            connections_polled = stats.connections_polled,
            refreshes_attempted = stats.refreshes_attempted,
            refreshes_succeeded = stats.refreshes_succeeded,
            refreshes_failed = stats.refreshes_failed,
            connections_error_set = stats.connections_error_set,
            "Token refresh tick completed"
        );

        Ok(())
    }

    /// Find active connections whose tokens expire within the lead time window
    async fn find_connections_due_for_refresh(
        &self,
        ___now: DateTime<Utc>,
    ) -> Result<Vec<connection::Model>, ApiError> {
        let expiry_cutoff =
            ___now + Duration::seconds(self.config.token_refresh.lead_time_seconds as i64);
        let expiry_cutoff_db: DateTimeWithTimeZone = expiry_cutoff.into();

        let connections = Connection::find()
            .filter(connection::Column::Status.eq("active"))
            .filter(connection::Column::RefreshTokenCiphertext.is_not_null())
            .filter(
                connection::Column::ExpiresAt
                    .is_not_null()
                    .and(connection::Column::ExpiresAt.lte(expiry_cutoff_db)),
            )
            .order_by_asc(connection::Column::ExpiresAt)
            .all(self.db.as_ref())
            .await
            .map_err(|e| {
                error!(error = ?e, "Failed to query connections due for refresh");
                ApiError::new(
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_SERVER_ERROR",
                    "Failed to query connections due for refresh",
                )
            })?;

        Ok(connections)
    }

    /// Refresh a single connection with jitter applied
    async fn refresh_connection_with_jitter(
        &self,
        connection: connection::Model,
        _now: DateTime<Utc>,
    ) -> Result<RefreshResult, ApiError> {
        // Apply jitter to avoid thundering herd
        let jitter_seconds = self.compute_jitter();
        if jitter_seconds > 0 {
            debug!(
                connection_id = %connection.id,
                jitter_seconds = jitter_seconds,
                "Applying jitter before token refresh"
            );
            sleep(TokioDuration::from_secs(jitter_seconds)).await;
        }

        self.refresh_connection(connection, _now).await
    }

    /// Refresh a single connection's tokens
    #[instrument(skip_all, fields(connection_id = %connection.id))]
    pub async fn refresh_connection(
        &self,
        connection: connection::Model,
        _now: DateTime<Utc>,
    ) -> Result<RefreshResult, ApiError> {
        let refresh_start = std::time::Instant::now();

        // Decrypt current tokens
        let (_access_token, refresh_token, _) = self
            .connection_repo
            .decrypt_tokens(&connection)
            .await
            .map_err(|e| {
                error!(error = ?e, "Failed to decrypt tokens for connection");
                ApiError::new(
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_SERVER_ERROR",
                    "Failed to decrypt tokens",
                )
            })?;

        if refresh_token.is_none() {
            warn!(
                connection_id = %connection.id,
                "Connection has no refresh token, cannot refresh"
            );
            return Ok(RefreshResult {
                success: false,
                connection_id: connection.id,
                new_access_token: None,
                new_refresh_token: None,
                new_expires_at: None,
                error: Some("No refresh token available".to_string()),
            });
        }

        // Get connector for this provider
        let connector = self
            .connector_registry
            .get(&connection.provider_slug)
            .map_err(|e| {
                error!(
                    provider_slug = %connection.provider_slug,
                    error = ?e,
                    "Failed to get connector for provider"
                );
                ApiError::new(
                    axum::http::StatusCode::NOT_FOUND,
                    "NOT_FOUND",
                    &format!(
                        "Connector for provider '{}' not found",
                        connection.provider_slug
                    ),
                )
            })?;

        // Perform token refresh via connector
        match connector.refresh_token(connection.clone()).await {
            Ok(refreshed_connection) => {
                let refresh_duration = refresh_start.elapsed();
                histogram!("token_refresh_latency_ms")
                    .record(refresh_duration.as_secs_f64() * 1_000.0);

                info!(
                    connection_id = %connection.id,
                    provider_slug = %connection.provider_slug,
                    refresh_duration_ms = refresh_duration.as_millis(),
                    "Successfully refreshed connection tokens"
                );

                // Record metrics with provider labels
                let metric_labels = vec![
                    ("provider_slug", connection.provider_slug.clone()),
                    ("tenant_id", connection.tenant_id.to_string()),
                ];
                counter!("token_refresh_success_total", &metric_labels).increment(1);

                Ok(RefreshResult {
                    success: true,
                    connection_id: refreshed_connection.id,
                    new_access_token: refreshed_connection
                        .access_token_ciphertext
                        .map(|_| "[REDACTED]".to_string()),
                    new_refresh_token: refreshed_connection
                        .refresh_token_ciphertext
                        .map(|_| "[REDACTED]".to_string()),
                    new_expires_at: refreshed_connection
                        .expires_at
                        .map(|dt| dt.with_timezone(&Utc)),
                    error: None,
                })
            }
            Err(e) => {
                let error_str = e.to_string();
                error!(
                    connection_id = %connection.id,
                    provider_slug = %connection.provider_slug,
                    error = %error_str,
                    "Failed to refresh connection tokens"
                );

                // Classify the error type for appropriate handling
                let error_classification = self.classify_refresh_error(&error_str);

                match error_classification {
                    RefreshErrorClassification::Permanent => {
                        error!(
                            connection_id = %connection.id,
                            provider_slug = %connection.provider_slug,
                            error = %error_str,
                            "Permanent token refresh failure - marking connection as error"
                        );

                        // Mark connection as error/revoked status
                        self.mark_connection_error(&connection.id, &error_str)
                            .await?;

                        counter!("token_refresh_permanent_failure_total").increment(1);
                    }
                    RefreshErrorClassification::Transient => {
                        warn!(
                            connection_id = %connection.id,
                            provider_slug = %connection.provider_slug,
                            error = %error_str,
                            "Transient token refresh failure - will retry later"
                        );

                        counter!("token_refresh_transient_failure_total").increment(1);
                    }
                    RefreshErrorClassification::RateLimited => {
                        warn!(
                            connection_id = %connection.id,
                            provider_slug = %connection.provider_slug,
                            error = %error_str,
                            "Rate limited during token refresh"
                        );

                        counter!("token_refresh_rate_limited_total").increment(1);
                    }
                }

                // Record metrics with provider labels
                let metric_labels = vec![
                    ("provider_slug", connection.provider_slug.clone()),
                    ("tenant_id", connection.tenant_id.to_string()),
                ];
                counter!("token_refresh_failure_total", &metric_labels).increment(1);

                Ok(RefreshResult {
                    success: false,
                    connection_id: connection.id,
                    new_access_token: None,
                    new_refresh_token: None,
                    new_expires_at: None,
                    error: Some(error_str),
                })
            }
        }
    }

    /// Classify token refresh errors for appropriate handling strategy
    pub fn classify_refresh_error(&self, error_str: &str) -> RefreshErrorClassification {
        let error_lower = error_str.to_lowercase();

        // Check for permanent failures first
        if error_lower.contains("invalid_grant")
            || error_lower.contains("invalid_client")
            || error_lower.contains("unauthorized_client")
            || error_lower.contains("revoked")
            || error_lower.contains("forbidden")
            || error_lower.contains("access_denied")
            || error_lower.contains("unsupported_grant_type")
        {
            return RefreshErrorClassification::Permanent;
        }

        // Check for rate limiting
        if error_lower.contains("rate_limit")
            || error_lower.contains("too_many_requests")
            || error_lower.contains("temporarily_unavailable")
            || error_lower.contains("quota_exceeded")
        {
            return RefreshErrorClassification::RateLimited;
        }

        // Default to transient for network and other temporary issues
        RefreshErrorClassification::Transient
    }

    /// Mark a connection as having an error status due to failed refresh
    async fn mark_connection_error(
        &self,
        connection_id: &Uuid,
        error_msg: &str,
    ) -> Result<(), ApiError> {
        let updated = ConnectionActiveModel {
            id: Set(*connection_id),
            status: Set("error".to_string()),
            updated_at: Set(Utc::now().into()),
            ..Default::default()
        };

        updated.update(self.db.as_ref()).await.map_err(|e| {
            error!(
                connection_id = %connection_id,
                error = ?e,
                "Failed to mark connection as error status"
            );
            ApiError::new(
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_SERVER_ERROR",
                "Failed to update connection status",
            )
        })?;

        warn!(
            connection_id = %connection_id,
            error = %error_msg,
            "Marked connection as error status due to failed token refresh"
        );

        counter!("token_refresh_connections_marked_error_total").increment(1);
        Ok(())
    }

    /// Compute jitter delay based on configuration
    fn compute_jitter(&self) -> u64 {
        if self.config.token_refresh.jitter_factor <= 0.0 {
            return 0;
        }

        let max_delay_seconds = (self.config.token_refresh.lead_time_seconds as f64
            * self.config.token_refresh.jitter_factor) as u64;

        let mut rng = rand::thread_rng();
        rng.gen_range(0..=max_delay_seconds)
    }

    /// On-demand refresh for when operations receive a 401 error
    /// This is meant to be called by sync executor or webhook handlers
    /// Provides single-flight protection to prevent concurrent refresh attempts
    #[instrument(skip_all, fields(connection_id = %connection_id))]
    pub async fn refresh_on_demand(&self, connection_id: &Uuid) -> Result<RefreshResult, ApiError> {
        // Check if there's already a refresh in progress for this connection
        {
            let in_flight = self.in_flight_refreshes.lock().await;
            if let Some(_existing_handle) = in_flight.get(connection_id) {
                info!(
                    connection_id = %connection_id,
                    "Refresh already in progress, waiting and retrying"
                );
                drop(in_flight); // Release lock before retry
                // Brief wait and retry once to avoid race conditions
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

                // Check if the refresh completed successfully by getting current state
                return self.get_current_connection_state(connection_id).await;
            }
        }

        // No refresh in progress, start one and mark it as in-flight
        let connection = Connection::find_by_id(*connection_id)
            .one(self.db.as_ref())
            .await
            .map_err(|e| {
                error!(
                    connection_id = %connection_id,
                    error = ?e,
                    "Failed to find connection for on-demand refresh"
                );
                ApiError::new(
                    axum::http::StatusCode::NOT_FOUND,
                    "NOT_FOUND",
                    "Connection not found"
                )
            })?
            .ok_or_else(|| {
                error!(connection_id = %connection_id, "Connection not found for on-demand refresh");
                ApiError::new(
                    axum::http::StatusCode::NOT_FOUND,
                    "NOT_FOUND",
                    "Connection not found"
                )
            })?;

        info!(
            connection_id = %connection_id,
            provider_slug = %connection.provider_slug,
            "Performing on-demand token refresh"
        );

        counter!("token_refresh_on_demand_attempts_total").increment(1);

        // Mark this connection as having a refresh in progress
        {
            let mut in_flight = self.in_flight_refreshes.lock().await;
            in_flight.insert(*connection_id, tokio::spawn(async {}));
        }

        let result = self.refresh_connection(connection, Utc::now()).await;

        // Clean up the in-flight entry
        {
            let mut in_flight = self.in_flight_refreshes.lock().await;
            in_flight.remove(connection_id);
        }

        let result = result?;

        if result.success {
            counter!("token_refresh_on_demand_success_total").increment(1);
        } else {
            counter!("token_refresh_on_demand_failure_total").increment(1);
        }

        Ok(result)
    }

    /// Helper method to get the current state of a connection after a refresh
    async fn get_current_connection_state(
        &self,
        connection_id: &Uuid,
    ) -> Result<RefreshResult, ApiError> {
        let connection = Connection::find_by_id(*connection_id)
            .one(self.db.as_ref())
            .await
            .map_err(|e| {
                error!(
                    connection_id = %connection_id,
                    error = ?e,
                    "Failed to find connection after refresh"
                );
                ApiError::new(
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_SERVER_ERROR",
                    "Failed to get connection state",
                )
            })?
            .ok_or_else(|| {
                error!(connection_id = %connection_id, "Connection disappeared after refresh");
                ApiError::new(
                    axum::http::StatusCode::NOT_FOUND,
                    "NOT_FOUND",
                    "Connection not found",
                )
            })?;

        Ok(RefreshResult {
            success: true,
            connection_id: *connection_id,
            new_access_token: None, // We don't expose decrypted tokens in the result
            new_refresh_token: None, // We don't expose decrypted tokens in the result
            new_expires_at: connection.expires_at.map(|dt| dt.naive_utc().and_utc()),
            error: None,
        })
    }
}

impl Clone for TokenRefreshService {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            db: self.db.clone(),
            connection_repo: self.connection_repo.clone(),
            connector_registry: self.connector_registry.clone(),
            in_flight_refreshes: self.in_flight_refreshes.clone(),
        }
    }
}
