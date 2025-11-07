//! Migration to create grounded_signals table

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(GroundedSignal::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(GroundedSignal::Id).uuid().primary_key())
                    .col(ColumnDef::new(GroundedSignal::SignalId).uuid().not_null())
                    .col(ColumnDef::new(GroundedSignal::TenantId).uuid().not_null())
                    .col(ColumnDef::new(GroundedSignal::IdempotencyKey).string())
                    .col(
                        ColumnDef::new(GroundedSignal::ScoreRelevance)
                            .float()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(GroundedSignal::ScoreNovelty)
                            .float()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(GroundedSignal::ScoreTimeliness)
                            .float()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(GroundedSignal::ScoreImpact)
                            .float()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(GroundedSignal::ScoreAlignment)
                            .float()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(GroundedSignal::ScoreCredibility)
                            .float()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(GroundedSignal::TotalScore)
                            .float()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(GroundedSignal::Status)
                            .string()
                            .not_null()
                            .default("draft"),
                    )
                    .col(
                        ColumnDef::new(GroundedSignal::Evidence)
                            .json_binary()
                            .not_null(),
                    )
                    .col(ColumnDef::new(GroundedSignal::Recommendation).string())
                    .col(
                        ColumnDef::new(GroundedSignal::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(GroundedSignal::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-grounded_signal-signal_id")
                            .from(GroundedSignal::Table, GroundedSignal::SignalId)
                            .to(Signals::Table, Signals::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-grounded_signal-tenant_id")
                            .from(GroundedSignal::Table, GroundedSignal::TenantId)
                            .to(Tenants::Table, Tenants::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes separately
        manager
            .create_index(
                Index::create()
                    .name("idx-grounded_signal-tenant_id")
                    .table(GroundedSignal::Table)
                    .col(GroundedSignal::TenantId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-grounded_signal-idempotency_key")
                    .table(GroundedSignal::Table)
                    .col(GroundedSignal::IdempotencyKey)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-grounded_signal-status")
                    .table(GroundedSignal::Table)
                    .col(GroundedSignal::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-grounded_signal-total_score")
                    .table(GroundedSignal::Table)
                    .col(GroundedSignal::TotalScore)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-grounded_signal-created_at")
                    .table(GroundedSignal::Table)
                    .col(GroundedSignal::CreatedAt)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(GroundedSignal::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum GroundedSignal {
    Table,
    Id,
    SignalId,
    TenantId,
    ScoreRelevance,
    ScoreNovelty,
    ScoreTimeliness,
    ScoreImpact,
    ScoreAlignment,
    ScoreCredibility,
    TotalScore,
    Status,
    Evidence,
    Recommendation,
    IdempotencyKey,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Signals {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Tenants {
    Table,
    Id,
}
