use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create the oauth_states table
        manager
            .create_table(
                Table::create()
                    .table("oauth_states")
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

        // Create unique index on tenant_id + provider + state
        manager
            .create_index(
                Index::create()
                    .name("idx_oauth_states_tenant_provider_state")
                    .table("oauth_states")
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
                    .table("oauth_states")
                    .col(OAuthState::ExpiresAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table("oauth_states").to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum OAuthState {
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
