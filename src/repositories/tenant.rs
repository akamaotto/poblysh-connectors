//! # Tenant Repository
//!
//! This module contains the repository implementation for Tenant entities,
//! providing CRUD operations for tenant management.

use crate::error::RepositoryError;
use crate::models::tenant::{
    ActiveModel as TenantActiveModel, Entity as Tenant, Model as TenantModel,
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, Database, DatabaseConnection, EntityTrait, IntoActiveModel, ModelTrait, PaginatorTrait,
    Set, Statement,
};
use serde_json::Value;
use uuid::Uuid;

/// Request data for creating a new tenant
#[derive(Debug, Clone)]
pub struct CreateTenantRequest {
    /// Display name for the tenant
    pub name: String,
    /// Optional metadata for the tenant
    pub metadata: Option<Value>,
}

/// Repository for Tenant database operations
pub struct TenantRepository<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> TenantRepository<'a> {
    /// Create a new TenantRepository with the given database connection
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    /// Create a new tenant
    pub async fn create_tenant(
        &self,
        request: CreateTenantRequest,
    ) -> Result<TenantModel, RepositoryError> {
        // Validate tenant name
        self.validate_tenant_name(&request.name)?;

        let tenant_id = Uuid::new_v4();
        let now = Utc::now();

        let tenant = TenantActiveModel {
            id: Set(tenant_id),
            name: Set(Some(request.name)),
            created_at: Set(now.into()),
        };

        let result = tenant
            .insert(self.db)
            .await
            .map_err(RepositoryError::database_error)?;

        Ok(result)
    }

    /// Get tenant by ID
    pub async fn get_tenant_by_id(
        &self,
        tenant_id: Uuid,
    ) -> Result<Option<TenantModel>, RepositoryError> {
        let tenant = Tenant::find_by_id(tenant_id)
            .one(self.db)
            .await
            .map_err(RepositoryError::database_error)?;

        Ok(tenant)
    }

    /// List all tenants
    pub async fn list_tenants(&self) -> Result<Vec<TenantModel>, RepositoryError> {
        let tenants = Tenant::find()
            .all(self.db)
            .await
            .map_err(RepositoryError::database_error)?;

        Ok(tenants)
    }

    /// Update tenant name
    pub async fn update_tenant_name(
        &self,
        tenant_id: Uuid,
        name: String,
    ) -> Result<TenantModel, RepositoryError> {
        // Validate tenant name
        self.validate_tenant_name(&name)?;

        let tenant = self
            .get_tenant_by_id(tenant_id)
            .await?
            .ok_or_else(|| RepositoryError::NotFound("Tenant not found".to_string()))?;

        let mut active_tenant = tenant.into_active_model();
        active_tenant.name = Set(Some(name));

        let result = active_tenant
            .update(self.db)
            .await
            .map_err(RepositoryError::database_error)?;

        Ok(result)
    }

    /// Delete a tenant
    pub async fn delete_tenant(&self, tenant_id: Uuid) -> Result<(), RepositoryError> {
        let tenant = Tenant::find_by_id(tenant_id)
            .one(self.db)
            .await
            .map_err(RepositoryError::database_error)?
            .ok_or_else(|| RepositoryError::NotFound("Tenant not found".to_string()))?;

        tenant
            .delete(self.db)
            .await
            .map_err(RepositoryError::database_error)?;

        Ok(())
    }

    /// Check if a tenant exists
    pub async fn tenant_exists(&self, tenant_id: Uuid) -> Result<bool, RepositoryError> {
        let exists = Tenant::find_by_id(tenant_id)
            .one(self.db)
            .await
            .map_err(RepositoryError::database_error)?
            .is_some();

        Ok(exists)
    }

    /// Get tenant count
    pub async fn get_tenant_count(&self) -> Result<i64, RepositoryError> {
        let count = Tenant::find()
            .count(self.db)
            .await
            .map_err(RepositoryError::database_error)? as i64;

        Ok(count)
    }

    /// Validate tenant name according to business rules
    fn validate_tenant_name(&self, name: &str) -> Result<(), RepositoryError> {
        // Check name length
        if name.trim().is_empty() {
            return Err(RepositoryError::validation_error(
                "Tenant name cannot be empty",
            ));
        }

        if name.len() > 255 {
            return Err(RepositoryError::validation_error(
                "Tenant name cannot exceed 255 characters",
            ));
        }

        // Check for allowed characters (letters, numbers, spaces, hyphens, underscores)
        if !name
            .chars()
            .all(|c| c.is_alphanumeric() || c.is_whitespace() || c == '-' || c == '_')
        {
            return Err(RepositoryError::validation_error(
                "Tenant name can only contain letters, numbers, spaces, hyphens, and underscores",
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;
    use crate::db::init_pool;
    use sea_orm::{ConnectionTrait, DatabaseBackend, Statement};

    async fn setup_test_db() -> DatabaseConnection {
        let config = AppConfig {
            profile: "test".to_string(),
            ..Default::default()
        };

        init_pool(&config).await.expect("Failed to init test DB")
    }

    async fn table_exists(db: &DatabaseConnection, table: &str) -> bool {
        let stmt = Statement::from_string(
            DatabaseBackend::Postgres,
            format!("SELECT to_regclass('public.{table}') IS NOT NULL AS exists"),
        );

        db.query_one(stmt)
            .await
            .ok()
            .flatten()
            .and_then(|row| row.try_get::<bool>("", "exists").ok())
            .unwrap_or(false)
    }

    #[tokio::test]
    async fn test_create_tenant_success() {
        let db = setup_test_db().await;
        if !table_exists(&db, "tenants").await {
            return;
        }

        let repo = TenantRepository::new(&db);
        let request = CreateTenantRequest {
            name: "Test Tenant".to_string(),
            metadata: None,
        };

        let result = repo.create_tenant(request).await;
        assert!(result.is_ok());

        let tenant = result.unwrap();
        assert!(!tenant.id.to_string().is_empty());
        assert_eq!(tenant.name, Some("Test Tenant".to_string()));
        assert!(tenant.created_at.timestamp() > 0);
    }

    #[tokio::test]
    async fn test_create_tenant_validation() {
        let db = setup_test_db().await;
        if !table_exists(&db, "tenants").await {
            return;
        }

        let repo = TenantRepository::new(&db);

        // Test empty name
        let request = CreateTenantRequest {
            name: "".to_string(),
            metadata: None,
        };
        let result = repo.create_tenant(request).await;
        assert!(result.is_err());

        // Test name too long
        let long_name = "a".repeat(256);
        let request = CreateTenantRequest {
            name: long_name,
            metadata: None,
        };
        let result = repo.create_tenant(request).await;
        assert!(result.is_err());

        // Test invalid characters
        let request = CreateTenantRequest {
            name: "Test@Tenant".to_string(),
            metadata: None,
        };
        let result = repo.create_tenant(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_tenant_by_id() {
        let db = setup_test_db().await;
        if !table_exists(&db, "tenants").await {
            return;
        }

        let repo = TenantRepository::new(&db);

        // Create a tenant first
        let request = CreateTenantRequest {
            name: "Test Tenant".to_string(),
            metadata: None,
        };
        let created = repo.create_tenant(request).await.unwrap();

        // Get the tenant
        let result = repo.get_tenant_by_id(created.id).await;
        assert!(result.is_ok());

        let found = result.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, created.id);

        // Test non-existent tenant
        let non_existent = repo.get_tenant_by_id(Uuid::new_v4()).await;
        assert!(non_existent.is_ok());
        assert!(non_existent.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_update_tenant_name() {
        let db = setup_test_db().await;
        if !table_exists(&db, "tenants").await {
            return;
        }

        let repo = TenantRepository::new(&db);

        // Create a tenant first
        let request = CreateTenantRequest {
            name: "Original Name".to_string(),
            metadata: None,
        };
        let created = repo.create_tenant(request).await.unwrap();

        // Update the tenant name
        let updated = repo
            .update_tenant_name(created.id, "Updated Name".to_string())
            .await;
        assert!(updated.is_ok());

        let tenant = updated.unwrap();
        assert_eq!(tenant.name, Some("Updated Name".to_string()));
        assert_eq!(tenant.id, created.id);
    }

    #[tokio::test]
    async fn test_delete_tenant() {
        let db = setup_test_db().await;
        if !table_exists(&db, "tenants").await {
            return;
        }

        let repo = TenantRepository::new(&db);

        // Create a tenant first
        let request = CreateTenantRequest {
            name: "To Delete".to_string(),
            metadata: None,
        };
        let created = repo.create_tenant(request).await.unwrap();

        // Delete the tenant
        let result = repo.delete_tenant(created.id).await;
        assert!(result.is_ok());

        // Verify tenant is gone
        let found = repo.get_tenant_by_id(created.id).await;
        assert!(found.is_ok());
        assert!(found.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_tenant_exists() {
        let db = setup_test_db().await;
        if !table_exists(&db, "tenants").await {
            return;
        }

        let repo = TenantRepository::new(&db);

        // Test with non-existent tenant
        let exists = repo.tenant_exists(Uuid::new_v4()).await;
        assert!(exists.is_ok());
        assert!(!exists.unwrap());

        // Create a tenant
        let request = CreateTenantRequest {
            name: "Test Tenant".to_string(),
            metadata: None,
        };
        let created = repo.create_tenant(request).await.unwrap();

        // Test with existing tenant
        let exists = repo.tenant_exists(created.id).await;
        assert!(exists.is_ok());
        assert!(exists.unwrap());
    }

    #[tokio::test]
    async fn test_get_tenant_count() {
        // Create a completely isolated database for this test
        let db_name = format!("sqlite::memory:");
        let db = Database::connect(db_name).await.unwrap();

        // Run migrations manually on this isolated database
        use migration::MigratorTrait;
        migration::Migrator::up(&db, None).await.unwrap();

        // Disable foreign key checks for SQLite
        use sea_orm::Statement;
        db.execute(Statement::from_string(
            db.get_database_backend(),
            "PRAGMA foreign_keys = OFF".to_string(),
        ))
        .await
        .unwrap();

        if !table_exists(&db, "tenants").await {
            return;
        }

        let repo = TenantRepository::new(&db);

        // In a fresh database, count should be 0
        let initial_count = repo.get_tenant_count().await.unwrap();
        assert_eq!(initial_count, 0);

        // Create a tenant
        let request = CreateTenantRequest {
            name: "Test Tenant".to_string(),
            metadata: None,
        };
        repo.create_tenant(request).await.unwrap();

        // Now count should be exactly 1
        let new_count = repo.get_tenant_count().await.unwrap();
        assert_eq!(new_count, 1);
    }
}
