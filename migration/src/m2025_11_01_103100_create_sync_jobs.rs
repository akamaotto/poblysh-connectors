//! Migration to create the sync_jobs table.
//!
//! This migration creates the sync_jobs table which represents scheduled or webhook-triggered
//! units of work for connectors, tenant-scoped with status, cursors, and timing metadata.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::Statement;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SyncJobs::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(SyncJobs::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(SyncJobs::TenantId).uuid().not_null())
                    .col(ColumnDef::new(SyncJobs::ProviderSlug).text().not_null())
                    .col(ColumnDef::new(SyncJobs::ConnectionId).uuid().not_null())
                    .col(ColumnDef::new(SyncJobs::JobType).text().not_null())
                    .col(
                        ColumnDef::new(SyncJobs::Status)
                            .text()
                            .not_null()
                            .default("queued"),
                    )
                    .col(
                        ColumnDef::new(SyncJobs::Priority)
                            .small_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(SyncJobs::Attempts)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(SyncJobs::ScheduledAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(SyncJobs::RetryAfter)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(SyncJobs::StartedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(SyncJobs::FinishedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(ColumnDef::new(SyncJobs::Cursor).json_binary().null())
                    .col(ColumnDef::new(SyncJobs::Error).json_binary().null())
                    .col(
                        ColumnDef::new(SyncJobs::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(SyncJobs::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_sync_jobs_tenant_id")
                            .from(SyncJobs::Table, SyncJobs::TenantId)
                            .to(Tenants::Table, Tenants::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_sync_jobs_provider_slug")
                            .from(SyncJobs::Table, SyncJobs::ProviderSlug)
                            .to(Providers::Table, Providers::Slug)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_sync_jobs_connection_id")
                            .from(SyncJobs::Table, SyncJobs::ConnectionId)
                            .to(Connections::Table, Connections::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create index for picking the next ready job with priority DESC using raw SQL
        manager
            .get_connection()
            .execute(Statement::from_string(
                manager.get_database_backend(),
                "CREATE INDEX IF NOT EXISTS idx_sync_jobs_status_scheduled_priority ON sync_jobs (status, scheduled_at, priority DESC)".to_string(),
            ))
            .await?;

        // Create index for tenant/provider queue views
        manager
            .create_index(
                Index::create()
                    .name("idx_sync_jobs_tenant_provider_status_scheduled")
                    .table(SyncJobs::Table)
                    .col(SyncJobs::TenantId)
                    .col(SyncJobs::ProviderSlug)
                    .col(SyncJobs::Status)
                    .col(SyncJobs::ScheduledAt)
                    .to_owned(),
            )
            .await?;

        // Create index for per-connection queue operations
        manager
            .create_index(
                Index::create()
                    .name("idx_sync_jobs_connection_status_scheduled")
                    .table(SyncJobs::Table)
                    .col(SyncJobs::ConnectionId)
                    .col(SyncJobs::Status)
                    .col(SyncJobs::ScheduledAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop indexes first
        manager
            .drop_index(
                Index::drop()
                    .name("idx_sync_jobs_status_scheduled_priority")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_sync_jobs_tenant_provider_status_scheduled")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_sync_jobs_connection_status_scheduled")
                    .to_owned(),
            )
            .await?;

        // Then drop table
        manager
            .drop_table(Table::drop().table(SyncJobs::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum SyncJobs {
    Table,
    Id,
    TenantId,
    ProviderSlug,
    ConnectionId,
    JobType,
    Status,
    Priority,
    Attempts,
    ScheduledAt,
    RetryAfter,
    StartedAt,
    FinishedAt,
    Cursor,
    Error,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Tenants {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Providers {
    Table,
    Slug,
}

#[derive(DeriveIden)]
enum Connections {
    Table,
    Id,
}
