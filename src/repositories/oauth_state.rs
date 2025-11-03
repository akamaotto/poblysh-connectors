//! # OAuth State Repository
//!
//! This module provides database operations for OAuth state management.

use chrono::{Duration, Utc};
use sea_orm::{
    ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, Set,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::models::oauth_state::{self, ActiveModel, Entity, Model};

/// Repository for OAuth state database operations
pub struct OAuthStateRepository {
    db: Arc<DatabaseConnection>,
}

impl OAuthStateRepository {
    /// Create a new OAuth state repository
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    /// Create a new OAuth state record
    pub async fn create(
        &self,
        tenant_id: Uuid,
        provider: &str,
        state: &str,
        code_verifier: Option<String>,
        expires_in_minutes: i64,
    ) -> Result<Model, sea_orm::DbErr> {
        let now = Utc::now();
        let expires_at = now + Duration::minutes(expires_in_minutes);

        let new_state = ActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            provider: Set(provider.to_string()),
            state: Set(state.to_string()),
            code_verifier: Set(code_verifier),
            expires_at: Set(expires_at),
            created_at: Set(now),
            updated_at: Set(now),
        };

        // Use raw SQL insertion to avoid SeaORM's UUID handling issues with SQLite
        use sea_orm::Statement;

        let id = new_state.id.unwrap();
        let tenant_id = new_state.tenant_id.unwrap();
        let provider = new_state.provider.unwrap();
        let state = new_state.state.unwrap();
        let code_verifier = new_state.code_verifier.unwrap();
        let expires_at = new_state.expires_at.unwrap();
        let created_at = new_state.created_at.unwrap();
        let updated_at = new_state.updated_at.unwrap();

        // Insert using raw SQL to avoid UnpackInsertId error
        let insert_query = Statement::from_sql_and_values(
            self.db.get_database_backend(),
            r#"
            INSERT INTO oauth_states (
                id, tenant_id, provider, state, code_verifier,
                expires_at, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            vec![
                id.into(),
                tenant_id.into(),
                provider.clone().into(),
                state.clone().into(),
                code_verifier.clone().into(),
                expires_at.into(),
                created_at.into(),
                updated_at.into(),
            ],
        );

        self.db.execute(insert_query).await?;

        // Create and return the model
        let oauth_state = Model {
            id,
            tenant_id,
            provider,
            state,
            code_verifier,
            expires_at,
            created_at,
            updated_at,
        };

        Ok(oauth_state)
    }

    /// Find OAuth state by tenant, provider, and state token
    pub async fn find_by_tenant_provider_state(
        &self,
        tenant_id: Uuid,
        provider: &str,
        state: &str,
    ) -> Result<Option<Model>, sea_orm::DbErr> {
        let result = Entity::find()
            .filter(oauth_state::Column::TenantId.eq(tenant_id))
            .filter(oauth_state::Column::Provider.eq(provider))
            .filter(oauth_state::Column::State.eq(state))
            .filter(oauth_state::Column::ExpiresAt.gt(Utc::now()))
            .one(&*self.db)
            .await?;

        Ok(result)
    }

    /// Find and consume an OAuth state (delete it after retrieval)
    pub async fn find_and_consume_by_tenant_provider_state(
        &self,
        tenant_id: Uuid,
        provider: &str,
        state: &str,
    ) -> Result<Option<Model>, sea_orm::DbErr> {
        let oauth_state = self
            .find_by_tenant_provider_state(tenant_id, provider, state)
            .await?;

        if let Some(ref state_model) = oauth_state {
            // Delete the state to prevent reuse
            let _ = Entity::delete_by_id(state_model.id).exec(&*self.db).await?;
        }

        Ok(oauth_state)
    }

    /// Find OAuth state by provider and state token (without tenant)
    pub async fn find_by_provider_state(
        &self,
        provider: &str,
        state: &str,
    ) -> Result<Option<Model>, sea_orm::DbErr> {
        let result = Entity::find()
            .filter(oauth_state::Column::Provider.eq(provider))
            .filter(oauth_state::Column::State.eq(state))
            // Temporarily disable expires filter for debugging expired state test
            .filter(oauth_state::Column::ExpiresAt.gt(Utc::now()))
            .one(&*self.db)
            .await?;

        Ok(result)
    }

    /// Find and consume an OAuth state by provider and state token (without tenant)
    pub async fn find_and_consume_by_provider_state(
        &self,
        provider: &str,
        state: &str,
    ) -> Result<Option<Model>, sea_orm::DbErr> {
        let oauth_state = self.find_by_provider_state(provider, state).await?;

        if let Some(ref state_model) = oauth_state {
            // Delete the state to prevent reuse
            let _ = Entity::delete_by_id(state_model.id).exec(&*self.db).await?;
        }

        Ok(oauth_state)
    }

    /// Clean up expired OAuth states
    pub async fn cleanup_expired(&self) -> Result<u64, sea_orm::DbErr> {
        let result = Entity::delete_many()
            .filter(oauth_state::Column::ExpiresAt.lt(Utc::now()))
            .exec(&*self.db)
            .await?;

        Ok(result.rows_affected)
    }

    /// Delete a specific OAuth state by ID
    pub async fn delete_by_id(&self, id: Uuid) -> Result<bool, sea_orm::DbErr> {
        let result = Entity::delete_by_id(id).exec(&*self.db).await?;
        Ok(result.rows_affected > 0)
    }

    /// Delete all OAuth states for a specific tenant
    pub async fn delete_by_tenant(&self, tenant_id: Uuid) -> Result<u64, sea_orm::DbErr> {
        let result = Entity::delete_many()
            .filter(oauth_state::Column::TenantId.eq(tenant_id))
            .exec(&*self.db)
            .await?;

        Ok(result.rows_affected)
    }

    /// Get count of active OAuth states for a tenant
    pub async fn count_by_tenant(&self, tenant_id: Uuid) -> Result<u64, sea_orm::DbErr> {
        let count = Entity::find()
            .filter(oauth_state::Column::TenantId.eq(tenant_id))
            .filter(oauth_state::Column::ExpiresAt.gt(Utc::now()))
            .count(&*self.db)
            .await?;

        Ok(count)
    }
}
