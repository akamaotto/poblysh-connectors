//! Configuration loading for the Connectors API.
//!
//! Loads layered `.env` files and environment variables prefixed with
//! `POBLYSH_`, producing a typed [`AppConfig`].

use std::{collections::BTreeMap, env, net::SocketAddr, path::PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;

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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub webhook_github_secret: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub webhook_slack_signing_secret: Option<String>,
    #[serde(default = "default_webhook_slack_tolerance_seconds")]
    pub webhook_slack_tolerance_seconds: u64,
    #[serde(default)]
    pub scheduler: SchedulerConfig,
    #[serde(default)]
    pub rate_limit_policy: RateLimitPolicyConfig,
}

/// Scheduler-specific configuration parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct SchedulerConfig {
    #[serde(default = "default_sync_scheduler_tick_interval_seconds")]
    pub tick_interval_seconds: u64,
    #[serde(default = "default_sync_scheduler_default_interval_seconds")]
    pub default_interval_seconds: u64,
    #[serde(default = "default_sync_scheduler_jitter_pct_min")]
    pub jitter_pct_min: f64,
    #[serde(default = "default_sync_scheduler_jitter_pct_max")]
    pub jitter_pct_max: f64,
    #[serde(default = "default_sync_scheduler_max_overridden_interval_seconds")]
    pub max_overridden_interval_seconds: u64,
}

/// Rate limit policy configuration for handling provider rate limits
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct RateLimitPolicyConfig {
    /// Base retry interval in seconds (default: 5)
    ///
    /// The starting backoff time when a rate limit is encountered.
    /// Subsequent retries use exponential backoff: base_seconds * 2^attempts.
    ///
    /// Environment variable: `POBLYSH_RATE_LIMIT_BASE_SECONDS`
    #[serde(default = "default_rate_limit_base_seconds")]
    #[schema(example = 5)]
    pub base_seconds: u64,

    /// Maximum retry interval in seconds (default: 900)
    ///
    /// Upper bound for exponential backoff calculations to prevent
    /// excessively long retry delays. Must be >= base_seconds.
    ///
    /// Environment variable: `POBLYSH_RATE_LIMIT_MAX_SECONDS`
    #[serde(default = "default_rate_limit_max_seconds")]
    #[schema(example = 900)]
    pub max_seconds: u64,

    /// Jitter factor for distributed systems (default: 0.1, range: 0.0-1.0)
    ///
    /// Random factor applied to backoff calculations to prevent thundering
    /// herd problems when multiple instances retry simultaneously.
    /// Formula: backoff * (1 ± jitter_factor)
    ///
    /// Environment variable: `POBLYSH_RATE_LIMIT_JITTER_FACTOR`
    #[serde(default = "default_rate_limit_jitter_factor")]
    #[schema(example = 0.1, minimum = 0.0, maximum = 1.0)]
    pub jitter_factor: f64,

    /// Provider-specific rate limit policy overrides
    ///
    /// Allows fine-tuning rate limits for specific providers that may have
    /// different rate limit behaviors or requirements.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub provider_overrides: BTreeMap<String, RateLimitProviderOverride>,
}

/// Provider-specific rate limit policy overrides
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct RateLimitProviderOverride {
    /// Override for base retry interval for this provider
    ///
    /// Environment variable: `POBLYSH_RATE_LIMIT_PROVIDER_OVERRIDE_{PROVIDER}_BASE_SECONDS`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(example = 10)]
    pub base_seconds: Option<u64>,

    /// Override for maximum retry interval for this provider
    ///
    /// Environment variable: `POBLYSH_RATE_LIMIT_PROVIDER_OVERRIDE_{PROVIDER}_MAX_SECONDS`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(example = 1800)]
    pub max_seconds: Option<u64>,

    /// Override for jitter factor for this provider
    ///
    /// Environment variable: `POBLYSH_RATE_LIMIT_PROVIDER_OVERRIDE_{PROVIDER}_JITTER_FACTOR`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(example = 0.2, minimum = 0.0, maximum = 1.0)]
    pub jitter_factor: Option<f64>,
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
            webhook_github_secret: None,
            webhook_slack_signing_secret: None,
            webhook_slack_tolerance_seconds: default_webhook_slack_tolerance_seconds(),
            scheduler: SchedulerConfig::default(),
            rate_limit_policy: RateLimitPolicyConfig::default(),
        }
    }
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            tick_interval_seconds: default_sync_scheduler_tick_interval_seconds(),
            default_interval_seconds: default_sync_scheduler_default_interval_seconds(),
            jitter_pct_min: default_sync_scheduler_jitter_pct_min(),
            jitter_pct_max: default_sync_scheduler_jitter_pct_max(),
            max_overridden_interval_seconds: default_sync_scheduler_max_overridden_interval_seconds(
            ),
        }
    }
}

impl Default for RateLimitPolicyConfig {
    fn default() -> Self {
        Self {
            base_seconds: default_rate_limit_base_seconds(),
            max_seconds: default_rate_limit_max_seconds(),
            jitter_factor: default_rate_limit_jitter_factor(),
            provider_overrides: BTreeMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn test_rate_limit_policy_validation() {
        // Test valid config
        let valid_config = RateLimitPolicyConfig {
            base_seconds: 5,
            max_seconds: 900,
            jitter_factor: 0.1,
            provider_overrides: BTreeMap::new(),
        };
        assert!(valid_config.validate().is_ok());

        // Test invalid bounds
        let invalid_bounds = RateLimitPolicyConfig {
            base_seconds: 1000,
            max_seconds: 500,
            jitter_factor: 0.1,
            provider_overrides: BTreeMap::new(),
        };
        assert!(invalid_bounds.validate().is_err());

        // Test invalid jitter
        let invalid_jitter = RateLimitPolicyConfig {
            base_seconds: 5,
            max_seconds: 900,
            jitter_factor: 1.5,
            provider_overrides: BTreeMap::new(),
        };
        assert!(invalid_jitter.validate().is_err());
    }

    #[test]
    fn test_provider_override_validation() {
        let mut provider_overrides = BTreeMap::new();
        provider_overrides.insert(
            "github".to_string(),
            RateLimitProviderOverride {
                base_seconds: Some(100),
                max_seconds: Some(50), // Invalid: base > max
                jitter_factor: None,
            },
        );

        let config = RateLimitPolicyConfig {
            base_seconds: 5,
            max_seconds: 900,
            jitter_factor: 0.1,
            provider_overrides,
        };
        assert!(config.validate().is_err());
    }
}

impl RateLimitPolicyConfig {
    /// Validate rate limit policy configuration bounds
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate base <= max
        if self.base_seconds > self.max_seconds {
            return Err(ConfigError::InvalidRateLimitBounds {
                base: self.base_seconds,
                max: self.max_seconds,
            });
        }

        // Validate jitter factor bounds
        if self.jitter_factor < 0.0 || self.jitter_factor > 1.0 {
            return Err(ConfigError::InvalidRateLimitJitter {
                value: self.jitter_factor,
            });
        }

        // Validate provider overrides
        for (provider, override_config) in &self.provider_overrides {
            let base = override_config.base_seconds.unwrap_or(self.base_seconds);
            let max = override_config.max_seconds.unwrap_or(self.max_seconds);
            let jitter = override_config.jitter_factor.unwrap_or(self.jitter_factor);

            if base > max {
                return Err(ConfigError::InvalidRateLimitProviderBounds {
                    provider: provider.clone(),
                    base,
                    max,
                });
            }

            if jitter < 0.0 || jitter > 1.0 {
                return Err(ConfigError::InvalidRateLimitProviderJitter {
                    provider: provider.clone(),
                    value: jitter,
                });
            }
        }

        Ok(())
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
        // Redact webhook secrets for security
        if config.webhook_github_secret.is_some() {
            config.webhook_github_secret = Some("[REDACTED]".to_string());
        }
        if config.webhook_slack_signing_secret.is_some() {
            config.webhook_slack_signing_secret = Some("[REDACTED]".to_string());
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

        // Validate scheduler configuration
        self.scheduler.validate()?;

        // Validate rate limit policy configuration
        self.rate_limit_policy.validate()?;

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

fn default_webhook_slack_tolerance_seconds() -> u64 {
    300 // 5 minutes
}

fn default_sync_scheduler_tick_interval_seconds() -> u64 {
    60 // 1 minute
}

fn default_sync_scheduler_default_interval_seconds() -> u64 {
    900 // 15 minutes
}

fn default_sync_scheduler_jitter_pct_min() -> f64 {
    0.0 // 0% minimum jitter
}

fn default_sync_scheduler_jitter_pct_max() -> f64 {
    0.2 // 20% maximum jitter
}

fn default_sync_scheduler_max_overridden_interval_seconds() -> u64 {
    86400 // 24 hours
}

fn default_rate_limit_base_seconds() -> u64 {
    5 // 5 seconds
}

fn default_rate_limit_max_seconds() -> u64 {
    900 // 15 minutes
}

fn default_rate_limit_jitter_factor() -> f64 {
    0.1 // 10% jitter
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
    #[error("sync scheduler tick interval must be between 10 and 300 seconds, got {value}")]
    InvalidSchedulerTickInterval { value: u64 },
    #[error(
        "sync scheduler default interval must be at least 60 seconds and not exceed max override ({max_allowed}), got {value}"
    )]
    InvalidSchedulerDefaultInterval { value: u64, max_allowed: u64 },
    #[error("sync scheduler jitter percentage {field} is out of bounds (min: {min}, max: {max})")]
    InvalidSchedulerJitterRange { min: f64, max: f64, field: String },
    #[error(
        "sync scheduler jitter percentage minimum ({min}) cannot be greater than maximum ({max})"
    )]
    InvalidSchedulerJitterInverted { min: f64, max: f64 },
    #[error(
        "sync scheduler max overridden interval must be at least 300 seconds (5 minutes), got {value}"
    )]
    InvalidSchedulerMaxInterval { value: u64 },
    #[error("rate limit base seconds ({base}) cannot be greater than max seconds ({max})")]
    InvalidRateLimitBounds { base: u64, max: u64 },
    #[error("rate limit jitter factor must be between 0.0 and 1.0, got {value}")]
    InvalidRateLimitJitter { value: f64 },
    #[error(
        "provider {provider} rate limit base seconds ({base}) cannot be greater than max seconds ({max})"
    )]
    InvalidRateLimitProviderBounds {
        provider: String,
        base: u64,
        max: u64,
    },
    #[error(
        "provider {provider} rate limit jitter factor must be between 0.0 and 1.0, got {value}"
    )]
    InvalidRateLimitProviderJitter { provider: String, value: f64 },
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

        // Parse webhook secrets
        let webhook_github_secret = layered.remove("WEBHOOK_GITHUB_SECRET");
        let webhook_slack_signing_secret = layered.remove("WEBHOOK_SLACK_SIGNING_SECRET");
        let webhook_slack_tolerance_seconds = layered
            .remove("WEBHOOK_SLACK_TOLERANCE_SECONDS")
            .and_then(|v| v.parse().ok())
            .unwrap_or_else(default_webhook_slack_tolerance_seconds);

        // Parse sync scheduler configuration
        let sync_scheduler_tick_interval_seconds = layered
            .remove("SYNC_SCHEDULER_TICK_INTERVAL_SECONDS")
            .and_then(|v| v.parse().ok())
            .unwrap_or_else(default_sync_scheduler_tick_interval_seconds);
        let sync_scheduler_default_interval_seconds = layered
            .remove("SYNC_SCHEDULER_DEFAULT_INTERVAL_SECONDS")
            .and_then(|v| v.parse().ok())
            .unwrap_or_else(default_sync_scheduler_default_interval_seconds);
        let sync_scheduler_jitter_pct_min = layered
            .remove("SYNC_SCHEDULER_JITTER_PCT_MIN")
            .and_then(|v| v.parse().ok())
            .unwrap_or_else(default_sync_scheduler_jitter_pct_min);
        let sync_scheduler_jitter_pct_max = layered
            .remove("SYNC_SCHEDULER_JITTER_PCT_MAX")
            .and_then(|v| v.parse().ok())
            .unwrap_or_else(default_sync_scheduler_jitter_pct_max);
        let sync_scheduler_max_overridden_interval_seconds = layered
            .remove("SYNC_SCHEDULER_MAX_OVERRIDDEN_INTERVAL_SECONDS")
            .and_then(|v| v.parse().ok())
            .unwrap_or_else(default_sync_scheduler_max_overridden_interval_seconds);

        // Parse rate limit policy configuration
        let rate_limit_base_seconds = layered
            .remove("RATE_LIMIT_BASE_SECONDS")
            .and_then(|v| v.parse().ok())
            .unwrap_or_else(default_rate_limit_base_seconds);
        let rate_limit_max_seconds = layered
            .remove("RATE_LIMIT_MAX_SECONDS")
            .and_then(|v| v.parse().ok())
            .unwrap_or_else(default_rate_limit_max_seconds);
        let rate_limit_jitter_factor = layered
            .remove("RATE_LIMIT_JITTER_FACTOR")
            .and_then(|v| v.parse().ok())
            .unwrap_or_else(default_rate_limit_jitter_factor);

        let scheduler = SchedulerConfig {
            tick_interval_seconds: sync_scheduler_tick_interval_seconds,
            default_interval_seconds: sync_scheduler_default_interval_seconds,
            jitter_pct_min: sync_scheduler_jitter_pct_min,
            jitter_pct_max: sync_scheduler_jitter_pct_max,
            max_overridden_interval_seconds: sync_scheduler_max_overridden_interval_seconds,
        };

        // Parse provider-specific overrides
        let mut provider_overrides = BTreeMap::new();

        // Collect all provider override environment variables
        for (key, value) in layered.clone() {
            if let Some(provider_suffix) = key.strip_prefix("RATE_LIMIT_OVERRIDE_") {
                // Expected format: RATE_LIMIT_OVERRIDE_<PROVIDER>_<SETTING>
                let parts: Vec<&str> = provider_suffix.split('_').collect();
                if parts.len() >= 2 {
                    let provider_name = parts[0].to_lowercase();
                    let setting_name = parts[1..].join("_");

                    let override_entry = provider_overrides
                        .entry(provider_name.clone())
                        .or_insert_with(|| RateLimitProviderOverride {
                            base_seconds: None,
                            max_seconds: None,
                            jitter_factor: None,
                        });

                    match setting_name.as_str() {
                        "base_seconds" => {
                            if let Ok(seconds) = value.parse::<u64>() {
                                override_entry.base_seconds = Some(seconds);
                            }
                        }
                        "max_seconds" => {
                            if let Ok(seconds) = value.parse::<u64>() {
                                override_entry.max_seconds = Some(seconds);
                            }
                        }
                        "jitter_factor" => {
                            if let Ok(factor) = value.parse::<f64>() {
                                override_entry.jitter_factor = Some(factor);
                            }
                        }
                        _ => {
                            // Unknown setting, ignore
                        }
                    }
                }
            }
        }

        let rate_limit_policy = RateLimitPolicyConfig {
            base_seconds: rate_limit_base_seconds,
            max_seconds: rate_limit_max_seconds,
            jitter_factor: rate_limit_jitter_factor,
            provider_overrides,
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
            webhook_github_secret,
            webhook_slack_signing_secret,
            webhook_slack_tolerance_seconds,
            scheduler,
            rate_limit_policy,
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

impl SchedulerConfig {
    /// Validate scheduler configuration bounds.
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.tick_interval_seconds < 10 || self.tick_interval_seconds > 300 {
            return Err(ConfigError::InvalidSchedulerTickInterval {
                value: self.tick_interval_seconds,
            });
        }

        if self.default_interval_seconds < 60
            || self.default_interval_seconds > self.max_overridden_interval_seconds
        {
            return Err(ConfigError::InvalidSchedulerDefaultInterval {
                value: self.default_interval_seconds,
                max_allowed: self.max_overridden_interval_seconds,
            });
        }

        if self.jitter_pct_min < 0.0 || self.jitter_pct_min > 1.0 {
            return Err(ConfigError::InvalidSchedulerJitterRange {
                min: self.jitter_pct_min,
                max: self.jitter_pct_max,
                field: "minimum percentage".to_string(),
            });
        }

        if self.jitter_pct_max < 0.0 || self.jitter_pct_max > 1.0 {
            return Err(ConfigError::InvalidSchedulerJitterRange {
                min: self.jitter_pct_min,
                max: self.jitter_pct_max,
                field: "maximum percentage".to_string(),
            });
        }

        if self.jitter_pct_min > self.jitter_pct_max {
            return Err(ConfigError::InvalidSchedulerJitterInverted {
                min: self.jitter_pct_min,
                max: self.jitter_pct_max,
            });
        }

        if self.max_overridden_interval_seconds < 60
            || self.max_overridden_interval_seconds > 604800
        {
            return Err(ConfigError::InvalidSchedulerMaxInterval {
                value: self.max_overridden_interval_seconds,
            });
        }

        Ok(())
    }
}
