//! Migration to create the signals table.
//!
//! This migration creates the signals table which stores normalized events emitted
//! by connectors, tenant-scoped and queryable by provider, kind, and time.

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
                    .table(Signals::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Signals::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Signals::TenantId).uuid().not_null())
                    .col(ColumnDef::new(Signals::ProviderSlug).text().not_null())
                    .col(ColumnDef::new(Signals::ConnectionId).uuid().not_null())
                    .col(ColumnDef::new(Signals::Kind).text().not_null())
                    .col(
                        ColumnDef::new(Signals::OccurredAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Signals::ReceivedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(Signals::Payload).json_binary().not_null())
                    .col(ColumnDef::new(Signals::DedupeKey).text().null())
                    .col(
                        ColumnDef::new(Signals::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Signals::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_signals_tenant_id")
                            .from(Signals::Table, Signals::TenantId)
                            .to(Tenants::Table, Tenants::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_signals_provider_slug")
                            .from(Signals::Table, Signals::ProviderSlug)
                            .to(Providers::Table, Providers::Slug)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_signals_connection_id")
                            .from(Signals::Table, Signals::ConnectionId)
                            .to(Connections::Table, Connections::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create index for provider time-range queries with occurred_at DESC using raw SQL
        manager
            .get_connection()
            .execute(Statement::from_string(
                manager.get_database_backend(),
                "CREATE INDEX IF NOT EXISTS idx_signals_tenant_provider_occurred ON signals (tenant_id, provider_slug, occurred_at DESC)".to_string(),
            ))
            .await?;

        // Create index for kind-filtered queries with occurred_at DESC using raw SQL
        manager
            .get_connection()
            .execute(Statement::from_string(
                manager.get_database_backend(),
                "CREATE INDEX IF NOT EXISTS idx_signals_tenant_kind_occurred ON signals (tenant_id, kind, occurred_at DESC)".to_string(),
            ))
            .await?;

        // Create index for per-connection exploration with occurred_at DESC using raw SQL
        manager
            .get_connection()
            .execute(Statement::from_string(
                manager.get_database_backend(),
                "CREATE INDEX IF NOT EXISTS idx_signals_connection_occurred ON signals (connection_id, occurred_at DESC)".to_string(),
            ))
            .await?;

        // Create index for future dedupe checks
        manager
            .create_index(
                Index::create()
                    .name("idx_signals_tenant_provider_dedupe")
                    .table(Signals::Table)
                    .col(Signals::TenantId)
                    .col(Signals::ProviderSlug)
                    .col(Signals::DedupeKey)
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
                    .name("idx_signals_tenant_provider_occurred")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_signals_tenant_kind_occurred")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_signals_connection_occurred")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_signals_tenant_provider_dedupe")
                    .to_owned(),
            )
            .await?;

        // Then drop table
        manager
            .drop_table(Table::drop().table(Signals::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Signals {
    Table,
    Id,
    TenantId,
    ProviderSlug,
    ConnectionId,
    Kind,
    OccurredAt,
    ReceivedAt,
    Payload,
    DedupeKey,
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
