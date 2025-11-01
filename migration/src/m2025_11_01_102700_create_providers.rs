//! Migration to create the providers table.
//!
//! This migration creates the providers table which serves as a global catalog
//! of external service providers with slug-based primary keys and OAuth metadata.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Providers::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Providers::Slug)
                            .text()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Providers::DisplayName).text().not_null())
                    .col(ColumnDef::new(Providers::AuthType).text().not_null())
                    .col(
                        ColumnDef::new(Providers::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Providers::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Providers::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Providers {
    Table,
    Slug,
    DisplayName,
    AuthType,
    CreatedAt,
    UpdatedAt,
}
