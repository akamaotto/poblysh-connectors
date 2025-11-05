//! Adds a partial unique index preventing duplicate incremental interval jobs.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::{DatabaseBackend, Statement};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let backend = manager.get_database_backend();
        match backend {
            DatabaseBackend::Postgres => {
                manager
                    .get_connection()
                    .execute(Statement::from_string(
                        backend,
                        "DO $$\nBEGIN\n    IF NOT EXISTS (\n        SELECT 1 FROM pg_indexes\n        WHERE schemaname = current_schema()\n          AND indexname = 'idx_sync_jobs_incremental_pending'\n    ) THEN\n        CREATE UNIQUE INDEX idx_sync_jobs_incremental_pending\n            ON sync_jobs (connection_id)\n            WHERE job_type = 'incremental'\n              AND status IN ('queued','running');\n    END IF;\nEND\n$$;"
                            .to_string(),
                    ))
                    .await
                    .map(|_| ())
            }
            _ => manager
                .get_connection()
                .execute(Statement::from_string(
                    backend,
                    "CREATE UNIQUE INDEX IF NOT EXISTS idx_sync_jobs_incremental_pending \
                     ON sync_jobs (connection_id) \
                     WHERE job_type = 'incremental' AND status IN ('queued','running')"
                        .to_string(),
                ))
                .await
                .map(|_| ()),
        }
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.get_connection()
            .execute(Statement::from_string(
                manager.get_database_backend(),
                "DROP INDEX IF EXISTS idx_sync_jobs_incremental_pending",
            ))
            .await
            .map(|_| ())
    }
}
