//! Telemetry utilities for request-scoped tracing metadata and global subscriber management.

use std::any::type_name_of_val;
use std::sync::atomic::{AtomicBool, Ordering};

use log::LevelFilter;
use thiserror::Error;
use tokio::task_local;
use tracing_log::LogTracer;
use tracing_subscriber::{
    EnvFilter, fmt,
    layer::Layer,
    layer::SubscriberExt,
    util::{SubscriberInitExt, TryInitError},
};

use crate::config::AppConfig;

/// Trace context containing request correlation ID.
#[derive(Debug, Clone)]
pub struct TraceContext {
    pub trace_id: String,
}

task_local! {
    static ACTIVE_TRACE_CONTEXT: TraceContext;
}

/// Errors that can occur while initializing global telemetry.
#[derive(Debug, Error)]
pub enum TelemetryInitError {
    #[error("failed to install log tracer bridge: {0}")]
    LogTracer(#[from] log::SetLoggerError),
    #[error("failed to install tracing subscriber: {0}")]
    Subscriber(#[from] TryInitError),
}

static TELEMETRY_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Initialize global tracing/logging exactly once, wiring `log::` macros into the tracing pipeline.
pub fn init_tracing(config: &AppConfig) -> Result<(), TelemetryInitError> {
    if TELEMETRY_INITIALIZED
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return Ok(());
    }

    // Install log bridge first so legacy `log::` macros route through tracing.
    if let Err(err) = LogTracer::builder()
        .with_max_level(LevelFilter::Trace)
        .init()
    {
        // If a LogTracer is already registered (e.g., by tests or another component),
        // treat this as success; otherwise surface the error.
        let logger_type = type_name_of_val(log::logger());
        if !logger_type.contains("LogTracer") {
            eprintln!(
                "Warning: Failed to install log tracer bridge: {}. legacy `log::` macros will not emit structured tracing events.",
                err
            );
        }
    }

    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.log_level));

    let fmt_layer = match config.log_format.as_str() {
        "pretty" => fmt::layer().pretty().boxed(),
        _ => fmt::layer().json().boxed(),
    };

    if let Err(err) = tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .try_init()
    {
        TELEMETRY_INITIALIZED.store(false, Ordering::SeqCst);
        eprintln!(
            "Warning: Failed to set global tracing subscriber: {}. Default subscriber remains in effect.",
            err
        );
    }

    Ok(())
}

/// Execute `future` within the provided trace context, making it available through task-local
/// storage for the duration of the request.
pub async fn with_trace_context<Fut, R>(context: TraceContext, future: Fut) -> R
where
    Fut: std::future::Future<Output = R>,
{
    ACTIVE_TRACE_CONTEXT.scope(context, future).await
}

/// Get the currently active trace ID, if one has been set for the running task.
pub fn current_trace_id() -> Option<String> {
    ACTIVE_TRACE_CONTEXT
        .try_with(|ctx| ctx.trace_id.clone())
        .ok()
}
