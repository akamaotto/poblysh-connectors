//! Migration to create tenant_signal_configs table

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TenantSignalConfig::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(TenantSignalConfig::TenantId).uuid().primary_key())
                    .col(
                        ColumnDef::new(TenantSignalConfig::WeakSignalThreshold)
                            .float()
                            .not_null()
                            .default(0.7),
                    )
                    .col(ColumnDef::new(TenantSignalConfig::ScoringWeights).json_binary())
                    .col(ColumnDef::new(TenantSignalConfig::WebhookUrl).string())
                    .col(
                        ColumnDef::new(TenantSignalConfig::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(TenantSignalConfig::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-tenant_signal_config-tenant_id")
                            .from(TenantSignalConfig::Table, TenantSignalConfig::TenantId)
                            .to(Tenants::Table, Tenants::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TenantSignalConfig::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum TenantSignalConfig {
    Table,
    TenantId,
    WeakSignalThreshold,
    ScoringWeights,
    WebhookUrl,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Tenants {
    Table,
    Id,
}