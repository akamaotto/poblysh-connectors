//! # Jobs API Handlers
//!
//! This module contains handlers for listing and managing sync jobs.

use crate::auth::{OperatorAuth, TenantExtension};
use crate::cursor::{decode_generic_cursor, encode_generic_cursor};
use crate::error::{ApiError, validation_error};
use crate::models::sync_job;
use crate::repositories::SyncJobRepository;
use crate::server::AppState;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::{DateTime, Utc};
use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Query parameters for listing jobs
#[derive(Debug, Deserialize)]
pub struct ListJobsQuery {
    /// Filter by job status (one of: queued, running, succeeded, failed)
    pub status: Option<String>,
    /// Filter by provider slug
    pub provider: Option<String>,
    /// Filter by connection ID (UUID)
    pub connection_id: Option<String>,
    /// Filter by job type (one of: full, incremental, webhook)
    pub job_type: Option<String>,
    /// Filter for jobs that started after this timestamp (RFC3339)
    pub started_after: Option<String>,
    /// Filter for jobs that finished after this timestamp (RFC3339)
    pub finished_after: Option<String>,
    /// Maximum number of jobs to return (default: 50, max: 100)
    pub limit: Option<u32>,
    /// Opaque cursor for pagination
    pub cursor: Option<String>,
}

/// Documented job status values for OpenAPI enumeration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum JobStatusParam {
    Queued,
    Running,
    Succeeded,
    Failed,
}

/// Documented job type values for OpenAPI enumeration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum JobTypeParam {
    Full,
    Incremental,
    Webhook,
}

/// Job information response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct JobInfo {
    /// Unique identifier for the sync job
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub id: String,
    /// Slug of the provider this job is for
    #[schema(example = "github")]
    pub provider_slug: String,
    /// Connection identifier this job is associated with
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub connection_id: String,
    /// Type of job
    #[schema(example = "webhook")]
    pub job_type: String,
    /// Current status of the job
    #[schema(example = "queued")]
    pub status: String,
    /// Job priority for scheduling
    #[schema(example = 50)]
    pub priority: i16,
    /// Number of attempts made for this job
    #[schema(example = 0)]
    pub attempts: i32,
    /// Timestamp when the job is scheduled to run
    #[schema(example = "2021-01-01T00:00:00Z")]
    pub scheduled_at: String,
    /// Timestamp when the job becomes eligible for retry after backoff
    #[schema(example = "2021-01-01T00:05:00Z")]
    pub retry_after: Option<String>,
    /// Timestamp when the job started execution
    #[schema(example = "2021-01-01T00:00:01Z")]
    pub started_at: Option<String>,
    /// Timestamp when the job finished execution
    #[schema(example = "2021-01-01T00:02:30Z")]
    pub finished_at: Option<String>,
}

/// Response payload for jobs listing endpoint
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct JobsResponse {
    /// List of jobs matching the query
    pub jobs: Vec<JobInfo>,
    /// Opaque cursor for fetching the next page (null if no more pages)
    pub next_cursor: Option<String>,
}

impl From<sync_job::Model> for JobInfo {
    fn from(model: sync_job::Model) -> Self {
        Self {
            id: model.id.to_string(),
            provider_slug: model.provider_slug,
            connection_id: model.connection_id.to_string(),
            job_type: model.job_type,
            status: model.status,
            priority: model.priority,
            attempts: model.attempts,
            scheduled_at: model.scheduled_at.to_rfc3339(),
            retry_after: model.retry_after.map(|dt| dt.to_rfc3339()),
            started_at: model.started_at.map(|dt| dt.to_rfc3339()),
            finished_at: model.finished_at.map(|dt| dt.to_rfc3339()),
        }
    }
}

/// List jobs endpoint requiring operator auth and tenant header
#[utoipa::path(
    get,
    path = "/jobs",
    security(("bearer_auth" = [])),
    params(
        ("cursor" = Option<String>, Query, description = "Pagination cursor (base64-encoded timestamp)"),
        ("limit" = Option<u32>, Query, description = "Maximum number of jobs to return (default 50, max 100)"),
        ("status" = Option<JobStatusParam>, Query, description = "Filter by job status"),
        ("provider" = Option<String>, Query, description = "Filter by provider type"),
        ("connection_id" = Option<String>, Query, description = "Filter by connection ID (UUID)"),
        ("job_type" = Option<JobTypeParam>, Query, description = "Filter by job type"),
        ("started_after" = Option<String>, Query, description = "Filter jobs that started after this ISO 8601 timestamp"),
        ("finished_after" = Option<String>, Query, description = "Filter jobs that finished after this ISO 8601 timestamp")
    ),
    responses(
        (status = 200, description = "List of jobs for the tenant", body = JobsResponse, example = json!({
            "jobs": [
                {
                    "id": "550e8400-e29b-41d4-a716-446655440000",
                    "provider_slug": "github",
                    "connection_id": "550e8400-e29b-41d4-a716-446655440001",
                    "job_type": "webhook",
                    "status": "succeeded",
                    "priority": 50,
                    "attempts": 1,
                    "scheduled_at": "2024-01-15T10:30:00Z",
                    "retry_after": null,
                    "started_at": "2024-01-15T10:30:01Z",
                    "finished_at": "2024-01-15T10:32:30Z"
                }
            ],
            "next_cursor": null
        })),
        (status = 400, description = "Invalid query parameters", body = ApiError),
        (status = 401, description = "Missing or invalid bearer token", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    ),
    tag = "jobs"
)]
pub async fn list_jobs(
    State(state): State<AppState>,
    _operator_auth: OperatorAuth,
    TenantExtension(tenant): TenantExtension,
    Query(params): Query<ListJobsQuery>,
) -> Result<Json<JobsResponse>, ApiError> {
    // Extract and validate limit
    let limit = if let Some(limit_val) = params.limit {
        if limit_val > 100 {
            return Err(validation_error(
                "Invalid limit",
                serde_json::json!({
                    "limit": "Maximum allowed limit is 100"
                }),
            ));
        } else if limit_val == 0 {
            return Err(validation_error(
                "Invalid limit",
                serde_json::json!({
                    "limit": "Minimum allowed limit is 1"
                }),
            ));
        }
        limit_val
    } else {
        50 // Default limit
    };

    // Extract and parse cursor if provided
    let cursor = if let Some(cursor_str) = &params.cursor {
        Some(parse_job_cursor(cursor_str)?)
    } else {
        None
    };

    // Validate and parse status filter
    let status_filter = if let Some(status_str) = &params.status {
        match status_str.as_str() {
            "queued" | "running" | "succeeded" | "failed" => Some(status_str.clone()),
            _ => {
                return Err(validation_error(
                    "Invalid status",
                    serde_json::json!({
                        "status": "Must be one of: queued, running, succeeded, failed"
                    }),
                ));
            }
        }
    } else {
        None
    };

    // Validate and parse job_type filter
    let job_type_filter = if let Some(job_type_str) = &params.job_type {
        match job_type_str.as_str() {
            "full" | "incremental" | "webhook" => Some(job_type_str.clone()),
            _ => {
                return Err(validation_error(
                    "Invalid job_type",
                    serde_json::json!({
                        "job_type": "Must be one of: full, incremental, webhook"
                    }),
                ));
            }
        }
    } else {
        None
    };

    // Extract provider filter
    let provider_filter = params.provider.clone();

    // Parse connection_id if provided
    let connection_id_filter = if let Some(conn_id_str) = &params.connection_id {
        Some(Uuid::parse_str(conn_id_str).map_err(|_| {
            validation_error(
                "Invalid connection_id",
                serde_json::json!({
                    "connection_id": "Must be a valid UUID"
                }),
            )
        })?)
    } else {
        None
    };

    // Parse started_after timestamp if provided
    let started_after_filter = if let Some(started_str) = &params.started_after {
        Some(
            DateTime::parse_from_rfc3339(started_str)
                .map_err(|_| {
                    validation_error(
                        "Invalid started_after format",
                        serde_json::json!({
                            "started_after": "Must be a valid ISO 8601 timestamp (RFC 3339)"
                        }),
                    )
                })?
                .with_timezone(&Utc),
        )
    } else {
        None
    };

    // Parse finished_after timestamp if provided
    let finished_after_filter = if let Some(finished_str) = &params.finished_after {
        Some(
            DateTime::parse_from_rfc3339(finished_str)
                .map_err(|_| {
                    validation_error(
                        "Invalid finished_after format",
                        serde_json::json!({
                            "finished_after": "Must be a valid ISO 8601 timestamp (RFC 3339)"
                        }),
                    )
                })?
                .with_timezone(&Utc),
        )
    } else {
        None
    };

    // List jobs using repository
    let repo = SyncJobRepository::new(state.db.clone());
    let config = crate::repositories::ListJobsConfig {
        status: status_filter,
        provider: provider_filter,
        connection_id: connection_id_filter,
        job_type: job_type_filter,
        started_after: started_after_filter,
        finished_after: finished_after_filter,
    };

    let result = repo
        .list_jobs(tenant.0, limit as i64, cursor, config)
        .await
        .map_err(|e| {
            tracing::error!("Failed to list jobs: {:?}", e);
            ApiError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_SERVER_ERROR",
                "Failed to list jobs",
            )
        })?;

    // Encode next cursor if there are more results
    let next_cursor = if let Some((scheduled_at, id)) = result.next_cursor {
        Some(encode_job_cursor(scheduled_at, id))
    } else {
        None
    };

    let response = JobsResponse {
        jobs: result.jobs.into_iter().map(JobInfo::from).collect(),
        next_cursor,
    };

    Ok(Json(response))
}

/// Encode job cursor data to standardized base64 string
fn encode_job_cursor(scheduled_at: DateTimeWithTimeZone, id: Uuid) -> String {
    let keys = serde_json::json!({
        "scheduled_at": scheduled_at.to_rfc3339(),
        "id": id.to_string()
    });
    encode_generic_cursor(keys)
}

/// Decode job cursor from standardized base64 string
fn parse_job_cursor(cursor_str: &str) -> Result<(DateTimeWithTimeZone, Uuid), ApiError> {
    let cursor = decode_generic_cursor(cursor_str).map_err(|_| {
        validation_error(
            "Invalid cursor format",
            serde_json::json!({
                "cursor": "Cursor must be valid base64-encoded JSON"
            }),
        )
    })?;

    // Extract required fields from cursor
    let scheduled_at_str = cursor.keys["scheduled_at"].as_str().ok_or_else(|| {
        validation_error(
            "Invalid cursor format",
            serde_json::json!({
                "cursor": "Cursor must contain scheduled_at field"
            }),
        )
    })?;

    let id_str = cursor.keys["id"].as_str().ok_or_else(|| {
        validation_error(
            "Invalid cursor format",
            serde_json::json!({
                "cursor": "Cursor must contain id field"
            }),
        )
    })?;

    let scheduled_at = DateTime::parse_from_rfc3339(scheduled_at_str).map_err(|_| {
        validation_error(
            "Invalid cursor format",
            serde_json::json!({
                "cursor": "scheduled_at must be a valid RFC3339 timestamp"
            }),
        )
    })?;

    let id = Uuid::parse_str(id_str).map_err(|_| {
        validation_error(
            "Invalid cursor format",
            serde_json::json!({
                "cursor": "id must be a valid UUID"
            }),
        )
    })?;

    Ok((scheduled_at, id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;
    use crate::db::init_pool;
    use crate::models::sync_job::ActiveModel;
    use crate::server::AppState;
    use axum::{
        body::Body,
        http::{Request, StatusCode, header},
    };
    use chrono::Utc;
    use sea_orm::{
        ActiveModelTrait, ConnectionTrait, DatabaseConnection, Set, prelude::DateTimeWithTimeZone,
    };
    use tower::ServiceExt;
    use uuid::Uuid;

    async fn setup_test_app() -> (AppState, DatabaseConnection, Uuid) {
        let config = AppConfig {
            profile: "test".to_string(),
            operator_tokens: vec!["test-token-123".to_string()],
            ..Default::default()
        };

        let db = init_pool(&config).await.expect("Failed to init test DB");

        // Create a test tenant for foreign key constraints
        let tenant_id = create_test_tenant(&db, None)
            .await
            .expect("Failed to create test tenant");

        let state = crate::server::create_test_app_state(config, db.clone());

        (state, db, tenant_id)
    }

    /// Creates a test tenant in the database
    async fn create_test_tenant(
        db: &DatabaseConnection,
        tenant_id: Option<uuid::Uuid>,
    ) -> Result<uuid::Uuid, sea_orm::DbErr> {
        use sea_orm::Statement;

        let id = tenant_id.unwrap_or_else(uuid::Uuid::new_v4);

        let stmt = Statement::from_string(
            db.get_database_backend(),
            format!(
                "INSERT INTO tenants (id, name) VALUES ('{}', 'Test Tenant')",
                id
            ),
        );

        db.execute(stmt).await?;
        Ok(id)
    }

    /// Creates a test provider in the database
    async fn create_test_provider(
        db: &DatabaseConnection,
        slug: &str,
        display_name: &str,
        auth_type: &str,
    ) -> Result<(), sea_orm::DbErr> {
        use sea_orm::Statement;

        let stmt = Statement::from_string(
            db.get_database_backend(),
            format!(
                "INSERT INTO providers (slug, display_name, auth_type, created_at, updated_at) VALUES ('{}', '{}', '{}', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
                slug, display_name, auth_type
            ),
        );

        db.execute(stmt).await?;
        Ok(())
    }

    /// Creates a test connection in the database
    async fn create_test_connection(
        db: &DatabaseConnection,
        tenant_id: uuid::Uuid,
        provider_slug: &str,
        connection_id: Option<uuid::Uuid>,
    ) -> Result<uuid::Uuid, sea_orm::DbErr> {
        use sea_orm::Statement;

        let id = connection_id.unwrap_or_else(uuid::Uuid::new_v4);

        let stmt = Statement::from_string(
            db.get_database_backend(),
            format!(
                "INSERT INTO connections (id, tenant_id, provider_slug, external_id, created_at, updated_at) VALUES ('{}', '{}', '{}', 'test-external-id', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
                id, tenant_id, provider_slug
            ),
        );

        db.execute(stmt).await?;
        Ok(id)
    }

    async fn create_test_job(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        provider_slug: &str,
        connection_id: Uuid,
        status: &str,
        job_type: &str,
        scheduled_at: DateTimeWithTimeZone,
    ) -> crate::models::sync_job::Model {
        let job = ActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            provider_slug: Set(provider_slug.to_string()),
            connection_id: Set(connection_id),
            job_type: Set(job_type.to_string()),
            status: Set(status.to_string()),
            priority: Set(50),
            attempts: Set(0),
            scheduled_at: Set(scheduled_at),
            retry_after: Set(None),
            started_at: Set(None),
            finished_at: Set(None),
            cursor: Set(None),
            error: Set(None),
            created_at: Set(scheduled_at),
            updated_at: Set(scheduled_at),
        };

        job.insert(db).await.expect("Failed to create test job")
    }

    #[test]
    fn test_job_cursor_encoding_roundtrip() {
        use chrono::TimeZone;
        let scheduled_at = Utc.timestamp_opt(1609459200, 0).unwrap().fixed_offset();
        let id = Uuid::new_v4();

        let encoded = encode_job_cursor(scheduled_at, id);
        let (decoded_scheduled_at, decoded_id) = parse_job_cursor(&encoded).unwrap();

        assert_eq!(scheduled_at, decoded_scheduled_at);
        assert_eq!(id, decoded_id);
    }

    #[test]
    fn test_parse_invalid_job_cursor() {
        let result = parse_job_cursor("invalid_base64!");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code.to_string(), "VALIDATION_FAILED");
    }

    #[test]
    fn test_parse_non_utf8_job_cursor() {
        // Create invalid base64 that decodes to invalid UTF-8
        use base64::{Engine as _, engine::general_purpose};
        let invalid_utf8 = general_purpose::STANDARD.encode([0xff, 0xfe, 0xfd]);
        let result = parse_job_cursor(&invalid_utf8);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code.to_string(), "VALIDATION_FAILED");
    }

    #[test]
    fn test_parse_invalid_json_job_cursor() {
        use base64::{Engine as _, engine::general_purpose};
        let invalid_json = general_purpose::STANDARD.encode("not valid json");
        let result = parse_job_cursor(&invalid_json);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code.to_string(), "VALIDATION_FAILED");
    }

    #[test]
    fn test_parse_incomplete_job_cursor() {
        // Missing 'id' field
        let incomplete_json = r#"{"version":1,"keys":{"scheduled_at":"2021-01-01T00:00:00Z"}}"#;
        use base64::{Engine as _, engine::general_purpose};
        let encoded = general_purpose::STANDARD.encode(incomplete_json);
        let result = parse_job_cursor(&encoded);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code.to_string(), "VALIDATION_FAILED");
    }

    #[tokio::test]
    async fn test_list_jobs_requires_auth() {
        let (state, _db, _tenant_id) = setup_test_app().await;
        let app = crate::server::create_app(state);

        let request = Request::builder()
            .method("GET")
            .uri("/jobs")
            .header("X-Tenant-Id", Uuid::new_v4().to_string())
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_list_jobs_requires_tenant_header() {
        let (state, _db, _tenant_id) = setup_test_app().await;
        let app = crate::server::create_app(state);

        let request = Request::builder()
            .method("GET")
            .uri("/jobs")
            .header(header::AUTHORIZATION, "Bearer test-token-123")
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_list_jobs_empty_result() {
        let (state, _db, tenant_id) = setup_test_app().await;
        let app = crate::server::create_app(state);

        let request = Request::builder()
            .method("GET")
            .uri("/jobs")
            .header(header::AUTHORIZATION, "Bearer test-token-123")
            .header("X-Tenant-Id", tenant_id.to_string())
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let jobs_response: JobsResponse = serde_json::from_slice(&body).unwrap();

        assert!(jobs_response.jobs.is_empty());
        assert!(jobs_response.next_cursor.is_none());
    }

    #[tokio::test]
    async fn test_list_jobs_with_data() {
        let (state, db, tenant_id) = setup_test_app().await;
        let connection_id = Uuid::new_v4();
        let scheduled_at = Utc::now().fixed_offset();

        // Use unique provider names to avoid conflicts
        let github_provider = format!("github-{}", Uuid::new_v4());
        let jira_provider = format!("jira-{}", Uuid::new_v4());

        // Create test providers first
        create_test_provider(&db, &github_provider, "GitHub", "oauth2")
            .await
            .expect("Failed to create GitHub provider");
        create_test_provider(&db, &jira_provider, "Jira", "oauth2")
            .await
            .expect("Failed to create Jira provider");

        // Create test connection first
        create_test_connection(&db, tenant_id, &github_provider, Some(connection_id))
            .await
            .expect("Failed to create test connection");

        // Create test jobs
        create_test_job(
            &db,
            tenant_id,
            &github_provider,
            connection_id,
            "queued",
            "webhook",
            scheduled_at,
        )
        .await;
        create_test_job(
            &db,
            tenant_id,
            &jira_provider,
            connection_id,
            "running",
            "full",
            scheduled_at,
        )
        .await;

        let app = crate::server::create_app(state);

        let request = Request::builder()
            .method("GET")
            .uri("/jobs")
            .header(header::AUTHORIZATION, "Bearer test-token-123")
            .header("X-Tenant-Id", tenant_id.to_string())
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let jobs_response: JobsResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(jobs_response.jobs.len(), 2);
        assert!(jobs_response.next_cursor.is_none()); // Only 2 jobs, default limit is 50
    }

    #[tokio::test]
    async fn test_list_jobs_with_status_filter() {
        let (state, db, tenant_id) = setup_test_app().await;
        let scheduled_at = Utc::now().fixed_offset();

        // Use unique provider names to avoid conflicts
        let github_provider = format!("github-{}", Uuid::new_v4());
        let jira_provider = format!("jira-{}", Uuid::new_v4());
        let slack_provider = format!("slack-{}", Uuid::new_v4());

        // Create test providers first
        create_test_provider(&db, &github_provider, "GitHub", "oauth2")
            .await
            .expect("Failed to create GitHub provider");
        create_test_provider(&db, &jira_provider, "Jira", "oauth2")
            .await
            .expect("Failed to create Jira provider");
        create_test_provider(&db, &slack_provider, "Slack", "oauth2")
            .await
            .expect("Failed to create Slack provider");

        // Create test connections for each provider
        let github_connection_id = Uuid::new_v4();
        let jira_connection_id = Uuid::new_v4();
        let slack_connection_id = Uuid::new_v4();

        create_test_connection(&db, tenant_id, &github_provider, Some(github_connection_id))
            .await
            .expect("Failed to create GitHub test connection");
        create_test_connection(&db, tenant_id, &jira_provider, Some(jira_connection_id))
            .await
            .expect("Failed to create Jira test connection");
        create_test_connection(&db, tenant_id, &slack_provider, Some(slack_connection_id))
            .await
            .expect("Failed to create Slack test connection");

        // Create test jobs with different statuses
        create_test_job(
            &db,
            tenant_id,
            &github_provider,
            github_connection_id,
            "queued",
            "webhook",
            scheduled_at,
        )
        .await;
        create_test_job(
            &db,
            tenant_id,
            &jira_provider,
            jira_connection_id,
            "running",
            "full",
            scheduled_at,
        )
        .await;
        create_test_job(
            &db,
            tenant_id,
            &slack_provider,
            slack_connection_id,
            "failed",
            "incremental",
            scheduled_at,
        )
        .await;

        let app = crate::server::create_app(state);

        let request = Request::builder()
            .method("GET")
            .uri("/jobs?status=queued")
            .header(header::AUTHORIZATION, "Bearer test-token-123")
            .header("X-Tenant-Id", tenant_id.to_string())
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let jobs_response: JobsResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(jobs_response.jobs.len(), 1);
        assert_eq!(jobs_response.jobs[0].status, "queued");
    }

    #[tokio::test]
    async fn test_list_jobs_with_provider_filter() {
        let (state, db, tenant_id) = setup_test_app().await;
        let connection_id = Uuid::new_v4();
        let scheduled_at = Utc::now().fixed_offset();

        // Use unique provider names to avoid conflicts
        let github_provider = format!("github-{}", Uuid::new_v4());
        let jira_provider = format!("jira-{}", Uuid::new_v4());

        // Create test providers first
        create_test_provider(&db, &github_provider, "GitHub", "oauth2")
            .await
            .expect("Failed to create GitHub provider");
        create_test_provider(&db, &jira_provider, "Jira", "oauth2")
            .await
            .expect("Failed to create Jira provider");

        // Create test connection first
        create_test_connection(&db, tenant_id, &github_provider, Some(connection_id))
            .await
            .expect("Failed to create test connection");

        // Create test jobs for different providers
        create_test_job(
            &db,
            tenant_id,
            &github_provider,
            connection_id,
            "queued",
            "webhook",
            scheduled_at,
        )
        .await;
        create_test_job(
            &db,
            tenant_id,
            &jira_provider,
            connection_id,
            "running",
            "full",
            scheduled_at,
        )
        .await;

        let app = crate::server::create_app(state);

        let request = Request::builder()
            .method("GET")
            .uri(format!("/jobs?provider={}", github_provider))
            .header(header::AUTHORIZATION, "Bearer test-token-123")
            .header("X-Tenant-Id", tenant_id.to_string())
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let jobs_response: JobsResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(jobs_response.jobs.len(), 1);
        assert_eq!(jobs_response.jobs[0].provider_slug, github_provider);
    }

    #[tokio::test]
    async fn test_list_jobs_with_invalid_connection_id() {
        let (state, _db, tenant_id) = setup_test_app().await;
        let app = crate::server::create_app(state);

        let request = Request::builder()
            .method("GET")
            .uri("/jobs?connection_id=invalid-uuid")
            .header(header::AUTHORIZATION, "Bearer test-token-123")
            .header("X-Tenant-Id", tenant_id.to_string())
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let error_response: ApiError = serde_json::from_slice(&body).unwrap();

        assert_eq!(error_response.code.to_string(), "VALIDATION_FAILED");
    }

    #[tokio::test]
    async fn test_list_jobs_with_invalid_timestamp() {
        let (state, _db, tenant_id) = setup_test_app().await;
        let app = crate::server::create_app(state);

        let request = Request::builder()
            .method("GET")
            .uri("/jobs?started_after=invalid-timestamp")
            .header(header::AUTHORIZATION, "Bearer test-token-123")
            .header("X-Tenant-Id", tenant_id.to_string())
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let error_response: ApiError = serde_json::from_slice(&body).unwrap();

        assert_eq!(error_response.code.to_string(), "VALIDATION_FAILED");
    }

    #[tokio::test]
    async fn test_list_jobs_with_limit_validation() {
        let (state, _db, tenant_id) = setup_test_app().await;
        let app = crate::server::create_app(state);

        // Test limit > 100
        let request = Request::builder()
            .method("GET")
            .uri("/jobs?limit=101")
            .header(header::AUTHORIZATION, "Bearer test-token-123")
            .header("X-Tenant-Id", tenant_id.to_string())
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let error_response: ApiError = serde_json::from_slice(&body).unwrap();

        assert_eq!(error_response.code.to_string(), "VALIDATION_FAILED");

        // Test limit = 0
        let request = Request::builder()
            .method("GET")
            .uri("/jobs?limit=0")
            .header(header::AUTHORIZATION, "Bearer test-token-123")
            .header("X-Tenant-Id", tenant_id.to_string())
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let error_response: ApiError = serde_json::from_slice(&body).unwrap();

        assert_eq!(error_response.code.to_string(), "VALIDATION_FAILED");
    }

    #[tokio::test]
    async fn test_list_jobs_tenant_isolation() {
        let (state, db, tenant1_id) = setup_test_app().await;
        let tenant2_id = Uuid::new_v4();
        let scheduled_at = Utc::now().fixed_offset();

        // Use unique provider name to avoid conflicts
        let github_provider = format!("github-{}", Uuid::new_v4());

        // Create test provider first
        create_test_provider(&db, &github_provider, "GitHub", "oauth2")
            .await
            .expect("Failed to create GitHub provider");

        // Create second tenant for isolation test
        create_test_tenant(&db, Some(tenant2_id))
            .await
            .expect("Failed to create second test tenant");

        // Create test connections for both tenants with unique IDs
        let connection1_id = Uuid::new_v4();
        let connection2_id = Uuid::new_v4();

        create_test_connection(&db, tenant1_id, &github_provider, Some(connection1_id))
            .await
            .expect("Failed to create first test connection");
        create_test_connection(&db, tenant2_id, &github_provider, Some(connection2_id))
            .await
            .expect("Failed to create second test connection");

        // Create jobs for two different tenants
        create_test_job(
            &db,
            tenant1_id,
            &github_provider,
            connection1_id,
            "queued",
            "webhook",
            scheduled_at,
        )
        .await;
        create_test_job(
            &db,
            tenant2_id,
            &github_provider,
            connection2_id,
            "queued",
            "webhook",
            scheduled_at,
        )
        .await;

        let app = crate::server::create_app(state);

        // Query with tenant1
        let request = Request::builder()
            .method("GET")
            .uri("/jobs")
            .header(header::AUTHORIZATION, "Bearer test-token-123")
            .header("X-Tenant-Id", tenant1_id.to_string())
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let jobs_response: JobsResponse = serde_json::from_slice(&body).unwrap();

        // Should only return jobs for tenant1
        assert_eq!(jobs_response.jobs.len(), 1);
    }
}
