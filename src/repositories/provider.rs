//! Provider repository for database operations
//!
//! This module provides the ProviderRepository struct which encapsulates
//! SeaORM operations for the providers table with tenant-aware methods.

use anyhow::Result;
use sea_orm::{ActiveModelTrait, DatabaseConnection, DbErr, EntityTrait, QueryOrder, Set};
use std::sync::Arc;

use crate::models::provider::{self, Entity as Provider};

/// Repository for provider database operations
#[derive(Debug, Clone)]
pub struct ProviderRepository {
    /// Database connection pool
    pub db: Arc<DatabaseConnection>,
}

impl ProviderRepository {
    /// Creates a new ProviderRepository instance
    ///
    /// # Arguments
    ///
    /// * `db` - Database connection pool
    ///
    /// # Returns
    ///
    /// Returns a new ProviderRepository instance
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    /// Finds a provider by its slug
    ///
    /// # Arguments
    ///
    /// * `slug` - The unique slug identifier of the provider
    ///
    /// # Returns
    ///
    /// Returns a Result containing the provider model if found, or an error
    pub async fn find_by_slug(&self, slug: &str) -> Result<Option<provider::Model>> {
        let provider = Provider::find_by_id(slug.to_string())
            .one(&*self.db)
            .await?;
        Ok(provider)
    }

    /// Alias matching spec naming
    pub async fn get_by_slug(&self, slug: &str) -> Result<Option<provider::Model>> {
        self.find_by_slug(slug).await
    }

    /// Finds all providers
    ///
    /// # Returns
    ///
    /// Returns a Result containing a vector of all provider models
    pub async fn find_all(&self) -> Result<Vec<provider::Model>> {
        let providers = Provider::find()
            .order_by_asc(provider::Column::Slug)
            .all(&*self.db)
            .await?;
        Ok(providers)
    }

    /// Alias matching spec naming
    pub async fn list_all(&self) -> Result<Vec<provider::Model>> {
        self.find_all().await
    }

    /// Creates a new provider
    ///
    /// # Arguments
    ///
    /// * `provider` - The active model representing the provider to create
    ///
    /// # Returns
    ///
    /// Returns a Result containing the created provider model
    pub async fn create(&self, provider: provider::ActiveModel) -> Result<provider::Model> {
        let slug = provider
            .slug
            .clone()
            .take()
            .ok_or_else(|| anyhow::anyhow!("provider slug must be set"))?;

        Provider::insert(provider)
            .exec(&*self.db)
            .await
            .map_err(|err| match err {
                DbErr::RecordNotFound(_) => err,
                _ => err,
            })?;

        let fetched = Provider::find_by_id(slug.clone()).one(&*self.db).await?;
        fetched.ok_or_else(|| anyhow::anyhow!("provider '{}' not persisted", slug))
    }

    /// Updates a provider by its slug
    ///
    /// # Arguments
    ///
    /// * `slug` - The unique slug identifier of the provider to update
    /// * `provider` - The active model containing the updated fields
    ///
    /// # Returns
    ///
    /// Returns a Result containing the updated provider model, or an error if not found
    pub async fn update_by_slug(
        &self,
        slug: &str,
        provider: provider::ActiveModel,
    ) -> Result<provider::Model> {
        // Find the existing provider
        let existing_provider = Provider::find_by_id(slug.to_string())
            .one(&*self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Provider with slug '{}' not found", slug))?;

        // Create an active model from the existing provider
        let mut active_model: provider::ActiveModel = existing_provider.into();

        // Update the fields from the input provider
        if let Some(display_name) = provider.display_name.clone().take() {
            active_model.display_name = Set(display_name);
        }
        if let Some(auth_type) = provider.auth_type.clone().take() {
            active_model.auth_type = Set(auth_type);
        }

        // Update the provider
        let updated_provider = active_model.update(&*self.db).await?;
        Ok(updated_provider)
    }

    /// Upsert a provider by slug (spec requirement)
    pub async fn upsert(
        &self,
        slug: &str,
        display_name: &str,
        auth_type: &str,
    ) -> Result<provider::Model> {
        if let Some(existing) = self.find_by_slug(slug).await? {
            let mut am: provider::ActiveModel = existing.into();
            am.display_name = Set(display_name.to_string());
            am.auth_type = Set(auth_type.to_string());
            Ok(am.update(&*self.db).await?)
        } else {
            let am = provider::ActiveModel {
                slug: Set(slug.to_string()),
                display_name: Set(display_name.to_string()),
                auth_type: Set(auth_type.to_string()),
                ..Default::default()
            };
            self.create(am).await
        }
    }

    /// Deletes a provider by its slug
    ///
    /// # Arguments
    ///
    /// * `slug` - The unique slug identifier of the provider to delete
    ///
    /// # Returns
    ///
    /// Returns a Result indicating success or failure
    pub async fn delete_by_slug(&self, slug: &str) -> Result<()> {
        let delete_result = Provider::delete_by_id(slug.to_string())
            .exec(&*self.db)
            .await?;

        if delete_result.rows_affected == 0 {
            return Err(anyhow::anyhow!("Provider with slug '{}' not found", slug));
        }

        Ok(())
    }
}
