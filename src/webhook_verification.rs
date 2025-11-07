//! # Webhook Signature Verification
//!
//! This module provides signature verification for GitHub and Slack webhooks
//! using HMAC-SHA256 with constant-time comparison to prevent timing attacks.

use std::time::{SystemTime, UNIX_EPOCH};

use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use tracing::{debug, error, info, warn};

use crate::config::AppConfig;

type HmacSha256 = Hmac<Sha256>;

// Simple in-memory fixed-window rate limiter per (provider, tenant_id)
// Window unit: seconds epoch rounded to minute
const WEBHOOK_RL_PER_MINUTE: u32 = 300; // default limit per provider/tenant per minute
static WEBHOOK_RL: OnceLock<Mutex<HashMap<String, (u64, u32)>>> = OnceLock::new();

fn is_rate_limited(provider: &str, tenant_id: &str) -> bool {
    let map = WEBHOOK_RL.get_or_init(|| Mutex::new(HashMap::new()));
    let key = format!("{}:{}", provider, tenant_id);
    let now_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let window = now_secs / 60; // minute bucket
    let mut guard = map.lock().unwrap();
    let entry = guard.entry(key).or_insert((window, 0));
    if entry.0 != window {
        // new window
        *entry = (window, 0);
    }
    if entry.1 >= WEBHOOK_RL_PER_MINUTE {
        true
    } else {
        entry.1 += 1;
        false
    }
}

/// Errors that can occur during webhook signature verification
#[derive(Debug, thiserror::Error)]
pub enum VerificationError {
    #[error("Missing required signature header: {header}")]
    MissingSignature { header: String },

    #[error("Invalid signature format: {header}")]
    InvalidSignatureFormat { header: String },

    #[error("Signature verification failed")]
    VerificationFailed,

    #[error("Missing required timestamp header: {header}")]
    MissingTimestamp { header: String },

    #[error("Invalid timestamp format: {header}")]
    InvalidTimestamp { header: String },

    #[error("Timestamp too old: {seconds}s old, max allowed: {max_seconds}s")]
    TimestampTooOld { seconds: u64, max_seconds: u64 },

    #[error("Timestamp too far in future: {seconds}s in future, max allowed: {max_seconds}s")]
    TimestampTooFuture { seconds: u64, max_seconds: u64 },

    #[error("Unsupported provider: {provider}")]
    UnsupportedProvider { provider: String },

    #[error("Webhook verification not configured for provider: {provider}")]
    NotConfigured { provider: String },
}

impl VerificationError {
    /// Returns the appropriate HTTP status code for this error
    pub fn status_code(&self) -> StatusCode {
        match self {
            VerificationError::MissingSignature { .. } => StatusCode::UNAUTHORIZED,
            VerificationError::InvalidSignatureFormat { .. } => StatusCode::UNAUTHORIZED,
            VerificationError::VerificationFailed => StatusCode::UNAUTHORIZED,
            VerificationError::MissingTimestamp { .. } => StatusCode::UNAUTHORIZED,
            VerificationError::InvalidTimestamp { .. } => StatusCode::UNAUTHORIZED,
            VerificationError::TimestampTooOld { .. } => StatusCode::UNAUTHORIZED,
            VerificationError::TimestampTooFuture { .. } => StatusCode::UNAUTHORIZED,
            VerificationError::UnsupportedProvider { .. } => StatusCode::NOT_FOUND,
            VerificationError::NotConfigured { .. } => StatusCode::UNAUTHORIZED,
        }
    }
}

/// Result type for webhook verification
pub type VerificationResult<T> = Result<T, VerificationError>;

/// Verifies GitHub webhook signature using HMAC-SHA256
pub fn verify_github_signature(
    body: &[u8],
    signature_header: &str,
    secret: &str,
) -> VerificationResult<()> {
    debug!(
        body_size = body.len(),
        "Starting GitHub signature verification"
    );

    if signature_header.is_empty() {
        return Err(VerificationError::MissingSignature {
            header: "X-Hub-Signature-256".to_string(),
        });
    }

    // GitHub signatures are prefixed with "sha256="
    let signature_prefix = "sha256=";
    if !signature_header.starts_with(signature_prefix) {
        return Err(VerificationError::InvalidSignatureFormat {
            header: "X-Hub-Signature-256 must start with 'sha256='".to_string(),
        });
    }

    let expected_hex = &signature_header[signature_prefix.len()..];

    // Compute HMAC-SHA256 of the body
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|_| VerificationError::VerificationFailed)?;
    mac.update(body);
    let expected_bytes = mac.finalize().into_bytes();

    // Decode the provided signature
    let provided_bytes =
        hex::decode(expected_hex).map_err(|_| VerificationError::InvalidSignatureFormat {
            header: "X-Hub-Signature-256 contains invalid hex".to_string(),
        })?;

    // Compare signatures using constant-time comparison to prevent timing attacks
    let expected_bytes_array: &[u8] = expected_bytes.as_ref();
    if subtle::ConstantTimeEq::ct_eq(expected_bytes_array, &provided_bytes[..]).into() {
        Ok(())
    } else {
        Err(VerificationError::VerificationFailed)
    }
}

/// Verifies Slack v2 webhook signature using HMAC-SHA256 with timestamp validation
pub fn verify_slack_signature(
    body: &[u8],
    signature_header: &str,
    timestamp_header: &str,
    secret: &str,
    tolerance_seconds: u64,
) -> VerificationResult<()> {
    debug!(
        body_size = body.len(),
        tolerance_seconds, "Starting Slack signature verification"
    );

    if signature_header.is_empty() {
        return Err(VerificationError::MissingSignature {
            header: "X-Slack-Signature".to_string(),
        });
    }

    if timestamp_header.is_empty() {
        return Err(VerificationError::MissingTimestamp {
            header: "X-Slack-Request-Timestamp".to_string(),
        });
    }

    // Parse timestamp
    let timestamp =
        timestamp_header
            .parse::<u64>()
            .map_err(|_| VerificationError::InvalidTimestamp {
                header: "X-Slack-Request-Timestamp must be a valid Unix timestamp".to_string(),
            })?;

    // Check timestamp is within tolerance window
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| VerificationError::InvalidTimestamp {
            header: "Failed to get current time".to_string(),
        })?
        .as_secs();

    let time_diff = now.abs_diff(timestamp);

    if time_diff > tolerance_seconds {
        if now > timestamp {
            return Err(VerificationError::TimestampTooOld {
                seconds: time_diff,
                max_seconds: tolerance_seconds,
            });
        } else {
            return Err(VerificationError::TimestampTooFuture {
                seconds: time_diff,
                max_seconds: tolerance_seconds,
            });
        }
    }

    // Slack signatures are prefixed with "v0="
    let signature_prefix = "v0=";
    if !signature_header.starts_with(signature_prefix) {
        return Err(VerificationError::InvalidSignatureFormat {
            header: "X-Slack-Signature must start with 'v0='".to_string(),
        });
    }

    let expected_hex = &signature_header[signature_prefix.len()..];

    // Construct the base string: "v0:{timestamp}:{body}"
    let base_string = format!("v0:{}:{}", timestamp, String::from_utf8_lossy(body));

    // Compute HMAC-SHA256 of the base string
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|_| VerificationError::VerificationFailed)?;
    mac.update(base_string.as_bytes());
    let expected_bytes = mac.finalize().into_bytes();

    // Decode the provided signature
    let provided_bytes =
        hex::decode(expected_hex).map_err(|_| VerificationError::InvalidSignatureFormat {
            header: "X-Slack-Signature contains invalid hex".to_string(),
        })?;

    // Compare signatures using constant-time comparison to prevent timing attacks
    let expected_bytes_array: &[u8] = expected_bytes.as_ref();
    if subtle::ConstantTimeEq::ct_eq(expected_bytes_array, &provided_bytes[..]).into() {
        Ok(())
    } else {
        Err(VerificationError::VerificationFailed)
    }
}

/// Verifies webhook signature for the given provider
pub fn verify_webhook_signature(
    provider: &str,
    body: &[u8],
    headers: &HeaderMap,
    config: &AppConfig,
) -> VerificationResult<()> {
    match provider {
        "github" => {
            let secret = config.webhook_github_secret.as_ref().ok_or_else(|| {
                VerificationError::NotConfigured {
                    provider: "github".to_string(),
                }
            })?;

            let signature_header = headers
                .get("x-hub-signature-256")
                .and_then(|h| h.to_str().ok())
                .unwrap_or("");

            verify_github_signature(body, signature_header, secret)
        }
        "slack" => {
            let secret = config
                .webhook_slack_signing_secret
                .as_ref()
                .ok_or_else(|| VerificationError::NotConfigured {
                    provider: "slack".to_string(),
                })?;

            let signature_header = headers
                .get("x-slack-signature")
                .and_then(|h| h.to_str().ok())
                .unwrap_or("");

            let timestamp_header = headers
                .get("x-slack-request-timestamp")
                .and_then(|h| h.to_str().ok())
                .unwrap_or("");

            verify_slack_signature(
                body,
                signature_header,
                timestamp_header,
                secret,
                config.webhook_slack_tolerance_seconds,
            )
        }
        "jira" => {
            let secret = config.webhook_jira_secret.as_ref().ok_or_else(|| {
                VerificationError::NotConfigured {
                    provider: "jira".to_string(),
                }
            })?;

            // Enforce a single method: Authorization: Bearer <secret>
            let provided_auth = headers
                .get("authorization")
                .and_then(|h| h.to_str().ok())
                .unwrap_or("");

            if let Some(token) = provided_auth.strip_prefix("Bearer ") {
                if subtle::ConstantTimeEq::ct_eq(token.as_bytes(), secret.as_bytes()).into() {
                    Ok(())
                } else {
                    Err(VerificationError::VerificationFailed)
                }
            } else {
                Err(VerificationError::MissingSignature {
                    header: "Authorization (Bearer)".to_string(),
                })
            }
        }
        "zoho-cliq" => {
            let token = config.webhook_zoho_cliq_token.as_ref().ok_or_else(|| {
                VerificationError::NotConfigured {
                    provider: "zoho-cliq".to_string(),
                }
            })?;

            // Enforce Authorization: Bearer <token> method
            let provided_auth = headers
                .get("authorization")
                .and_then(|h| h.to_str().ok())
                .unwrap_or("");

            if let Some(bearer_token) = provided_auth.strip_prefix("Bearer ") {
                if subtle::ConstantTimeEq::ct_eq(bearer_token.as_bytes(), token.as_bytes()).into() {
                    Ok(())
                } else {
                    Err(VerificationError::VerificationFailed)
                }
            } else {
                Err(VerificationError::MissingSignature {
                    header: "Authorization (Bearer)".to_string(),
                })
            }
        }
        _ => Err(VerificationError::UnsupportedProvider {
            provider: provider.to_string(),
        }),
    }
}

/// Middleware for webhook signature verification on public routes
pub async fn webhook_verification_middleware(
    State(config): State<std::sync::Arc<AppConfig>>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract path first to avoid borrowing issues
    let path = request.uri().path().to_string();

    // Only apply to public webhook routes with tenant_id
    if !path.starts_with("/webhooks/") || path.split('/').count() != 4 {
        return Ok(next.run(request).await);
    }

    // Extract provider and tenant_id from path
    let path_parts: Vec<&str> = path.split('/').collect();
    let provider = path_parts[2];
    let tenant_id = path_parts[3];

    // Check if verification is configured for this provider
    let verification_enabled = match provider {
        "github" => config.webhook_github_secret.is_some(),
        "slack" => config.webhook_slack_signing_secret.is_some(),
        "jira" => config.webhook_jira_secret.is_some(),
        "zoho-cliq" => config.webhook_zoho_cliq_token.is_some(),
        _ => false,
    };

    // For Jira, allow pass-through in local/test when secret not configured
    if provider == "jira" && !verification_enabled {
        if matches!(config.profile.as_str(), "local" | "test") {
            // Allow in dev/test without verification
            return Ok(next.run(request).await);
        } else {
            warn!(
                provider = %provider,
                "Jira webhook secret not configured; skipping verification"
            );
            return Ok(next.run(request).await);
        }
    }

    if !verification_enabled {
        warn!(
            provider = %provider,
            "Webhook verification not configured for provider"
        );
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Basic per-tenant/provider rate limiting (fixed window per minute)
    if is_rate_limited(provider, tenant_id) {
        warn!(provider = %provider, tenant_id = %tenant_id, "Webhook rate limit exceeded");
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    // Get the request body bytes for signature verification
    let (parts, body) = request.into_parts();
    let body_bytes = axum::body::to_bytes(body, usize::MAX).await.map_err(|e| {
        error!(error = ?e, "Failed to read request body for webhook verification");
        StatusCode::BAD_REQUEST
    })?;

    // Verify the signature
    match verify_webhook_signature(provider, &body_bytes, &parts.headers, &config) {
        Ok(()) => {
            info!(
                provider = %provider,
                body_size = body_bytes.len(),
                "Webhook signature verified successfully"
            );

            // Reconstruct the request with the body
            let request = Request::from_parts(parts, axum::body::Body::from(body_bytes));
            Ok(next.run(request).await)
        }
        Err(e) => {
            let error_msg = match &e {
                VerificationError::MissingSignature { header } => {
                    format!("Missing required header: {}", header)
                }
                VerificationError::InvalidSignatureFormat { header } => {
                    format!("Invalid signature format: {}", header)
                }
                VerificationError::VerificationFailed => {
                    "Signature verification failed".to_string()
                }
                VerificationError::MissingTimestamp { header } => {
                    format!("Missing required timestamp: {}", header)
                }
                VerificationError::InvalidTimestamp { header } => {
                    format!("Invalid timestamp format: {}", header)
                }
                VerificationError::TimestampTooOld {
                    seconds,
                    max_seconds,
                } => {
                    format!("Timestamp too old: {}s (max: {}s)", seconds, max_seconds)
                }
                VerificationError::TimestampTooFuture {
                    seconds,
                    max_seconds,
                } => {
                    format!(
                        "Timestamp too far in future: {}s (max: {}s)",
                        seconds, max_seconds
                    )
                }
                VerificationError::UnsupportedProvider { provider } => {
                    format!("Unsupported provider: {}", provider)
                }
                VerificationError::NotConfigured { provider } => {
                    format!(
                        "Webhook verification not configured for provider: {}",
                        provider
                    )
                }
            };

            error!(
                provider = %provider,
                error = %error_msg,
                "Webhook signature verification failed"
            );

            Err(e.status_code())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_signature_verification_success() {
        let secret = "test_secret";
        let body = b"test payload";

        // Compute expected signature
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(body);
        let expected = hex::encode(mac.finalize().into_bytes());
        let signature_header = format!("sha256={}", expected);

        assert!(verify_github_signature(body, &signature_header, secret).is_ok());
    }

    #[test]
    fn test_github_signature_verification_invalid_signature() {
        let secret = "test_secret";
        let body = b"test payload";
        let signature_header = "sha256=invalid_signature";

        assert!(verify_github_signature(body, signature_header, secret).is_err());
    }

    #[test]
    fn test_github_signature_verification_missing_signature() {
        let secret = "test_secret";
        let body = b"test payload";
        let signature_header = "";

        assert!(verify_github_signature(body, signature_header, secret).is_err());
    }

    #[test]
    fn test_github_signature_verification_invalid_format() {
        let secret = "test_secret";
        let body = b"test payload";
        let signature_header = "invalid_format";

        assert!(verify_github_signature(body, signature_header, secret).is_err());
    }

    #[test]
    fn test_slack_signature_verification_success() {
        let secret = "test_secret";
        let body = b"test payload";
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string();

        // Compute expected signature
        let base_string = format!("v0:{}:{}", timestamp, String::from_utf8_lossy(body));
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(base_string.as_bytes());
        let expected = hex::encode(mac.finalize().into_bytes());
        let signature_header = format!("v0={}", expected);

        assert!(verify_slack_signature(body, &signature_header, &timestamp, secret, 300).is_ok());
    }

    #[test]
    fn test_slack_signature_verification_timestamp_too_old() {
        let secret = "test_secret";
        let body = b"test payload";
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let old_timestamp = now - 400; // 400 seconds ago
        let timestamp = old_timestamp.to_string();

        let base_string = format!("v0:{}:{}", timestamp, String::from_utf8_lossy(body));
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(base_string.as_bytes());
        let expected = hex::encode(mac.finalize().into_bytes());
        let signature_header = format!("v0={}", expected);

        assert!(verify_slack_signature(body, &signature_header, &timestamp, secret, 300).is_err());
    }

    #[test]
    fn test_slack_signature_verification_invalid_timestamp() {
        let secret = "test_secret";
        let body = b"test payload";
        let timestamp = "invalid_timestamp";
        let signature_header = "v0=valid_signature";

        assert!(verify_slack_signature(body, signature_header, timestamp, secret, 300).is_err());
    }

    #[test]
    fn test_unsupported_provider() {
        let body = b"test payload";
        let headers = HeaderMap::new();
        let config = AppConfig::default();

        assert!(verify_webhook_signature("unsupported", body, &headers, &config).is_err());
    }

    #[test]
    fn test_jira_secret_header_no_longer_accepted() {
        let mut headers = HeaderMap::new();
        headers.insert("X-Webhook-Secret", "test-secret-123".parse().unwrap());

        let mut config = AppConfig::default();
        config.webhook_jira_secret = Some("test-secret-123".to_string());

        // Only Authorization: Bearer is accepted now
        assert!(verify_webhook_signature("jira", b"{}", &headers, &config).is_err());
    }

    #[test]
    fn test_jira_secret_verification_with_bearer() {
        let mut headers = HeaderMap::new();
        headers.insert("authorization", "Bearer test-secret-123".parse().unwrap());

        let mut config = AppConfig::default();
        config.webhook_jira_secret = Some("test-secret-123".to_string());

        assert!(verify_webhook_signature("jira", b"{}", &headers, &config).is_ok());
    }

    #[test]
    fn test_jira_secret_verification_missing() {
        let headers = HeaderMap::new();
        let mut config = AppConfig::default();
        config.webhook_jira_secret = Some("test-secret-123".to_string());

        assert!(verify_webhook_signature("jira", b"{}", &headers, &config).is_err());
    }

    #[test]
    fn test_zoho_cliq_token_verification_with_bearer() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "authorization",
            "Bearer zoho-cliq-token-123".parse().unwrap(),
        );

        let mut config = AppConfig::default();
        config.webhook_zoho_cliq_token = Some("zoho-cliq-token-123".to_string());

        assert!(verify_webhook_signature("zoho-cliq", b"{}", &headers, &config).is_ok());
    }

    #[test]
    fn test_zoho_cliq_token_verification_invalid_token() {
        let mut headers = HeaderMap::new();
        headers.insert("authorization", "Bearer wrong-token".parse().unwrap());

        let mut config = AppConfig::default();
        config.webhook_zoho_cliq_token = Some("zoho-cliq-token-123".to_string());

        assert!(verify_webhook_signature("zoho-cliq", b"{}", &headers, &config).is_err());
    }

    #[test]
    fn test_zoho_cliq_token_verification_missing() {
        let headers = HeaderMap::new();
        let mut config = AppConfig::default();
        config.webhook_zoho_cliq_token = Some("zoho-cliq-token-123".to_string());

        assert!(verify_webhook_signature("zoho-cliq", b"{}", &headers, &config).is_err());
    }

    #[test]
    fn test_zoho_cliq_token_verification_not_configured() {
        let mut headers = HeaderMap::new();
        headers.insert("authorization", "Bearer some-token".parse().unwrap());

        let config = AppConfig::default(); // No token configured

        assert!(verify_webhook_signature("zoho-cliq", b"{}", &headers, &config).is_err());
    }
}
