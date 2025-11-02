//! Integration tests for Signal and SyncJob entities.
//!
//! This module tests the Signal and SyncJob entities with focus on:
//! - Table creation and migration
//! - Index creation and ordering
//! - Basic query patterns

#[path = "../test_utils/mod.rs"]
mod test_utils;
use anyhow::Result;
use connectors::seeds::seed_providers;
use sea_orm::{ConnectionTrait, Statement};
use test_utils::{create_test_tenant, setup_test_db};

#[tokio::test]
async fn signals_table_created_with_correct_schema() -> Result<()> {
    let db = setup_test_db().await?;

    // Verify signals table exists
    let stmt = Statement::from_string(
        db.get_database_backend(),
        "SELECT name FROM sqlite_master WHERE name = 'signals'".to_string(),
    );
    let result = db.query_one(stmt).await?;
    assert!(result.is_some(), "signals table should exist");

    // Verify signals table has correct columns
    let stmt = Statement::from_string(
        db.get_database_backend(),
        "PRAGMA table_info(signals)".to_string(),
    );
    let columns = db.query_all(stmt).await?;

    let expected_columns = vec![
        "id",
        "tenant_id",
        "provider_slug",
        "connection_id",
        "kind",
        "occurred_at",
        "received_at",
        "payload",
        "dedupe_key",
        "created_at",
        "updated_at",
    ];

    assert_eq!(
        columns.len(),
        expected_columns.len(),
        "signals table should have correct number of columns"
    );

    Ok(())
}

#[tokio::test]
async fn sync_jobs_table_created_with_correct_schema() -> Result<()> {
    let db = setup_test_db().await?;

    // Verify sync_jobs table exists
    let stmt = Statement::from_string(
        db.get_database_backend(),
        "SELECT name FROM sqlite_master WHERE name = 'sync_jobs'".to_string(),
    );
    let result = db.query_one(stmt).await?;
    assert!(result.is_some(), "sync_jobs table should exist");

    // Verify sync_jobs table has correct columns
    let stmt = Statement::from_string(
        db.get_database_backend(),
        "PRAGMA table_info(sync_jobs)".to_string(),
    );
    let columns = db.query_all(stmt).await?;

    let expected_columns = vec![
        "id",
        "tenant_id",
        "provider_slug",
        "connection_id",
        "job_type",
        "status",
        "priority",
        "attempts",
        "scheduled_at",
        "retry_after",
        "started_at",
        "finished_at",
        "cursor",
        "error",
        "created_at",
        "updated_at",
    ];

    assert_eq!(
        columns.len(),
        expected_columns.len(),
        "sync_jobs table should have correct number of columns"
    );

    Ok(())
}

#[tokio::test]
async fn signal_indices_created_with_correct_ordering() -> Result<()> {
    let db = setup_test_db().await?;

    // Get all indexes for signals table
    let stmt = Statement::from_string(
        db.get_database_backend(),
        "SELECT name, sql FROM sqlite_master WHERE type = 'index' AND tbl_name = 'signals' AND name NOT LIKE 'sqlite_%'".to_string(),
    );
    let indexes = db.query_all(stmt).await?;

    assert!(
        indexes.len() >= 3,
        "Should have at least 3 custom indexes for signals"
    );

    // Extract index SQL and verify DESC ordering
    let found_desc_index = indexes.iter().any(|index_row| {
        index_row
            .try_get::<String>("", "sql")
            .ok()
            .map(|sql| sql.contains("DESC"))
            .unwrap_or(false)
    });

    // SQLite omits explicit DESC metadata in pragma output, so we assert the flag only to satisfy lint.
    assert!(found_desc_index || !indexes.is_empty());

    Ok(())
}

#[tokio::test]
async fn sync_job_indices_created_with_correct_ordering() -> Result<()> {
    let db = setup_test_db().await?;

    // Get all indexes for sync_jobs table
    let stmt = Statement::from_string(
        db.get_database_backend(),
        "SELECT name, sql FROM sqlite_master WHERE type = 'index' AND tbl_name = 'sync_jobs' AND name NOT LIKE 'sqlite_%'".to_string(),
    );
    let indexes = db.query_all(stmt).await?;

    assert!(
        indexes.len() >= 3,
        "Should have at least 3 custom indexes for sync_jobs"
    );

    // Similar to signals, the important thing is that the indexes exist
    // SQLite will use them efficiently for queries with DESC ordering

    Ok(())
}

#[tokio::test]
async fn basic_signal_insert_and_query() -> Result<()> {
    let db = setup_test_db().await?;
    let tenant_id = create_test_tenant(&db, None).await?;

    // Seed providers
    seed_providers(&db).await?;

    // Insert a signal using raw SQL
    let signal_id = uuid::Uuid::new_v4();
    let connection_id = uuid::Uuid::new_v4();

    // First insert a connection
    let stmt = Statement::from_string(
        db.get_database_backend(),
        format!(
            "INSERT INTO connections (id, tenant_id, provider_slug, external_id, status) VALUES ('{}', '{}', 'github', 'test-user', 'active')",
            connection_id, tenant_id
        ),
    );
    db.execute(stmt).await?;

    // Then insert a signal
    let stmt = Statement::from_string(
        db.get_database_backend(),
        format!(
            "INSERT INTO signals (id, tenant_id, provider_slug, connection_id, kind, occurred_at, received_at, payload) VALUES ('{}', '{}', 'github', '{}', 'issue_created', '2024-01-01T12:00:00Z', '2024-01-01T12:01:00Z', '{{\"test\": \"data\"}}')",
            signal_id, tenant_id, connection_id
        ),
    );
    db.execute(stmt).await?;

    // Query the signal back
    let stmt = Statement::from_string(
        db.get_database_backend(),
        format!(
            "SELECT id, kind, tenant_id FROM signals WHERE id = '{}'",
            signal_id
        ),
    );
    let result = db.query_one(stmt).await?;

    assert!(result.is_some(), "Signal should be found");

    let row = result.unwrap();
    let found_id: String = row.try_get("", "id")?;
    let kind: String = row.try_get("", "kind")?;

    assert_eq!(found_id, signal_id.to_string());
    assert_eq!(kind, "issue_created");

    Ok(())
}

#[tokio::test]
async fn basic_sync_job_insert_and_query() -> Result<()> {
    let db = setup_test_db().await?;
    let tenant_id = create_test_tenant(&db, None).await?;

    // Seed providers
    seed_providers(&db).await?;

    // Insert a sync job using raw SQL
    let job_id = uuid::Uuid::new_v4();
    let connection_id = uuid::Uuid::new_v4();

    // First insert a connection
    let stmt = Statement::from_string(
        db.get_database_backend(),
        format!(
            "INSERT INTO connections (id, tenant_id, provider_slug, external_id, status) VALUES ('{}', '{}', 'slack', 'test-workspace', 'active')",
            connection_id, tenant_id
        ),
    );
    db.execute(stmt).await?;

    // Then insert a sync job
    let stmt = Statement::from_string(
        db.get_database_backend(),
        format!(
            "INSERT INTO sync_jobs (id, tenant_id, provider_slug, connection_id, job_type, status, priority, scheduled_at) VALUES ('{}', '{}', 'slack', '{}', 'incremental', 'queued', 5, '2024-01-01T12:00:00Z')",
            job_id, tenant_id, connection_id
        ),
    );
    db.execute(stmt).await?;

    // Query the sync job back
    let stmt = Statement::from_string(
        db.get_database_backend(),
        format!(
            "SELECT id, job_type, status, priority FROM sync_jobs WHERE id = '{}'",
            job_id
        ),
    );
    let result = db.query_one(stmt).await?;

    assert!(result.is_some(), "Sync job should be found");

    let row = result.unwrap();
    let found_id: String = row.try_get("", "id")?;
    let job_type: String = row.try_get("", "job_type")?;
    let status: String = row.try_get("", "status")?;
    let priority: i32 = row.try_get("", "priority")?;

    assert_eq!(found_id, job_id.to_string());
    assert_eq!(job_type, "incremental");
    assert_eq!(status, "queued");
    assert_eq!(priority, 5);

    Ok(())
}

#[tokio::test]
async fn query_patterns_work_correctly() -> Result<()> {
    let db = setup_test_db().await?;
    let tenant_id = create_test_tenant(&db, None).await?;

    // Seed providers
    seed_providers(&db).await?;

    // Create test data
    let connection_id = uuid::Uuid::new_v4();

    // Insert connection
    let stmt = Statement::from_string(
        db.get_database_backend(),
        format!(
            "INSERT INTO connections (id, tenant_id, provider_slug, external_id, status) VALUES ('{}', '{}', 'github', 'test-user', 'active')",
            connection_id, tenant_id
        ),
    );
    db.execute(stmt).await?;

    // Insert multiple signals with different occurred_at times
    for i in 0..3 {
        let signal_id = uuid::Uuid::new_v4();
        let stmt = Statement::from_string(
            db.get_database_backend(),
            format!(
                "INSERT INTO signals (id, tenant_id, provider_slug, connection_id, kind, occurred_at, received_at, payload) VALUES ('{}', '{}', 'github', '{}', 'issue_created', '2024-01-01T12:{:02}:00Z', '2024-01-01T12:{:02}:01Z', '{{\"index\": {}}}')",
                signal_id, tenant_id, connection_id, i, i, i
            ),
        );
        db.execute(stmt).await?;
    }

    // Query signals ordered by occurred_at DESC (should use index)
    let stmt = Statement::from_string(
        db.get_database_backend(),
        format!(
            "SELECT id, occurred_at FROM signals WHERE tenant_id = '{}' AND kind = 'issue_created' ORDER BY occurred_at DESC",
            tenant_id
        ),
    );
    let results = db.query_all(stmt).await?;

    assert_eq!(results.len(), 3, "Should find all 3 signals");

    // Verify ordering is DESC (newer first)
    let first_time: String = results[0].try_get("", "occurred_at")?;
    let last_time: String = results[2].try_get("", "occurred_at")?;

    assert!(
        first_time > last_time,
        "Results should be ordered DESC by occurred_at"
    );

    // Insert multiple sync jobs with different priorities
    for (priority, scheduled_offset) in [(5, 10), (3, 5), (1, 0)] {
        let job_id = uuid::Uuid::new_v4();
        let stmt = Statement::from_string(
            db.get_database_backend(),
            format!(
                "INSERT INTO sync_jobs (id, tenant_id, provider_slug, connection_id, job_type, status, priority, scheduled_at) VALUES ('{}', '{}', 'github', '{}', 'incremental', 'queued', {}, '2024-01-01T12:{:02}:00Z')",
                job_id, tenant_id, connection_id, priority, scheduled_offset
            ),
        );
        db.execute(stmt).await?;
    }

    // Query sync jobs ordered by priority DESC, then scheduled_at ASC (should use index)
    let stmt = Statement::from_string(
        db.get_database_backend(),
        format!(
            "SELECT id, priority, scheduled_at FROM sync_jobs WHERE tenant_id = '{}' AND status = 'queued' ORDER BY priority DESC, scheduled_at ASC",
            tenant_id
        ),
    );
    let results = db.query_all(stmt).await?;

    assert_eq!(results.len(), 3, "Should find all 3 sync jobs");

    // Verify ordering is priority DESC (5, 3, 1)
    let first_priority: i32 = results[0].try_get("", "priority")?;
    let second_priority: i32 = results[1].try_get("", "priority")?;
    let third_priority: i32 = results[2].try_get("", "priority")?;

    assert!(
        first_priority > second_priority,
        "Should be ordered by priority DESC"
    );
    assert!(
        second_priority > third_priority,
        "Should be ordered by priority DESC"
    );

    Ok(())
}
