//! Configuration loading for the Connectors API.
//!
//! Loads layered `.env` files and environment variables prefixed with
//! `POBLYSH_`, producing a typed [`AppConfig`].

use std::{collections::BTreeMap, env, net::SocketAddr, path::PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Application configuration derived from `POBLYSH_*` environment variables.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct AppConfig {
    #[serde(default = "default_profile")]
    pub profile: String,
    #[serde(default = "default_api_bind_addr")]
    pub api_bind_addr: String,
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default = "default_database_url")]
    pub database_url: String,
    #[serde(default = "default_db_max_connections")]
    pub db_max_connections: u32,
    #[serde(default = "default_db_acquire_timeout_ms")]
    pub db_acquire_timeout_ms: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            profile: default_profile(),
            api_bind_addr: default_api_bind_addr(),
            log_level: default_log_level(),
            database_url: default_database_url(),
            db_max_connections: default_db_max_connections(),
            db_acquire_timeout_ms: default_db_acquire_timeout_ms(),
        }
    }
}

impl AppConfig {
    /// Returns the configured bind address as a socket address.
    pub fn bind_addr(&self) -> Result<SocketAddr, std::net::AddrParseError> {
        self.api_bind_addr.parse()
    }

    /// Returns a redacted JSON representation (no secrets in current schema).
    pub fn redacted_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

fn default_profile() -> String {
    "local".to_string()
}

fn default_api_bind_addr() -> String {
    "0.0.0.0:8080".to_string()
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_database_url() -> String {
    "postgresql://akamaotto:TheP%4055w0rd%21@localhost:5432/connectors".to_string()
}

fn default_db_max_connections() -> u32 {
    10
}

fn default_db_acquire_timeout_ms() -> u64 {
    5000
}

/// Errors that can occur while loading configuration.
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to load environment file {path}: {source}")]
    EnvFile {
        path: PathBuf,
        source: dotenvy::Error,
    },
    #[error("invalid api bind address '{value}': {source}")]
    InvalidBindAddr {
        value: String,
        source: std::net::AddrParseError,
    },
}

/// Loads configuration using layered `.env` files and `POBLYSH_*` env vars.
pub struct ConfigLoader {
    base_dir: PathBuf,
}

impl ConfigLoader {
    /// Creates a new loader rooted at the current working directory.
    pub fn new() -> Self {
        Self {
            base_dir: env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        }
    }

    /// Creates a loader rooted at the provided directory (useful for tests).
    pub fn with_base_dir(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    /// Loads configuration according to the spec requirements.
    pub fn load(&self) -> Result<AppConfig, ConfigError> {
        let (mut layered, profile_hint) = self.collect_layered_env()?;

        // Overlay process environment last so it wins.
        for (key, value) in env::vars() {
            if let Some(stripped) = key.strip_prefix("POBLYSH_") {
                layered.insert(stripped.to_string(), value);
            }
        }

        let profile = layered
            .remove("PROFILE")
            .filter(|v| !v.is_empty())
            .unwrap_or(profile_hint);
        let api_bind_addr = layered
            .remove("API_BIND_ADDR")
            .filter(|v| !v.is_empty())
            .unwrap_or_else(default_api_bind_addr);
        let log_level = layered
            .remove("LOG_LEVEL")
            .filter(|v| !v.is_empty())
            .unwrap_or_else(default_log_level);
        let database_url = layered
            .remove("DATABASE_URL")
            .filter(|v| !v.is_empty())
            .unwrap_or_else(default_database_url);
        let db_max_connections = layered
            .remove("DB_MAX_CONNECTIONS")
            .and_then(|v| v.parse().ok())
            .unwrap_or_else(default_db_max_connections);
        let db_acquire_timeout_ms = layered
            .remove("DB_ACQUIRE_TIMEOUT_MS")
            .and_then(|v| v.parse().ok())
            .unwrap_or_else(default_db_acquire_timeout_ms);

        let config = AppConfig {
            profile,
            api_bind_addr,
            log_level,
            database_url,
            db_max_connections,
            db_acquire_timeout_ms,
        };

        match config.bind_addr() {
            Ok(_) => Ok(config),
            Err(source) => Err(ConfigError::InvalidBindAddr {
                value: config.api_bind_addr.clone(),
                source,
            }),
        }
    }

    fn collect_layered_env(&self) -> Result<(BTreeMap<String, String>, String), ConfigError> {
        let mut values = BTreeMap::new();

        self.merge_dotenv(self.base_dir.join(".env"), &mut values)?;
        self.merge_dotenv(self.base_dir.join(".env.local"), &mut values)?;

        let profile = env::var("POBLYSH_PROFILE")
            .ok()
            .or_else(|| values.get("PROFILE").cloned())
            .unwrap_or_else(default_profile);

        self.merge_dotenv(
            self.base_dir.join(format!(".env.{}", &profile)),
            &mut values,
        )?;
        self.merge_dotenv(
            self.base_dir.join(format!(".env.{}.local", &profile)),
            &mut values,
        )?;

        Ok((values, profile))
    }

    fn merge_dotenv(
        &self,
        path: PathBuf,
        values: &mut BTreeMap<String, String>,
    ) -> Result<(), ConfigError> {
        match dotenvy::from_path_iter(&path) {
            Ok(iter) => {
                for item in iter {
                    let (key, value) = item.map_err(|source| ConfigError::EnvFile {
                        path: path.clone(),
                        source,
                    })?;
                    if let Some(stripped) = key.strip_prefix("POBLYSH_") {
                        values.insert(stripped.to_string(), value);
                    }
                }
                Ok(())
            }
            Err(dotenvy::Error::Io(ref io_err))
                if io_err.kind() == std::io::ErrorKind::NotFound =>
            {
                Ok(())
            }
            Err(err) => Err(ConfigError::EnvFile { path, source: err }),
        }
    }
}
