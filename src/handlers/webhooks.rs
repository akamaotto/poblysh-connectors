//! # Webhook Handlers
//!
//! This module contains handlers for processing webhook callbacks from external providers.
//! For MVP, these endpoints are protected by operator authentication and tenant scoping.

use axum::{
    extract::{Path, Request, State},
    http::{HeaderMap, StatusCode},
    response::Json,
};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use tracing::{debug, error, info};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::auth::{OperatorAuth, TenantExtension, TenantId};
use crate::error::ApiError;
use crate::handlers::TenantHeader;
use crate::repositories::{ConnectionRepository, ProviderRepository, SyncJobRepository};
use crate::server::AppState;

/// Path parameter for provider slug
#[derive(Debug, Deserialize, ToSchema)]
pub struct ProviderPath {
    /// Provider slug (e.g., "github", "jira")
    pub provider: String,
}

/// Path parameter for provider slug and tenant ID (public webhook routes)
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct ProviderTenantPath {
    /// Provider slug (e.g., "github", "jira", "zoho-cliq")
    #[param(min_length = 1, example = "zoho-cliq")]
    pub provider: String,
    /// Tenant UUID for scoping the webhook
    #[param(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub tenant_id: String, // Using String to avoid ToSchema issues with Uuid
}

/// Optional connection ID header for targeting specific connections
#[derive(Debug, Deserialize)]
pub struct ConnectionIdHeader {
    /// Connection UUID to target (optional)
    #[serde(rename = "X-Connection-Id")]
    pub connection_id: Option<Uuid>,
}

/// Webhook accept response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct WebhookAcceptResponse {
    /// Acceptance status
    pub status: String,
}

/// Path parameter for provider slug with OpenAPI support
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct ProviderPathParam {
    /// Provider slug (e.g., "github", "jira")
    #[param(min_length = 1, example = "github")]
    pub provider: String,
}

/// GitHub signature header for webhook verification
#[derive(Debug, Serialize, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Header)]
pub struct GitHubSignatureHeader {
    /// HMAC-SHA256 signature of the request body (hex string with sha256= prefix)
    #[serde(rename = "X-Hub-Signature-256")]
    #[param(rename = "X-Hub-Signature-256", value_type = String)]
    pub signature: String,
}

/// Slack signature headers for webhook verification
#[derive(Debug, Serialize, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Header)]
pub struct SlackSignatureHeaders {
    /// HMAC-SHA256 signature of the request body (hex string with v0= prefix)
    #[serde(rename = "X-Slack-Signature")]
    #[param(rename = "X-Slack-Signature", value_type = String)]
    pub signature: String,
    /// Unix timestamp of when the request was generated
    #[serde(rename = "X-Slack-Request-Timestamp")]
    #[param(rename = "X-Slack-Request-Timestamp", value_type = String)]
    pub timestamp: String,
}

/// Helper function to extract optional connection ID from headers
fn extract_connection_id(headers: &HeaderMap) -> Result<Option<Uuid>, ApiError> {
    match headers.get("X-Connection-Id") {
        Some(header_value) => {
            let header_str = header_value.to_str().map_err(|_| {
                ApiError::new(
                    axum::http::StatusCode::BAD_REQUEST,
                    "VALIDATION_FAILED",
                    "invalid X-Connection-Id header encoding",
                )
            })?;

            let uuid = Uuid::parse_str(header_str).map_err(|_| {
                ApiError::new(
                    axum::http::StatusCode::BAD_REQUEST,
                    "VALIDATION_FAILED",
                    "invalid X-Connection-Id header format",
                )
            })?;

            Ok(Some(uuid))
        }
        None => Ok(None),
    }
}

/// Verify Jira webhook secret
pub fn verify_jira_webhook_secret(
    headers: &HeaderMap,
    _body: &[u8],
    expected_secret: &str,
) -> Result<(), ApiError> {
    // Check for X-Webhook-Secret header (common convention for simple webhook verification)
    if let Some(secret_header) = headers.get("X-Webhook-Secret") {
        let provided_secret = secret_header.to_str().map_err(|_| {
            ApiError::new(
                axum::http::StatusCode::BAD_REQUEST,
                "VALIDATION_FAILED",
                "Invalid X-Webhook-Secret header encoding",
            )
        })?;

        if provided_secret == expected_secret {
            return Ok(());
        } else {
            tracing::warn!("Jira webhook secret verification failed: secret mismatch");
            return Err(ApiError::new(
                axum::http::StatusCode::UNAUTHORIZED,
                "UNAUTHORIZED",
                "Invalid webhook secret",
            ));
        }
    }

    // Check for Authorization header with Bearer token (alternative approach)
    if let Some(auth_header) = headers.get("Authorization") {
        let auth_str = auth_header.to_str().map_err(|_| {
            ApiError::new(
                axum::http::StatusCode::BAD_REQUEST,
                "VALIDATION_FAILED",
                "Invalid Authorization header encoding",
            )
        })?;

        // Expect format: "Bearer <secret>" or just "<secret>"
        let provided_secret = if let Some(bearer_token) = auth_str.strip_prefix("Bearer ") {
            bearer_token
        } else {
            auth_str
        };

        if provided_secret == expected_secret {
            return Ok(());
        } else {
            tracing::warn!(
                "Jira webhook secret verification failed: authorization secret mismatch"
            );
            return Err(ApiError::new(
                axum::http::StatusCode::UNAUTHORIZED,
                "UNAUTHORIZED",
                "Invalid webhook secret",
            ));
        }
    }

    tracing::warn!("Jira webhook secret verification failed: no secret provided");
    Err(ApiError::new(
        axum::http::StatusCode::UNAUTHORIZED,
        "UNAUTHORIZED",
        "Missing webhook secret",
    ))
}

/// Helper function to parse JSON from body bytes
fn parse_webhook_body_from_bytes(bytes: &[u8]) -> Option<JsonValue> {
    // If body is empty, return None
    if bytes.is_empty() {
        return None;
    }

    // Try to parse as JSON - if it fails, we still want to capture the raw payload
    serde_json::from_slice(bytes).ok()
}

/// Verify Gmail webhook OIDC token synchronously
fn verify_gmail_webhook_oidc(
    headers: &HeaderMap,
    config: &crate::config::AppConfig,
    _body_bytes: &[u8],
) -> Result<(), ApiError> {
    // For Gmail provider, OIDC verification is mandatory
    let (audience, issuers) = match (&config.pubsub_oidc_audience, &config.pubsub_oidc_issuers) {
        (Some(audience), Some(issuers)) => (audience, issuers),
        _ => {
            // OIDC verification is required for Gmail
            let missing_fields = match (&config.pubsub_oidc_audience, &config.pubsub_oidc_issuers) {
                (None, None) => "both POBLYSH_PUBSUB_OIDC_AUDIENCE and POBLYSH_PUBSUB_OIDC_ISSUERS",
                (None, Some(_)) => "POBLYSH_PUBSUB_OIDC_AUDIENCE",
                (Some(_), None) => "POBLYSH_PUBSUB_OIDC_ISSUERS",
                _ => unreachable!(),
            };
            return Err(ApiError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_SERVER_ERROR",
                format!(
                    "Gmail OIDC verification is required but missing configuration: {}",
                    missing_fields
                ),
            ));
        }
    };

    // Extract Authorization header
    let auth_header = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| {
            ApiError::new(
                StatusCode::UNAUTHORIZED,
                "UNAUTHORIZED",
                "Missing Authorization header for Gmail webhook",
            )
        })?;

    // Verify JWT synchronously using Gmail connector's OIDC verification
    let spam_filter = std::sync::Arc::new(crate::mail::default::DefaultMailSpamFilter::default());
    let connector = crate::connectors::gmail::GmailConnector::new_with_oidc(
        "dummy-client-id".to_string(),
        "dummy-client-secret".to_string(),
        Some(audience.clone()),
        Some(issuers.clone()),
        spam_filter,
    );

    // Use tokio runtime to verify the token synchronously
    let rt = tokio::runtime::Handle::current();
    rt.block_on(async { connector.verify_oidc_token(Some(auth_header)).await })
        .map_err(|e| {
            error!(error = ?e, "Gmail OIDC token verification failed");
            ApiError::new(
                StatusCode::UNAUTHORIZED,
                "UNAUTHORIZED",
                format!("OIDC token verification failed: {}", e),
            )
        })?;

    Ok(())
}

/// Validate Gmail webhook body size
fn validate_gmail_webhook_body_size(
    body_bytes: &[u8],
    config: &crate::config::AppConfig,
) -> Result<(), ApiError> {
    let max_size_kb = config.pubsub_max_body_kb;
    let max_size_bytes = max_size_kb * 1024;

    if body_bytes.len() > max_size_bytes {
        return Err(ApiError::new(
            StatusCode::PAYLOAD_TOO_LARGE,
            "PAYLOAD_TOO_LARGE",
            format!(
                "Webhook body size {} bytes exceeds maximum allowed size {} KB",
                body_bytes.len(),
                max_size_kb
            ),
        ));
    }

    Ok(())
}

/// Accept webhook from external provider
///
/// This endpoint receives webhook callbacks from external providers. For MVP,
/// it requires operator authentication and tenant scoping. A valid connection
/// can be specified via the X-Connection-Id header to enqueue a sync job.
#[utoipa::path(
    post,
    path = "/webhooks/{provider}",
    security(("bearer_auth" = [])),
    params(
        TenantHeader,
        ("X-Connection-Id" = Option<String>, Header, description = "Optional connection ID to target"),
        ProviderPathParam
    ),
    request_body(content = Option<JsonValue>, description = "Webhook payload (opaque to API)", content_type = "application/json"),
    responses(
        (status = 202, description = "Webhook accepted", body = WebhookAcceptResponse),
        (status = 400, description = "Invalid connection ID header", body = ApiError),
        (status = 401, description = "Missing or invalid operator token", body = ApiError),
        (status = 404, description = "Provider not found or connection not found for tenant/provider", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    ),
    tag = "webhooks"
)]
pub async fn ingest_webhook(
    State(state): State<AppState>,
    _operator_auth: OperatorAuth,
    TenantExtension(tenant): TenantExtension,
    Path(provider_param): Path<ProviderPathParam>,
    req: Request,
) -> Result<(StatusCode, Json<WebhookAcceptResponse>), ApiError> {
    let provider_slug = provider_param.provider;
    let tenant_id = tenant.0;

    debug!(
        provider_slug = %provider_slug,
        tenant_id = %tenant_id,
        "Processing webhook ingestion"
    );

    // Extract headers first, then move request to get body
    let headers = req.headers().clone();
    let (_parts, body) = req.into_parts();
    let body_bytes = axum::body::to_bytes(body, usize::MAX).await.map_err(|_| {
        ApiError::new(
            axum::http::StatusCode::BAD_REQUEST,
            "VALIDATION_FAILED",
            "Failed to read request body",
        )
    })?;

    // Jira webhook secret verification if WEBHOOK_JIRA_SECRET is configured
    // Note: For operator-protected route, verification is optional. If configured, enforce; if not, proceed.
    if provider_slug == "jira" && state.config.webhook_jira_secret.is_some() {
        let jira_secret = state.config.webhook_jira_secret.as_ref().unwrap();
        verify_jira_webhook_secret(&headers, &body_bytes, jira_secret)?;
        debug!("Jira webhook secret verification successful");
    }

    // Filter out sensitive headers before persisting in job cursor
    // Note: For Gmail, we preserve the Authorization header for OIDC verification
    let mut sensitive_headers = std::collections::HashSet::from([
        "cookie",
        "set-cookie",
        "proxy-authorization",
        "www-authenticate",
        "authentication-info",
        "x-api-key",
        "x-auth-token",
        "x-csrf-token",
        "x-xsrf-token",
    ]);

    // Only filter authorization header for non-Gmail providers
    if provider_slug != "gmail" {
        sensitive_headers.insert("authorization");
    }

    let webhook_headers: std::collections::HashMap<String, String> = headers
        .iter()
        .filter_map(|(name, value)| {
            let name_lower = name.as_str().to_lowercase();
            // Skip sensitive headers
            if sensitive_headers.contains(&name_lower.as_str()) {
                tracing::debug!("Filtering sensitive header: {}", name_lower);
                return None;
            }

            Some((
                name_lower, // Canonical lower-case headers
                value.to_str().unwrap_or("").to_string(),
            ))
        })
        .collect();

    // Operator route: do not require Jira secret; optional verification above is sufficient.

    // Validate provider exists
    let provider_repo = ProviderRepository::new(std::sync::Arc::new(state.db.clone()));
    let _provider = provider_repo
        .find_by_slug(&provider_slug)
        .await
        .map_err(|e| {
            error!(error = ?e, "Failed to lookup provider");
            ApiError::new(
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_SERVER_ERROR",
                "Failed to validate provider",
            )
        })?
        .ok_or_else(|| {
            info!(provider_slug = %provider_slug, "Provider not found");
            ApiError::new(
                axum::http::StatusCode::NOT_FOUND,
                "NOT_FOUND",
                format!("provider '{}' not found", provider_slug),
            )
        })?;

    // Extract connection ID from headers
    let connection_id = extract_connection_id(&headers)?;

    // Extract webhook body from already read bytes
    let body = parse_webhook_body_from_bytes(&body_bytes);

    // Gmail-specific synchronous verification (OIDC and body size)
    if provider_slug == "gmail" {
        // Validate body size first to reject oversized payloads early
        validate_gmail_webhook_body_size(&body_bytes, &state.config)?;

        // Verify OIDC token synchronously
        verify_gmail_webhook_oidc(&headers, &state.config, &body_bytes)?;
    }

    // If connection ID is provided, validate it belongs to tenant and provider
    if let Some(conn_id) = connection_id {
        let connection_repo = ConnectionRepository::new(
            std::sync::Arc::new(state.db.clone()),
            state.crypto_key.clone(),
        );
        let _connection = connection_repo
            .find_by_tenant_and_provider(&tenant_id, &provider_slug)
            .await
            .map_err(|e| {
                error!(error = ?e, "Failed to validate connection");
                ApiError::new(
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_SERVER_ERROR",
                    "Failed to validate connection",
                )
            })?
            .into_iter()
            .find(|conn| conn.id == conn_id)
            .ok_or_else(|| {
                error!(
                    tenant_id = %tenant_id,
                    provider_slug = %provider_slug,
                    connection_id = %conn_id,
                    "Connection not found for tenant/provider"
                );
                ApiError::new(
                    axum::http::StatusCode::NOT_FOUND,
                    "NOT_FOUND",
                    "connection not found for tenant/provider",
                )
            })?;

        info!(
            tenant_id = %tenant_id,
            provider_slug = %provider_slug,
            connection_id = %conn_id,
            "Valid connection found, enqueuing webhook sync job"
        );

        // Create cursor with webhook context including headers and payload
        // body is Option<JsonValue> from the helper function
        let cursor = Some(serde_json::json!({
            "webhook_headers": webhook_headers,
            "webhook_payload": body,
            "received_at": chrono::Utc::now().to_rfc3339()
        }));

        // Enqueue webhook sync job
        let sync_job_repo = SyncJobRepository::new(state.db.clone());
        sync_job_repo
            .enqueue_webhook_job(tenant_id, &provider_slug, conn_id, cursor)
            .await
            .map_err(|e| {
                error!(error = ?e, "Failed to enqueue webhook sync job");
                ApiError::new(
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_SERVER_ERROR",
                    "Failed to enqueue webhook job",
                )
            })?;

        info!(
            tenant_id = %tenant_id,
            provider_slug = %provider_slug,
            connection_id = %conn_id,
            "Webhook sync job enqueued successfully"
        );
    } else {
        info!(
            tenant_id = %tenant_id,
            provider_slug = %provider_slug,
            "Webhook accepted without connection targeting"
        );
    }

    let response = WebhookAcceptResponse {
        status: "accepted".to_string(),
    };

    Ok((StatusCode::ACCEPTED, Json(response)))
}

/// Accept webhook from external provider via public route with signature verification
///
/// This endpoint receives webhook callbacks from external providers with flexible authentication:
/// 1. **Operator Auth Override**: Valid operator bearer token (`Authorization: Bearer <token>`) bypasses signature verification
/// 2. **Signature Verification**: Provider-specific signatures are verified when no operator auth is present
/// 3. **Tenant Context**: The tenant_id in the URL path provides tenant scoping
///
/// **Authentication Precedence**:
/// - If valid operator bearer token is present → Always accepted (signature verification skipped)
/// - Else if valid provider signature is present → Accepted (signature verification required)
/// - Else → Rejected with appropriate error
///
/// **Provider-Specific Requirements**:
/// - **GitHub**: `X-Hub-Signature-256: sha256=<hex>` header
/// - **Slack**: `X-Slack-Signature: v0=<hex>` and `X-Slack-Request-Timestamp` headers
/// - **Jira/Zoho-Cliq**: `Authorization: Bearer <token>` header
///
/// **Error Responses**:
/// - `401 UNAUTHORIZED`: Missing/invalid signature when no operator auth, or missing verification config
/// - `404 NOT_FOUND`: Unsupported provider
/// - `429 RATE_LIMIT_EXCEEDED`: Rate limit exceeded
/// - All errors use `application/problem+json` format with SCREAMING_SNAKE_CASE codes
#[utoipa::path(
    post,
    path = "/webhooks/{provider}/{tenant_id}",
    params(
        ("X-Connection-Id" = Option<String>, Header, description = "Optional connection ID to target"),
        ("Authorization" = Option<String>, Header, description = "Bearer token for operator auth override OR provider-specific verification (e.g., Zoho Cliq webhooks)"),
        ("X-Hub-Signature-256" = Option<String>, Header, description = "GitHub HMAC-SHA256 signature (required for GitHub webhooks without operator auth)"),
        ("X-Slack-Signature" = Option<String>, Header, description = "Slack HMAC-SHA256 signature (required for Slack webhooks without operator auth)"),
        ("X-Slack-Request-Timestamp" = Option<String>, Header, description = "Slack request timestamp (required for Slack webhooks without operator auth)"),
        ProviderTenantPath
    ),
    request_body(content = Option<JsonValue>, description = "Webhook payload (opaque to API)", content_type = "application/json"),
    responses(
        (status = 202, description = "Webhook accepted (either via operator auth or valid signature)", body = WebhookAcceptResponse),
        (status = 400, description = "Invalid connection ID header or malformed request", body = ApiError),
        (status = 401, description = "Missing/invalid signature OR webhook verification not configured", body = ApiError),
        (status = 404, description = "Provider not found or unsupported", body = ApiError),
        (status = 429, description = "Rate limit exceeded", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    ),
    tag = "webhooks"
)]
pub async fn ingest_public_webhook(
    State(state): State<AppState>,
    Path(path_params): Path<ProviderTenantPath>,
    req: Request,
) -> Result<(StatusCode, Json<WebhookAcceptResponse>), ApiError> {
    let provider_slug = path_params.provider;
    let tenant_uuid = path_params.tenant_id.parse::<Uuid>().map_err(|_| {
        ApiError::new(
            axum::http::StatusCode::BAD_REQUEST,
            "VALIDATION_FAILED",
            "Invalid tenant ID format - must be a valid UUID",
        )
    })?;
    let tenant_id = TenantId(tenant_uuid);

    debug!(
        provider_slug = %provider_slug,
        tenant_id = %tenant_id.0,
        "Processing public webhook ingestion with verified signature"
    );

    // Extract headers before consuming the request
    let headers = req.headers().clone();

    // Extract body bytes before consuming the request
    let body_bytes = match axum::body::to_bytes(req.into_body(), 1024 * 1024).await {
        Ok(bytes) => bytes,
        Err(e) => {
            tracing::error!("Failed to read webhook body: {}", e);
            return Err(ApiError::new(
                axum::http::StatusCode::BAD_REQUEST,
                "INVALID_BODY",
                "Failed to read request body",
            ));
        }
    };

    // Filter out sensitive headers before persisting in job cursor
    // Note: For Gmail, we preserve the Authorization header for OIDC verification
    let mut sensitive_headers = std::collections::HashSet::from([
        "cookie",
        "set-cookie",
        "proxy-authorization",
        "www-authenticate",
        "authentication-info",
        "x-api-key",
        "x-auth-token",
        "x-csrf-token",
        "x-xsrf-token",
        "x-hub-signature-256", // Remove signature headers from persisted data
        "x-slack-signature",
        "x-slack-request-timestamp",
        "x-webhook-secret", // Remove webhook secret headers from persisted data
    ]);

    // Only filter authorization header for non-Gmail providers
    if provider_slug != "gmail" {
        sensitive_headers.insert("authorization");
    }

    let webhook_headers: std::collections::HashMap<String, String> = headers
        .iter()
        .filter_map(|(name, value)| {
            let name_lower = name.as_str().to_lowercase();
            // Skip sensitive headers and signature headers
            if sensitive_headers.contains(&name_lower.as_str()) {
                tracing::debug!("Filtering sensitive/signature header: {}", name_lower);
                return None;
            }

            Some((
                name_lower, // Canonical lower-case headers
                value.to_str().unwrap_or("").to_string(),
            ))
        })
        .collect();

    // Validate provider exists
    let provider_repo = ProviderRepository::new(std::sync::Arc::new(state.db.clone()));
    let _provider = provider_repo
        .find_by_slug(&provider_slug)
        .await
        .map_err(|e| {
            error!(error = ?e, "Failed to lookup provider");
            ApiError::new(
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_SERVER_ERROR",
                "Failed to validate provider",
            )
        })?
        .ok_or_else(|| {
            info!(provider_slug = %provider_slug, "Provider not found");
            ApiError::new(
                axum::http::StatusCode::NOT_FOUND,
                "NOT_FOUND",
                format!("provider '{}' not found", provider_slug),
            )
        })?;

    // Extract connection ID from headers
    let connection_id = extract_connection_id(&headers)?;

    // Extract webhook body from already read bytes
    let body = parse_webhook_body_from_bytes(&body_bytes);

    // Gmail-specific synchronous verification (OIDC and body size)
    if provider_slug == "gmail" {
        // Validate body size first to reject oversized payloads early
        validate_gmail_webhook_body_size(&body_bytes, &state.config)?;

        // Verify OIDC token synchronously
        verify_gmail_webhook_oidc(&headers, &state.config, &body_bytes)?;
    }

    // If connection ID is provided, validate it belongs to tenant and provider
    if let Some(conn_id) = connection_id {
        let connection_repo = ConnectionRepository::new(
            std::sync::Arc::new(state.db.clone()),
            state.crypto_key.clone(),
        );
        let _connection = connection_repo
            .find_by_tenant_and_provider(&tenant_id.0, &provider_slug)
            .await
            .map_err(|e| {
                error!(error = ?e, "Failed to validate connection");
                ApiError::new(
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_SERVER_ERROR",
                    "Failed to validate connection",
                )
            })?
            .into_iter()
            .find(|conn| conn.id == conn_id)
            .ok_or_else(|| {
                error!(
                    tenant_id = %tenant_id.0,
                    provider_slug = %provider_slug,
                    connection_id = %conn_id,
                    "Connection not found for tenant/provider"
                );
                ApiError::new(
                    axum::http::StatusCode::NOT_FOUND,
                    "NOT_FOUND",
                    "connection not found for tenant/provider",
                )
            })?;

        info!(
            tenant_id = %tenant_id.0,
            provider_slug = %provider_slug,
            connection_id = %conn_id,
            "Valid connection found, enqueuing public webhook sync job"
        );

        // Create cursor with webhook context including headers and payload
        // body is Option<JsonValue> from the helper function
        let cursor = Some(serde_json::json!({
            "webhook_headers": webhook_headers,
            "webhook_payload": body,
            "received_at": chrono::Utc::now().to_rfc3339(),
            "verification_method": "signature"
        }));

        // Enqueue webhook sync job
        let sync_job_repo = SyncJobRepository::new(state.db.clone());
        sync_job_repo
            .enqueue_webhook_job(tenant_id.0, &provider_slug, conn_id, cursor)
            .await
            .map_err(|e| {
                error!(error = ?e, "Failed to enqueue webhook sync job");
                ApiError::new(
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_SERVER_ERROR",
                    "Failed to enqueue webhook job",
                )
            })?;

        info!(
            tenant_id = %tenant_id.0,
            provider_slug = %provider_slug,
            connection_id = %conn_id,
            "Public webhook sync job enqueued successfully"
        );
    } else {
        info!(
            tenant_id = %tenant_id.0,
            provider_slug = %provider_slug,
            "Public webhook accepted without connection targeting"
        );
    }

    let response = WebhookAcceptResponse {
        status: "accepted".to_string(),
    };

    Ok((StatusCode::ACCEPTED, Json(response)))
}

/// Generate a GitHub HMAC-SHA256 signature for testing
#[allow(dead_code)]
fn generate_github_signature(body: &str, secret: &str) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(body.as_bytes());
    let result = mac.finalize();
    let digest = hex::encode(result.into_bytes());
    format!("sha256={}", digest)
}

/// Generate a Slack v2 signature for testing
#[allow(dead_code)]
fn generate_slack_signature(body: &str, timestamp: &str, secret: &str) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    type HmacSha256 = Hmac<Sha256>;
    let base_string = format!("v0:{}:{}", timestamp, body);
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(base_string.as_bytes());
    let result = mac.finalize();
    let digest = hex::encode(result.into_bytes());
    format!("v0={}", digest)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::config::AppConfig;
    use crate::db::init_pool;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use migration::{Migrator, MigratorTrait};
    use tower::ServiceExt;

    async fn setup_test_app() -> (AppState, axum::Router) {
        let config = AppConfig {
            profile: "test".to_string(),
            operator_tokens: vec!["test-token".to_string()],
            ..Default::default()
        };

        let db = init_pool(&config).await.expect("Failed to init test DB");

        // Apply migrations
        Migrator::up(&db, None).await.unwrap();

        let state = crate::server::create_test_app_state(config, db);

        let app = crate::server::create_app(state.clone());
        (state, app)
    }

    async fn setup_test_app_with_config(config: AppConfig) -> (AppState, axum::Router) {
        let db = init_pool(&config).await.expect("Failed to init test DB");
        // Apply migrations
        Migrator::up(&db, None).await.unwrap();

        let state = crate::server::create_test_app_state(config, db);
        let app = crate::server::create_app(state.clone());
        (state, app)
    }

    async fn create_test_provider(state: &AppState, slug: &str) {
        use sea_orm::{ActiveModelTrait, Set};
        let provider_repo = ProviderRepository::new(std::sync::Arc::new(state.db.clone()));

        // Check if provider already exists to avoid constraint violations
        if provider_repo.find_by_slug(slug).await.unwrap().is_some() {
            return; // Provider already exists, skip creation
        }

        let provider = crate::models::provider::ActiveModel {
            slug: Set(slug.to_string()),
            display_name: Set(format!("Test {}", slug)),
            auth_type: Set("oauth2".to_string()),
            created_at: Set(chrono::Utc::now().fixed_offset()),
            updated_at: Set(chrono::Utc::now().fixed_offset()),
        };
        provider.insert(&state.db).await.unwrap();
    }

    async fn create_test_tenant(state: &AppState, tenant_id: Uuid) {
        use sea_orm::{ActiveModelTrait, Set};

        // Check if tenant already exists to avoid constraint violations
        use sea_orm::EntityTrait;
        if let Ok(Some(_)) = crate::models::tenant::Entity::find_by_id(tenant_id)
            .one(&state.db)
            .await
        {
            return; // Tenant already exists
        }

        let tenant = crate::models::tenant::ActiveModel {
            id: Set(tenant_id),
            name: Set(Some("Test Tenant".to_string())),
            created_at: Set(chrono::Utc::now().fixed_offset()),
        };
        tenant.insert(&state.db).await.unwrap();
    }

    async fn create_test_connection(
        state: &AppState,
        tenant_id: Uuid,
        provider_slug: &str,
    ) -> Uuid {
        use sea_orm::{ActiveModelTrait, Set};

        // Ensure tenant exists first
        create_test_tenant(state, tenant_id).await;

        let connection = crate::models::connection::ActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            provider_slug: Set(provider_slug.to_string()),
            external_id: Set("test-external-id".to_string()),
            status: Set("active".to_string()),
            display_name: Set(Some("Test Connection".to_string())),
            access_token_ciphertext: Set(Some(vec![1, 2, 3, 4])),
            refresh_token_ciphertext: Set(None),
            expires_at: Set(None),
            scopes: Set(None),
            metadata: Set(None),
            created_at: Set(chrono::Utc::now().fixed_offset()),
            updated_at: Set(chrono::Utc::now().fixed_offset()),
        };
        let result = connection.insert(&state.db).await.unwrap();
        result.id
    }

    #[tokio::test]
    async fn test_webhook_ingest_accepts_known_provider() {
        let (state, app) = setup_test_app().await;
        create_test_provider(&state, "github").await;

        let tenant_id = Uuid::new_v4();
        let request = Request::builder()
            .method("POST")
            .uri("/webhooks/github")
            .header("Authorization", "Bearer test-token")
            .header("X-Tenant-Id", tenant_id.to_string())
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"event": "push"}"#))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        println!("Response status: {}", response.status());
        assert_eq!(response.status(), StatusCode::ACCEPTED);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let webhook_response: WebhookAcceptResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(webhook_response.status, "accepted");
    }

    #[tokio::test]
    async fn test_webhook_ingest_returns_404_for_unknown_provider() {
        let (_state, app) = setup_test_app().await;

        let tenant_id = Uuid::new_v4();
        let request = Request::builder()
            .method("POST")
            .uri("/webhooks/unknown")
            .header("Authorization", "Bearer test-token")
            .header("X-Tenant-Id", tenant_id.to_string())
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_webhook_ingest_returns_401_without_auth() {
        let (_state, app) = setup_test_app().await;

        let tenant_id = Uuid::new_v4();
        let request = Request::builder()
            .method("POST")
            .uri("/webhooks/github")
            .header("X-Tenant-Id", tenant_id.to_string())
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_webhook_ingest_returns_400_without_tenant() {
        let (_state, app) = setup_test_app().await;

        let request = Request::builder()
            .method("POST")
            .uri("/webhooks/github")
            .header("Authorization", "Bearer test-token")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_webhook_ingest_returns_400_for_invalid_connection_id() {
        let (state, app) = setup_test_app().await;
        create_test_provider(&state, "github").await;

        let tenant_id = Uuid::new_v4();
        let request = Request::builder()
            .method("POST")
            .uri("/webhooks/github")
            .header("Authorization", "Bearer test-token")
            .header("X-Tenant-Id", tenant_id.to_string())
            .header("X-Connection-Id", "not-a-uuid")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_webhook_ingest_returns_404_for_invalid_connection() {
        let (state, app) = setup_test_app().await;
        create_test_provider(&state, "github").await;

        let tenant_id = Uuid::new_v4();
        let connection_id = Uuid::new_v4();
        let request = Request::builder()
            .method("POST")
            .uri("/webhooks/github")
            .header("Authorization", "Bearer test-token")
            .header("X-Tenant-Id", tenant_id.to_string())
            .header("X-Connection-Id", connection_id.to_string())
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_public_webhook_jira_requires_secret_when_configured() {
        // Configure test profile with Jira webhook secret
        let mut config = AppConfig::default();
        config.profile = "test".to_string();
        config.operator_tokens = vec!["test-token".to_string()];
        config.webhook_jira_secret = Some("test-secret-123".to_string());

        let (state, app) = setup_test_app_with_config(config).await;

        // Ensure Jira provider exists
        create_test_provider(&state, "jira").await;

        let tenant_id = Uuid::new_v4();

        // Missing secret → 401
        let request = Request::builder()
            .method("POST")
            .uri(format!("/webhooks/jira/{}", tenant_id))
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"event": "jira:issue_updated"}"#))
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        // Wrong secret → 401
        let request = Request::builder()
            .method("POST")
            .uri(format!("/webhooks/jira/{}", tenant_id))
            .header("Content-Type", "application/json")
            .header("Authorization", "Bearer wrong")
            .body(Body::from(r#"{"event": "jira:issue_updated"}"#))
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        // Correct secret (Authorization: Bearer) → 202
        let request = Request::builder()
            .method("POST")
            .uri(format!("/webhooks/jira/{}", tenant_id))
            .header("Content-Type", "application/json")
            .header("Authorization", "Bearer test-secret-123")
            .body(Body::from(r#"{"event": "jira:issue_updated"}"#))
            .unwrap();
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::ACCEPTED);
    }

    #[tokio::test]
    async fn test_public_webhook_jira_allows_without_secret_in_test_profile() {
        // Configure test profile without Jira webhook secret (allowed for test profile)
        let mut config = AppConfig::default();
        config.profile = "test".to_string();
        config.operator_tokens = vec!["test-token".to_string()];

        let (state, app) = setup_test_app_with_config(config).await;

        // Ensure Jira provider exists
        create_test_provider(&state, "jira").await;

        let tenant_id = Uuid::new_v4();
        let request = Request::builder()
            .method("POST")
            .uri(format!("/webhooks/jira/{}", tenant_id))
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"event": "jira:issue_updated"}"#))
            .unwrap();
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::ACCEPTED);
    }

    #[tokio::test]
    async fn test_webhook_ingest_enqueues_job_with_valid_connection() {
        let (state, app) = setup_test_app().await;
        create_test_provider(&state, "github").await;

        let tenant_id = Uuid::new_v4();
        let connection_id = create_test_connection(&state, tenant_id, "github").await;

        let request = Request::builder()
            .method("POST")
            .uri("/webhooks/github")
            .header("Authorization", "Bearer test-token")
            .header("X-Tenant-Id", tenant_id.to_string())
            .header("X-Connection-Id", connection_id.to_string())
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"event": "push", "repo": "test"}"#))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::ACCEPTED);

        // Verify sync job was created
        let sync_job_repo = SyncJobRepository::new(state.db.clone());
        let jobs = sync_job_repo
            .list_by_tenant(
                tenant_id,
                Some("github".to_string()),
                None,
                Some(10),
                Some(0),
            )
            .await
            .unwrap();

        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].job_type, "webhook");
        assert_eq!(jobs[0].connection_id, connection_id);
        assert_eq!(jobs[0].provider_slug, "github");
        assert!(jobs[0].cursor.is_some());

        // Verify that sensitive headers are filtered from the cursor
        let cursor: &serde_json::Value = jobs[0].cursor.as_ref().unwrap();
        let webhook_headers = cursor.get("webhook_headers").unwrap().as_object().unwrap();

        // Should contain webhook-specific headers
        assert!(webhook_headers.contains_key("content-type"));
        assert!(webhook_headers.contains_key("x-connection-id"));

        // Should NOT contain sensitive headers
        assert!(!webhook_headers.contains_key("authorization"));
        assert!(!webhook_headers.contains_key("cookie"));

        // Verify payload is captured
        assert!(cursor.get("webhook_payload").is_some());
        assert!(cursor.get("received_at").is_some());
    }

    #[tokio::test]
    async fn test_public_webhook_github_valid_signature_accepted() {
        let mut config = AppConfig::default();
        config.profile = "test".to_string();
        config.webhook_github_secret = Some("test-secret-123".to_string());
        config.operator_tokens = vec!["operator-token".to_string()];

        let (state, app) = setup_test_app_with_config(config).await;
        create_test_provider(&state, "github").await;

        let tenant_id = Uuid::new_v4();
        let body = r#"{"event": "push", "repository": {"name": "test"}}"#;
        let signature = generate_github_signature(body, "test-secret-123");

        let request = Request::builder()
            .method("POST")
            .uri(format!("/webhooks/github/{}", tenant_id))
            .header("Content-Type", "application/json")
            .header("X-Hub-Signature-256", signature)
            .body(Body::from(body))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::ACCEPTED);

        let response_body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let webhook_response: WebhookAcceptResponse =
            serde_json::from_slice(&response_body).unwrap();
        assert_eq!(webhook_response.status, "accepted");
    }

    #[tokio::test]
    async fn test_public_webhook_github_invalid_signature_rejected() {
        let mut config = AppConfig::default();
        config.profile = "test".to_string();
        config.webhook_github_secret = Some("test-secret-123".to_string());

        let (state, app) = setup_test_app_with_config(config).await;
        create_test_provider(&state, "github").await;

        let tenant_id = Uuid::new_v4();
        let body = r#"{"event": "push"}"#;

        let request = Request::builder()
            .method("POST")
            .uri(format!("/webhooks/github/{}", tenant_id))
            .header("Content-Type", "application/json")
            .header("X-Hub-Signature-256", "sha256=invalid_signature")
            .body(Body::from(body))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        // Verify it's a proper problem+json response
        let response_body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let error_response: serde_json::Value = serde_json::from_slice(&response_body).unwrap();
        assert_eq!(error_response["code"], "INVALID_SIGNATURE");
    }

    #[tokio::test]
    async fn test_public_webhook_github_missing_signature_rejected() {
        let mut config = AppConfig::default();
        config.profile = "test".to_string();
        config.webhook_github_secret = Some("test-secret-123".to_string());

        let (state, app) = setup_test_app_with_config(config).await;
        create_test_provider(&state, "github").await;

        let tenant_id = Uuid::new_v4();
        let body = r#"{"event": "push"}"#;

        let request = Request::builder()
            .method("POST")
            .uri(format!("/webhooks/github/{}", tenant_id))
            .header("Content-Type", "application/json")
            .body(Body::from(body))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        // Verify it's a proper problem+json response
        let response_body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let error_response: serde_json::Value = serde_json::from_slice(&response_body).unwrap();
        assert_eq!(error_response["code"], "INVALID_SIGNATURE");
    }

    #[tokio::test]
    async fn test_public_webhook_github_operator_auth_overrides_signature() {
        let mut config = AppConfig::default();
        config.profile = "test".to_string();
        config.webhook_github_secret = Some("test-secret-123".to_string());
        config.operator_tokens = vec!["operator-token".to_string()];

        let (state, app) = setup_test_app_with_config(config).await;
        create_test_provider(&state, "github").await;

        let tenant_id = Uuid::new_v4();
        let body = r#"{"event": "push"}"#;

        // Request with valid operator token but invalid/missing signature should be accepted
        let request = Request::builder()
            .method("POST")
            .uri(format!("/webhooks/github/{}", tenant_id))
            .header("Authorization", "Bearer operator-token")
            .header("Content-Type", "application/json")
            // No signature header or invalid signature - should be bypassed due to operator auth
            .body(Body::from(body))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::ACCEPTED);

        let response_body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let webhook_response: WebhookAcceptResponse =
            serde_json::from_slice(&response_body).unwrap();
        assert_eq!(webhook_response.status, "accepted");
    }

    #[tokio::test]
    async fn test_public_webhook_slack_valid_signature_accepted() {
        let mut config = AppConfig::default();
        config.profile = "test".to_string();
        config.webhook_slack_signing_secret = Some("test-slack-secret".to_string());
        config.webhook_slack_tolerance_seconds = 300;

        let (state, app) = setup_test_app_with_config(config).await;
        // Check if slack provider already exists to avoid constraint violation
        let provider_repo = ProviderRepository::new(std::sync::Arc::new(state.db.clone()));
        if provider_repo.find_by_slug("slack").await.unwrap().is_none() {
            create_test_provider(&state, "slack").await;
        }

        let tenant_id = Uuid::new_v4();
        let body = r#"{"type": "message", "text": "hello"}"#;
        let timestamp = (chrono::Utc::now().timestamp()).to_string();
        let signature = generate_slack_signature(body, &timestamp, "test-slack-secret");

        let request = Request::builder()
            .method("POST")
            .uri(format!("/webhooks/slack/{}", tenant_id))
            .header("Content-Type", "application/json")
            .header("X-Slack-Signature", signature)
            .header("X-Slack-Request-Timestamp", timestamp)
            .body(Body::from(body))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::ACCEPTED);
    }

    #[tokio::test]
    async fn test_public_webhook_slack_timestamp_too_old_rejected() {
        let mut config = AppConfig::default();
        config.profile = "test".to_string();
        config.webhook_slack_signing_secret = Some("test-slack-secret".to_string());
        config.webhook_slack_tolerance_seconds = 300;

        let (state, app) = setup_test_app_with_config(config).await;
        // Check if slack provider already exists to avoid constraint violation
        let provider_repo = ProviderRepository::new(std::sync::Arc::new(state.db.clone()));
        if provider_repo.find_by_slug("slack").await.unwrap().is_none() {
            create_test_provider(&state, "slack").await;
        }

        let tenant_id = Uuid::new_v4();
        let body = r#"{"type": "message"}"#;
        // Use a timestamp that's 10 minutes ago (outside the 300s tolerance window)
        let old_timestamp = (chrono::Utc::now().timestamp() - 600).to_string();
        let signature = generate_slack_signature(body, &old_timestamp, "test-slack-secret");

        let request = Request::builder()
            .method("POST")
            .uri(format!("/webhooks/slack/{}", tenant_id))
            .header("Content-Type", "application/json")
            .header("X-Slack-Signature", signature)
            .header("X-Slack-Request-Timestamp", old_timestamp)
            .body(Body::from(body))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        // Verify it's a proper problem+json response with REPLAY_ATTACK_DETECTED
        let response_body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let error_response: serde_json::Value = serde_json::from_slice(&response_body).unwrap();
        assert_eq!(error_response["code"], "REPLAY_ATTACK_DETECTED");
    }

    #[tokio::test]
    async fn test_public_webhook_github_secret_missing_rejected() {
        let mut config = AppConfig::default();
        config.profile = "test".to_string();
        // No GitHub secret configured

        let (state, app) = setup_test_app_with_config(config).await;
        create_test_provider(&state, "github").await;

        let tenant_id = Uuid::new_v4();
        let body = r#"{"event": "push"}"#;

        let request = Request::builder()
            .method("POST")
            .uri(format!("/webhooks/github/{}", tenant_id))
            .header("Content-Type", "application/json")
            .body(Body::from(body))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        // Verify it's a proper problem+json response
        let response_body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let error_response: serde_json::Value = serde_json::from_slice(&response_body).unwrap();
        assert_eq!(error_response["code"], "UNAUTHORIZED");
    }

    #[tokio::test]
    async fn test_public_webhook_unsupported_provider_returns_404() {
        let mut config = AppConfig::default();
        config.profile = "test".to_string();
        config.webhook_github_secret = Some("test-secret-123".to_string());

        let (state, app) = setup_test_app_with_config(config).await;
        // Only create github provider, not the unsupported one
        create_test_provider(&state, "github").await;

        let tenant_id = Uuid::new_v4();
        let body = r#"{"event": "test"}"#;

        let request = Request::builder()
            .method("POST")
            .uri(format!("/webhooks/unsupported-provider/{}", tenant_id))
            .header("Content-Type", "application/json")
            .body(Body::from(body))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        // Verify it's a proper problem+json response
        let response_body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let error_response: serde_json::Value = serde_json::from_slice(&response_body).unwrap();
        assert_eq!(error_response["code"], "NOT_FOUND");
    }
}
