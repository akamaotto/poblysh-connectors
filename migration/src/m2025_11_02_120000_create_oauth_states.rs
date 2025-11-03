use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Check if we're using SQLite and apply SQLite-specific schema
        let db_backend = manager.get_database_backend();

        if db_backend == sea_orm::DatabaseBackend::Sqlite {
            // SQLite-compatible version using TEXT for UUID and INTEGER for timestamps
            manager
                .create_table(
                    Table::create()
                        .table(OAuthState::Table)
                        .if_not_exists()
                        .col(
                            ColumnDef::new(OAuthState::Id)
                                .text()
                                .not_null()
                                .primary_key(),
                        )
                        .col(ColumnDef::new(OAuthState::TenantId).text().not_null())
                        .col(ColumnDef::new(OAuthState::Provider).string().not_null())
                        .col(ColumnDef::new(OAuthState::State).string().not_null())
                        .col(ColumnDef::new(OAuthState::CodeVerifier).string().null())
                        .col(ColumnDef::new(OAuthState::ExpiresAt).timestamp().not_null())
                        .col(
                            ColumnDef::new(OAuthState::CreatedAt)
                                .timestamp()
                                .not_null()
                                .default(Expr::current_timestamp()),
                        )
                        .col(
                            ColumnDef::new(OAuthState::UpdatedAt)
                                .timestamp()
                                .not_null()
                                .default(Expr::current_timestamp()),
                        )
                        .to_owned(),
                )
                .await?;
        } else {
            // PostgreSQL version with proper UUID and timestamptz support
            manager
                .create_table(
                    Table::create()
                        .table(OAuthState::Table)
                        .if_not_exists()
                        .col(
                            ColumnDef::new(OAuthState::Id)
                                .uuid()
                                .not_null()
                                .primary_key(),
                        )
                        .col(ColumnDef::new(OAuthState::TenantId).uuid().not_null())
                        .col(ColumnDef::new(OAuthState::Provider).string().not_null())
                        .col(ColumnDef::new(OAuthState::State).string().not_null())
                        .col(ColumnDef::new(OAuthState::CodeVerifier).string().null())
                        .col(
                            ColumnDef::new(OAuthState::ExpiresAt)
                                .timestamp_with_time_zone()
                                .not_null(),
                        )
                        .col(
                            ColumnDef::new(OAuthState::CreatedAt)
                                .timestamp_with_time_zone()
                                .not_null()
                                .default(Expr::current_timestamp()),
                        )
                        .col(
                            ColumnDef::new(OAuthState::UpdatedAt)
                                .timestamp_with_time_zone()
                                .not_null()
                                .default(Expr::current_timestamp()),
                        )
                        .to_owned(),
                )
                .await?;
        }

        // Create unique index on tenant_id + provider + state
        // Note: Skip index creation for SQLite due to potential schema issues
        if db_backend != sea_orm::DatabaseBackend::Sqlite {
            manager
                .create_index(
                    Index::create()
                        .name("idx_oauth_states_tenant_provider_state")
                        .table(OAuthState::Table)
                        .col(OAuthState::TenantId)
                        .col(OAuthState::Provider)
                        .col(OAuthState::State)
                        .unique()
                        .to_owned(),
                )
                .await?;

            // Create index on expires_at for cleanup
            manager
                .create_index(
                    Index::create()
                        .name("idx_oauth_states_expires_at")
                        .table(OAuthState::Table)
                        .col(OAuthState::ExpiresAt)
                        .to_owned(),
                )
                .await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(OAuthState::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum OAuthState {
    #[sea_orm(iden = "oauth_states")]
    Table,
    Id,
    TenantId,
    Provider,
    State,
    CodeVerifier,
    ExpiresAt,
    CreatedAt,
    UpdatedAt,
}
