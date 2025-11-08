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
    pub github_client_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub github_client_secret: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub github_oauth_base: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub github_api_base: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub webhook_slack_signing_secret: Option<String>,
    #[serde(default = "default_webhook_slack_tolerance_seconds")]
    pub webhook_slack_tolerance_seconds: u64,
    #[serde(default = "default_webhook_rate_limit_per_minute")]
    pub webhook_rate_limit_per_minute: u32,
    #[serde(default = "default_webhook_rate_limit_burst_size")]
    pub webhook_rate_limit_burst_size: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jira_client_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jira_client_secret: Option<String>,
    #[serde(default = "default_jira_oauth_base")]
    pub jira_oauth_base: String,
    #[serde(default = "default_jira_api_base")]
    pub jira_api_base: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub webhook_jira_secret: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub webhook_zoho_cliq_token: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gmail_scopes: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pubsub_oidc_audience: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pubsub_oidc_issuers: Option<Vec<String>>,
    #[serde(default = "default_pubsub_max_body_kb")]
    pub pubsub_max_body_kb: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gmail_client_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gmail_client_secret: Option<String>,
    #[serde(default)]
    pub scheduler: SchedulerConfig,
    #[serde(default)]
    pub rate_limit_policy: RateLimitPolicyConfig,
    #[serde(default)]
    pub token_refresh: TokenRefreshConfig,
    #[serde(default)]
    pub mail_spam: MailSpamConfig,
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

/// Mail spam filtering configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct MailSpamConfig {
    /// Spam threshold (0.0 to 1.0). Messages scoring >= threshold are considered spam (default: 0.8)
    ///
    /// Environment variable: `POBLYSH_MAIL_SPAM_THRESHOLD`
    #[serde(default = "default_mail_spam_threshold")]
    pub threshold: f32,

    /// Comma-separated list of domains and email addresses that are always allowed (whitelist)
    ///
    /// Supports email addresses (user@example.com) and domains (@example.com)
    ///
    /// Environment variable: `POBLYSH_MAIL_SPAM_ALLOWLIST`
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowlist: Vec<String>,

    /// Comma-separated list of domains and email addresses that are always blocked (blacklist)
    ///
    /// Supports email addresses (user@example.com) and domains (@example.com)
    ///
    /// Environment variable: `POBLYSH_MAIL_SPAM_DENYLIST`
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub denylist: Vec<String>,
}

impl Default for MailSpamConfig {
    fn default() -> Self {
        Self {
            threshold: default_mail_spam_threshold(),
            allowlist: Vec::new(),
            denylist: Vec::new(),
        }
    }
}

impl MailSpamConfig {
    /// Validate mail spam configuration bounds
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate threshold bounds
        if self.threshold < 0.0 || self.threshold > 1.0 {
            return Err(ConfigError::InvalidMailSpamThreshold {
                value: self.threshold,
            });
        }

        // Validate allowlist entries
        for entry in &self.allowlist {
            if !is_valid_email_or_domain(entry) {
                return Err(ConfigError::InvalidMailSpamAllowlistEntry {
                    entry: entry.clone(),
                });
            }
        }

        // Validate denylist entries
        for entry in &self.denylist {
            if !is_valid_email_or_domain(entry) {
                return Err(ConfigError::InvalidMailSpamDenylistEntry {
                    entry: entry.clone(),
                });
            }
        }

        Ok(())
    }
}

/// Token refresh service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct TokenRefreshConfig {
    /// Background refresh interval in seconds (default: 3600)
    #[serde(default = "default_token_refresh_tick_seconds")]
    pub tick_seconds: u64,

    /// Lead time before expiry to trigger refresh in seconds (default: 600)
    #[serde(default = "default_token_refresh_lead_time_seconds")]
    pub lead_time_seconds: u64,

    /// Maximum number of concurrent refresh operations (default: 4)
    #[serde(default = "default_token_refresh_concurrency")]
    pub concurrency: u32,

    /// Jitter factor to avoid thundering herd (default: 0.1)
    #[serde(default = "default_token_refresh_jitter_factor")]
    pub jitter_factor: f64,
}

impl TokenRefreshConfig {
    /// Validate token refresh configuration bounds
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate tick interval (minimum 60 seconds)
        if self.tick_seconds < 60 {
            return Err(ConfigError::InvalidTokenRefreshTickInterval {
                value: self.tick_seconds,
            });
        }

        // Validate lead time (minimum 60 seconds, maximum 86400 seconds)
        if self.lead_time_seconds < 60 || self.lead_time_seconds > 86400 {
            return Err(ConfigError::InvalidTokenRefreshLeadTime {
                value: self.lead_time_seconds,
            });
        }

        // Validate concurrency (minimum 1, maximum 20)
        if self.concurrency == 0 || self.concurrency > 20 {
            return Err(ConfigError::InvalidTokenRefreshConcurrency {
                value: self.concurrency,
            });
        }

        // Validate jitter factor bounds
        if self.jitter_factor < 0.0 || self.jitter_factor > 1.0 {
            return Err(ConfigError::InvalidTokenRefreshJitter {
                value: self.jitter_factor,
            });
        }

        Ok(())
    }
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
            github_client_id: None,
            github_client_secret: None,
            github_oauth_base: None,
            github_api_base: None,
            webhook_slack_signing_secret: None,
            jira_client_id: None,
            jira_client_secret: None,
            jira_oauth_base: default_jira_oauth_base(),
            jira_api_base: default_jira_api_base(),
            webhook_jira_secret: None,
            webhook_zoho_cliq_token: None,
            gmail_scopes: None,
            pubsub_oidc_audience: None,
            pubsub_oidc_issuers: None,
            pubsub_max_body_kb: default_pubsub_max_body_kb(),
            gmail_client_id: None,
            gmail_client_secret: None,
            webhook_slack_tolerance_seconds: default_webhook_slack_tolerance_seconds(),
            webhook_rate_limit_per_minute: default_webhook_rate_limit_per_minute(),
            webhook_rate_limit_burst_size: default_webhook_rate_limit_burst_size(),
            scheduler: SchedulerConfig::default(),
            rate_limit_policy: RateLimitPolicyConfig::default(),
            token_refresh: TokenRefreshConfig::default(),
            mail_spam: MailSpamConfig::default(),
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

impl Default for TokenRefreshConfig {
    fn default() -> Self {
        Self {
            tick_seconds: default_token_refresh_tick_seconds(),
            lead_time_seconds: default_token_refresh_lead_time_seconds(),
            concurrency: default_token_refresh_concurrency(),
            jitter_factor: default_token_refresh_jitter_factor(),
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
        if !(0.0..=1.0).contains(&self.jitter_factor) {
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

            if !(0.0..=1.0).contains(&jitter) {
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
        if config.github_client_id.is_some() {
            config.github_client_id = Some("[REDACTED]".to_string());
        }
        if config.github_client_secret.is_some() {
            config.github_client_secret = Some("[REDACTED]".to_string());
        }
        if config.webhook_slack_signing_secret.is_some() {
            config.webhook_slack_signing_secret = Some("[REDACTED]".to_string());
        }
        if config.jira_client_id.is_some() {
            config.jira_client_id = Some("[REDACTED]".to_string());
        }
        if config.jira_client_secret.is_some() {
            config.jira_client_secret = Some("[REDACTED]".to_string());
        }
        if !config.jira_oauth_base.is_empty() && config.jira_oauth_base != default_jira_oauth_base()
        {
            config.jira_oauth_base = "[REDACTED]".to_string();
        }
        if !config.jira_api_base.is_empty() && config.jira_api_base != default_jira_api_base() {
            config.jira_api_base = "[REDACTED]".to_string();
        }
        if config.webhook_jira_secret.is_some() {
            config.webhook_jira_secret = Some("[REDACTED]".to_string());
        }
        if config.webhook_zoho_cliq_token.is_some() {
            config.webhook_zoho_cliq_token = Some("[REDACTED]".to_string());
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

        // Validate GitHub configuration (only required outside local/test)
        if !matches!(self.profile.as_str(), "local" | "test") {
            if self.github_client_id.is_none() {
                return Err(ConfigError::MissingGitHubClientId);
            }
            if self.github_client_secret.is_none() {
                return Err(ConfigError::MissingGitHubClientSecret);
            }
        }

        // For non-local and non-test profiles, validate Jira configuration
        if !matches!(self.profile.as_str(), "local" | "test") {
            if self.jira_client_id.is_none() {
                return Err(ConfigError::MissingJiraClientId);
            }
            if self.jira_client_secret.is_none() {
                return Err(ConfigError::MissingJiraClientSecret);
            }
        }
        // Validate scheduler configuration
        self.scheduler.validate()?;

        // Validate rate limit policy configuration
        self.rate_limit_policy.validate()?;

        // Validate token refresh configuration
        self.token_refresh.validate()?;

        // Validate mail spam configuration
        self.mail_spam.validate()?;

        // Validate webhook configuration
        if self.webhook_slack_tolerance_seconds == 0 {
            return Err(ConfigError::InvalidSlackTolerance {
                value: self.webhook_slack_tolerance_seconds,
            });
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

fn default_webhook_slack_tolerance_seconds() -> u64 {
    300 // 5 minutes
}

fn default_webhook_rate_limit_per_minute() -> u32 {
    300 // Default rate limit per minute
}

fn default_webhook_rate_limit_burst_size() -> u32 {
    50 // Default burst size
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

fn default_token_refresh_tick_seconds() -> u64 {
    3600 // 1 hour
}

fn default_token_refresh_lead_time_seconds() -> u64 {
    600 // 10 minutes
}

fn default_token_refresh_concurrency() -> u32 {
    4 // concurrent refresh operations
}

fn default_token_refresh_jitter_factor() -> f64 {
    0.1 // 10% jitter
}

fn default_jira_oauth_base() -> String {
    "https://auth.atlassian.com".to_string()
}

fn default_jira_api_base() -> String {
    "https://api.atlassian.com".to_string()
}

fn default_pubsub_max_body_kb() -> usize {
    256 // 256KB default max body size
}

fn default_mail_spam_threshold() -> f32 {
    0.8 // Default spam threshold
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
    #[error("GitHub client ID is missing; set GITHUB_CLIENT_ID environment variable")]
    MissingGitHubClientId,
    #[error("GitHub client secret is missing; set GITHUB_CLIENT_SECRET environment variable")]
    MissingGitHubClientSecret,
    #[error("Jira client ID is missing; set JIRA_CLIENT_ID environment variable")]
    MissingJiraClientId,
    #[error("Jira client secret is missing; set JIRA_CLIENT_SECRET environment variable")]
    MissingJiraClientSecret,
    #[error(
        "Jira webhook secret is missing; set JIRA_WEBHOOK_SECRET or POBLYSH_JIRA_WEBHOOK_SECRET environment variable"
    )]
    MissingJiraWebhookSecret,
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
    #[error("token refresh tick interval must be at least 60 seconds, got {value}")]
    InvalidTokenRefreshTickInterval { value: u64 },
    #[error("token refresh lead time must be between 60 and 86400 seconds, got {value}")]
    InvalidTokenRefreshLeadTime { value: u64 },
    #[error("token refresh concurrency must be between 1 and 20, got {value}")]
    InvalidTokenRefreshConcurrency { value: u32 },
    #[error("token refresh jitter factor must be between 0.0 and 1.0, got {value}")]
    InvalidTokenRefreshJitter { value: f64 },
    #[error("mail spam threshold must be between 0.0 and 1.0, got {value}")]
    InvalidMailSpamThreshold { value: f32 },
    #[error("invalid mail spam allowlist entry: {entry}")]
    InvalidMailSpamAllowlistEntry { entry: String },
    #[error("invalid mail spam denylist entry: {entry}")]
    InvalidMailSpamDenylistEntry { entry: String },
    #[error("webhook Slack tolerance must be positive, got {value}")]
    InvalidSlackTolerance { value: u64 },
}

/// Check if a string is a valid email or domain format
fn is_valid_email_or_domain(entry: &str) -> bool {
    if let Some(domain) = entry.strip_prefix('@') {
        // Domain format (e.g., @example.com)
        domain.contains('.')
            && domain
                .chars()
                .all(|c| c.is_alphanumeric() || c == '.' || c == '-')
    } else if entry.contains('@') {
        // Email format
        let parts: Vec<&str> = entry.split('@').collect();
        parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty()
    } else {
        // Simple domain format
        entry.contains('.')
            && entry
                .chars()
                .all(|c| c.is_alphanumeric() || c == '.' || c == '-')
    }
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
        let github_client_id = layered.remove("GITHUB_CLIENT_ID");
        let github_client_secret = layered.remove("GITHUB_CLIENT_SECRET");
        let github_oauth_base = layered.remove("GITHUB_OAUTH_BASE");
        let github_api_base = layered.remove("GITHUB_API_BASE");
        let webhook_slack_signing_secret = layered.remove("WEBHOOK_SLACK_SIGNING_SECRET");
        let jira_client_id = layered.remove("JIRA_CLIENT_ID").and_then(|val| {
            let trimmed = val.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        });
        let jira_client_secret = layered.remove("JIRA_CLIENT_SECRET").and_then(|val| {
            let trimmed = val.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        });
        let jira_oauth_base = layered
            .remove("JIRA_OAUTH_BASE")
            .or_else(|| Some(default_jira_oauth_base()));
        let jira_api_base = layered
            .remove("JIRA_API_BASE")
            .or_else(|| Some(default_jira_api_base()));
        let webhook_jira_secret = layered.remove("WEBHOOK_JIRA_SECRET");
        let webhook_zoho_cliq_token = layered.remove("WEBHOOK_ZOHO_CLIQ_TOKEN");

        // Parse Gmail configuration
        let gmail_scopes = layered.remove("GMAIL_SCOPES");
        let gmail_client_id = layered.remove("GMAIL_CLIENT_ID");
        let gmail_client_secret = layered.remove("GMAIL_CLIENT_SECRET");
        let pubsub_oidc_audience = layered.remove("PUBSUB_OIDC_AUDIENCE");
        let pubsub_oidc_issuers = layered.remove("PUBSUB_OIDC_ISSUERS").map(|issuers| {
            issuers
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        });
        let pubsub_max_body_kb = layered
            .remove("PUBSUB_MAX_BODY_KB")
            .and_then(|v| v.parse().ok())
            .unwrap_or_else(default_pubsub_max_body_kb);
        let webhook_slack_tolerance_seconds = layered
            .remove("WEBHOOK_SLACK_TOLERANCE_SECONDS")
            .and_then(|v| v.parse().ok())
            .unwrap_or_else(default_webhook_slack_tolerance_seconds);

        let webhook_rate_limit_per_minute = layered
            .remove("WEBHOOK_RATE_LIMIT_PER_MINUTE")
            .and_then(|v| v.parse().ok())
            .unwrap_or_else(default_webhook_rate_limit_per_minute);

        let webhook_rate_limit_burst_size = layered
            .remove("WEBHOOK_RATE_LIMIT_BURST_SIZE")
            .and_then(|v| v.parse().ok())
            .unwrap_or_else(default_webhook_rate_limit_burst_size);

        // Do not inject hardcoded Jira client credentials; require explicit configuration

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

        // Parse token refresh configuration
        let token_refresh_tick_seconds = layered
            .remove("TOKEN_REFRESH_TICK_SECONDS")
            .and_then(|v| v.parse().ok())
            .unwrap_or_else(default_token_refresh_tick_seconds);
        let token_refresh_lead_time_seconds = layered
            .remove("TOKEN_REFRESH_LEAD_TIME_SECONDS")
            .and_then(|v| v.parse().ok())
            .unwrap_or_else(default_token_refresh_lead_time_seconds);
        let token_refresh_concurrency = layered
            .remove("TOKEN_REFRESH_CONCURRENCY")
            .and_then(|v| v.parse().ok())
            .unwrap_or_else(default_token_refresh_concurrency);
        let token_refresh_jitter_factor = layered
            .remove("TOKEN_REFRESH_JITTER_FACTOR")
            .and_then(|v| v.parse().ok())
            .unwrap_or_else(default_token_refresh_jitter_factor);

        // Parse mail spam configuration
        let mail_spam_threshold = layered
            .remove("MAIL_SPAM_THRESHOLD")
            .and_then(|v| v.parse().ok())
            .unwrap_or_else(default_mail_spam_threshold);
        let mail_spam_allowlist = layered
            .remove("MAIL_SPAM_ALLOWLIST")
            .map(|allowlist| {
                allowlist
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default();
        let mail_spam_denylist = layered
            .remove("MAIL_SPAM_DENYLIST")
            .map(|denylist| {
                denylist
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default();

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

        let token_refresh = TokenRefreshConfig {
            tick_seconds: token_refresh_tick_seconds,
            lead_time_seconds: token_refresh_lead_time_seconds,
            concurrency: token_refresh_concurrency,
            jitter_factor: token_refresh_jitter_factor,
        };

        let mail_spam = MailSpamConfig {
            threshold: mail_spam_threshold,
            allowlist: mail_spam_allowlist,
            denylist: mail_spam_denylist,
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
            github_client_id,
            github_client_secret,
            github_oauth_base,
            github_api_base,
            webhook_slack_signing_secret,
            webhook_slack_tolerance_seconds,
            webhook_rate_limit_per_minute,
            webhook_rate_limit_burst_size,
            scheduler,
            rate_limit_policy,
            token_refresh,
            jira_client_id,
            jira_client_secret,
            jira_oauth_base: jira_oauth_base.unwrap_or_default(),
            jira_api_base: jira_api_base.unwrap_or_default(),
            webhook_jira_secret,
            webhook_zoho_cliq_token,
            gmail_scopes,
            gmail_client_id,
            gmail_client_secret,
            pubsub_oidc_audience,
            pubsub_oidc_issuers,
            pubsub_max_body_kb,
            mail_spam,
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
