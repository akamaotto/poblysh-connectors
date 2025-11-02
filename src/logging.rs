
use crate::config::AppConfig;
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter, Registry};

pub fn init_subscriber(config: &AppConfig) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.log_level));

    let formatter = fmt::layer().json();

    let subscriber = Registry::default().with(filter).with(formatter);

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set global default subscriber");
}
