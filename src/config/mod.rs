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
    #[serde(default = "default_log_format")]
    pub log_format: String,
    #[serde(default = "default_database_url")]
    pub database_url: String,
    #[serde(default = "default_db_max_connections")]
    pub db_max_connections: u32,
    #[serde(default = "default_db_acquire_timeout_ms")]
    pub db_acquire_timeout_ms: u64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub operator_tokens: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crypto_key: Option<Vec<u8>>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            profile: default_profile(),
            api_bind_addr: default_api_bind_addr(),
            log_level: default_log_level(),
            log_format: default_log_format(),
            database_url: default_database_url(),
            db_max_connections: default_db_max_connections(),
            db_acquire_timeout_ms: default_db_acquire_timeout_ms(),
            operator_tokens: Vec::new(),
            crypto_key: None,
        }
    }
}

impl AppConfig {
    /// Returns the configured bind address as a socket address.
    pub fn bind_addr(&self) -> Result<SocketAddr, std::net::AddrParseError> {
        self.api_bind_addr.parse()
    }

    /// Returns a redacted JSON representation (secrets are redacted).
    pub fn redacted_json(&self) -> serde_json::Result<String> {
        let mut config = self.clone();
        // Redact operator tokens for security
        if !config.operator_tokens.is_empty() {
            config.operator_tokens = vec!["[REDACTED]".to_string()];
        }
        // Redact crypto key for security
        if config.crypto_key.is_some() {
            config.crypto_key = Some(b"[REDACTED]".to_vec());
        }
        serde_json::to_string_pretty(&config)
    }

    /// Validates the configuration, returning an error if required settings are missing.
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate crypto key
        if let Some(ref key) = self.crypto_key {
            if key.len() != 32 {
                return Err(ConfigError::InvalidCryptoKeyLength { length: key.len() });
            }
        } else {
            return Err(ConfigError::MissingCryptoKey);
        }

        // For local and test profiles, require at least one operator token
        if (self.profile == "local" || self.profile == "test") && self.operator_tokens.is_empty() {
            return Err(ConfigError::MissingOperatorTokens);
        }

        // For production profiles, also require at least one operator token
        if !matches!(self.profile.as_str(), "local" | "test") && self.operator_tokens.is_empty() {
            return Err(ConfigError::MissingOperatorTokens);
        }

        Ok(())
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

fn default_log_format() -> String {
    "json".to_string()
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
    #[error("no operator tokens configured; set POBLYSH_OPERATOR_TOKEN or POBLYSH_OPERATOR_TOKENS")]
    MissingOperatorTokens,
    #[error("crypto key is missing; set POBLYSH_CRYPTO_KEY environment variable")]
    MissingCryptoKey,
    #[error("crypto key is invalid base64: {error}")]
    InvalidCryptoKeyBase64 { error: String },
    #[error("crypto key must decode to exactly 32 bytes, got {length} bytes")]
    InvalidCryptoKeyLength { length: usize },
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
        let log_format = layered
            .remove("LOG_FORMAT")
            .filter(|v| !v.is_empty())
            .unwrap_or_else(default_log_format);
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

        // Handle operator tokens - support both single token and comma-separated list
        let operator_tokens = if let Some(tokens) = layered.remove("OPERATOR_TOKENS") {
            // POBLYSH_OPERATOR_TOKENS (comma-separated)
            tokens
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        } else if let Some(token) = layered.remove("OPERATOR_TOKEN") {
            // POBLYSH_OPERATOR_TOKEN (single)
            vec![token]
        } else {
            Vec::new()
        };

        // Parse and validate crypto key
        let crypto_key = if let Some(key_str) = layered.remove("CRYPTO_KEY") {
            // Decode base64 e key, Engine as _
            {
                use base64::{Engine as _, engine::general_purpose};
                general_purpose::STANDARD.decode(&key_str).map_err(|e| {
                    ConfigError::InvalidCryptoKeyBase64 {
                        error: e.to_string(),
                    }
                })?
            }
        } else {
            Vec::new()
        };

        let config = AppConfig {
            profile,
            api_bind_addr,
            log_level,
            log_format,
            database_url,
            db_max_connections,
            db_acquire_timeout_ms,
            operator_tokens,
            crypto_key: if crypto_key.is_empty() {
                None
            } else {
                Some(crypto_key)
            },
        };

        // Validate configuration
        config.validate()?;

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

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}
