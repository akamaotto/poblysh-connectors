//! Migration to create the connections table.
//!
//! This migration creates the connections table which stores tenant-scoped authorizations
//! to external providers, with support for OAuth tokens and multi-tenancy.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Connections::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Connections::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Connections::TenantId).uuid().not_null())
                    .col(ColumnDef::new(Connections::ProviderSlug).text().not_null())
                    .col(ColumnDef::new(Connections::ExternalId).text().not_null())
                    .col(ColumnDef::new(Connections::DisplayName).text().null())
                    .col(
                        ColumnDef::new(Connections::Status)
                            .text()
                            .not_null()
                            .default("active"),
                    )
                    .col(
                        ColumnDef::new(Connections::AccessTokenCiphertext)
                            .binary()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Connections::RefreshTokenCiphertext)
                            .binary()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Connections::ExpiresAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(ColumnDef::new(Connections::Scopes).json_binary().null())
                    .col(ColumnDef::new(Connections::Metadata).json_binary().null())
                    .col(
                        ColumnDef::new(Connections::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Connections::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_connections_provider_slug")
                            .from(Connections::Table, Connections::ProviderSlug)
                            .to(Providers::Table, Providers::Slug)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_connections_tenant_id")
                            .from(Connections::Table, Connections::TenantId)
                            .to(Tenants::Table, Tenants::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create composite unique index on (tenant_id, provider_slug, external_id)
        manager
            .create_index(
                Index::create()
                    .name("idx_connections_tenant_provider_external")
                    .table(Connections::Table)
                    .col(Connections::TenantId)
                    .col(Connections::ProviderSlug)
                    .col(Connections::ExternalId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Create index on tenant_id for tenant isolation queries
        manager
            .create_index(
                Index::create()
                    .name("idx_connections_tenant_id")
                    .table(Connections::Table)
                    .col(Connections::TenantId)
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
                    .name("idx_connections_tenant_provider_external")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(Index::drop().name("idx_connections_tenant_id").to_owned())
            .await?;

        // Then drop table
        manager
            .drop_table(Table::drop().table(Connections::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Connections {
    Table,
    Id,
    TenantId,
    ProviderSlug,
    ExternalId,
    DisplayName,
    Status,
    AccessTokenCiphertext,
    RefreshTokenCiphertext,
    ExpiresAt,
    Scopes,
    Metadata,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Providers {
    Table,
    Slug,
}

#[derive(DeriveIden)]
enum Tenants {
    Table,
    Id,
}
