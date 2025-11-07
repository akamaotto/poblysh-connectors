//! # Tenant Signal Configuration Repository
//!
//! This module contains the repository implementation for TenantSignalConfig entities,
//! providing tenant-scoped configuration management for signal processing.

use crate::error::RepositoryError;
use crate::models::tenant_signal_config::{
    ActiveModel as TenantConfigActiveModel, Entity as TenantConfig, Model as TenantConfigModel,
    ScoringWeights,
};
use sea_orm::{
    ActiveModelTrait, DatabaseConnection, EntityTrait, IntoActiveModel, ModelTrait, Set,
};
use uuid::Uuid;

/// Repository for TenantSignalConfig database operations
pub struct TenantSignalConfigRepository<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> TenantSignalConfigRepository<'a> {
    /// Create a new TenantSignalConfigRepository with the given database connection
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    /// Get or create tenant configuration with defaults
    pub async fn get_or_create(
        &self,
        tenant_id: Uuid,
    ) -> Result<TenantConfigModel, RepositoryError> {
        // Try to find existing config
        if let Some(config) = self.get(tenant_id).await? {
            return Ok(config);
        }

        // Create with defaults
        let config = TenantConfigActiveModel {
            tenant_id: Set(tenant_id),
            weak_signal_threshold: Set(0.7),
            scoring_weights: Set(None),
            webhook_url: Set(None),
            created_at: Set(Some(chrono::Utc::now().into())),
            updated_at: Set(Some(chrono::Utc::now().into())),
        };

        let result = config
            .insert(self.db)
            .await
            .map_err(RepositoryError::database_error)?;

        Ok(result)
    }

    /// Get tenant configuration by tenant ID
    pub async fn get(&self, tenant_id: Uuid) -> Result<Option<TenantConfigModel>, RepositoryError> {
        let config = TenantConfig::find_by_id(tenant_id)
            .one(self.db)
            .await
            .map_err(RepositoryError::database_error)?;

        Ok(config)
    }

    /// Update weak signal threshold for tenant
    pub async fn update_threshold(
        &self,
        tenant_id: Uuid,
        threshold: f32,
    ) -> Result<TenantConfigModel, RepositoryError> {
        let mut config = self.get_or_create(tenant_id).await?.into_active_model();

        config.weak_signal_threshold = Set(threshold.clamp(0.0, 1.0)); // Ensure valid range
        config.updated_at = Set(Some(chrono::Utc::now().into()));

        let result = config
            .update(self.db)
            .await
            .map_err(RepositoryError::database_error)?;

        Ok(result)
    }

    /// Update scoring weights for tenant
    pub async fn update_scoring_weights(
        &self,
        tenant_id: Uuid,
        weights: ScoringWeights,
    ) -> Result<TenantConfigModel, RepositoryError> {
        // Validate weights
        if !TenantConfigModel::validate_weights(&weights) {
            return Err(RepositoryError::validation_error(
                "Scoring weights must sum to approximately 1.0",
            ));
        }

        let mut config = self.get_or_create(tenant_id).await?.into_active_model();

        config.scoring_weights = Set(Some(serde_json::to_value(weights).unwrap()));
        config.updated_at = Set(Some(chrono::Utc::now().into()));

        let result = config
            .update(self.db)
            .await
            .map_err(RepositoryError::database_error)?;

        Ok(result)
    }

    /// Update webhook URL for tenant
    pub async fn update_webhook_url(
        &self,
        tenant_id: Uuid,
        webhook_url: Option<String>,
    ) -> Result<TenantConfigModel, RepositoryError> {
        // Validate webhook URL if provided
        if let Some(ref url) = webhook_url {
            self.validate_webhook_url(url)?;
        }

        let mut config = self.get_or_create(tenant_id).await?.into_active_model();

        config.webhook_url = Set(webhook_url);
        config.updated_at = Set(Some(chrono::Utc::now().into()));

        let result = config
            .update(self.db)
            .await
            .map_err(RepositoryError::database_error)?;

        Ok(result)
    }

    /// Get weak signal threshold for tenant (with fallback to default)
    pub async fn get_threshold(&self, tenant_id: Uuid) -> Result<f32, RepositoryError> {
        let config = self.get_or_create(tenant_id).await?;
        Ok(config.weak_signal_threshold)
    }

    /// Get scoring weights for tenant (with fallback to defaults)
    pub async fn get_scoring_weights(
        &self,
        tenant_id: Uuid,
    ) -> Result<ScoringWeights, RepositoryError> {
        let config = self.get_or_create(tenant_id).await?;
        Ok(config.get_scoring_weights())
    }

    /// Get webhook URL for tenant
    pub async fn get_webhook_url(
        &self,
        tenant_id: Uuid,
    ) -> Result<Option<String>, RepositoryError> {
        let config = self.get(tenant_id).await?;
        Ok(config.and_then(|c| c.webhook_url))
    }

    /// Delete tenant configuration
    pub async fn delete(&self, tenant_id: Uuid) -> Result<(), RepositoryError> {
        let config = TenantConfig::find_by_id(tenant_id)
            .one(self.db)
            .await
            .map_err(RepositoryError::database_error)?
            .ok_or_else(|| RepositoryError::NotFound("TenantSignalConfig not found".to_string()))?;

        config
            .delete(self.db)
            .await
            .map_err(RepositoryError::database_error)?;

        Ok(())
    }

    /// Validate webhook URL format and security requirements
    fn validate_webhook_url(&self, url: &str) -> Result<(), RepositoryError> {
        // Check URL length
        if url.len() > 2048 {
            return Err(RepositoryError::validation_error(
                "Webhook URL must be less than 2048 characters",
            ));
        }

        // Parse URL to validate format
        let parsed = url::Url::parse(url).map_err(|_| {
            RepositoryError::validation_error("Webhook URL must be a valid HTTP/HTTPS URL")
        })?;

        // Must be HTTP or HTTPS
        if !matches!(parsed.scheme(), "http" | "https") {
            return Err(RepositoryError::validation_error(
                "Webhook URL must use HTTP or HTTPS protocol",
            ));
        }

        // Must be HTTPS for security
        if parsed.scheme() != "https" {
            return Err(RepositoryError::validation_error(
                "Webhook URL must use HTTPS protocol for security",
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
    use crate::models::tenant::ActiveModel as TenantActiveModel;
    use sea_orm::ActiveModelTrait;
    use uuid::Uuid;

    async fn setup_test_tenant() -> (DatabaseConnection, Uuid) {
        let config = AppConfig {
            profile: "test".to_string(),
            ..Default::default()
        };

        let db = init_pool(&config).await.expect("Failed to init test DB");

        // Create tenant
        let tenant_id = Uuid::new_v4();
        let tenant = TenantActiveModel {
            id: sea_orm::Set(tenant_id),
            ..Default::default()
        };
        tenant.insert(&db).await.unwrap();

        (db, tenant_id)
    }

    #[tokio::test]
    async fn test_get_or_create_default_config() {
        let (db, tenant_id) = setup_test_tenant().await;
        let repo = TenantSignalConfigRepository::new(&db);

        let config = repo.get_or_create(tenant_id).await.unwrap();
        assert_eq!(config.tenant_id, tenant_id);
        assert_eq!(config.weak_signal_threshold, 0.7);
        assert!(config.scoring_weights.is_none());
        assert!(config.webhook_url.is_none());
    }

    #[tokio::test]
    async fn test_update_threshold() {
        let (db, tenant_id) = setup_test_tenant().await;
        let repo = TenantSignalConfigRepository::new(&db);

        let config = repo.update_threshold(tenant_id, 0.85).await.unwrap();
        assert_eq!(config.weak_signal_threshold, 0.85);

        // Test threshold clamping
        let config = repo.update_threshold(tenant_id, 1.5).await.unwrap();
        assert_eq!(config.weak_signal_threshold, 1.0);

        let config = repo.update_threshold(tenant_id, -0.5).await.unwrap();
        assert_eq!(config.weak_signal_threshold, 0.0);
    }

    #[tokio::test]
    async fn test_update_scoring_weights() {
        let (db, tenant_id) = setup_test_tenant().await;
        let repo = TenantSignalConfigRepository::new(&db);

        let weights = ScoringWeights {
            impact: 0.3,
            relevance: 0.2,
            novelty: 0.15,
            alignment: 0.15,
            timeliness: 0.1,
            credibility: 0.1,
        };

        let config = repo
            .update_scoring_weights(tenant_id, weights.clone())
            .await
            .unwrap();

        let retrieved_weights = config.get_scoring_weights();
        assert_eq!(retrieved_weights.impact, weights.impact);
        assert_eq!(retrieved_weights.relevance, weights.relevance);
    }

    #[tokio::test]
    async fn test_invalid_scoring_weights() {
        let (db, tenant_id) = setup_test_tenant().await;
        let repo = TenantSignalConfigRepository::new(&db);

        let invalid_weights = ScoringWeights {
            impact: 0.5,
            relevance: 0.5,
            novelty: 0.1,
            alignment: 0.1,
            timeliness: 0.1,
            credibility: 0.1,
        }; // Sum = 1.4

        let result = repo
            .update_scoring_weights(tenant_id, invalid_weights)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update_webhook_url() {
        let (db, tenant_id) = setup_test_tenant().await;
        let repo = TenantSignalConfigRepository::new(&db);

        let valid_url = "https://example.com/webhook".to_string();
        let config = repo
            .update_webhook_url(tenant_id, Some(valid_url))
            .await
            .unwrap();

        assert_eq!(
            config.webhook_url,
            Some("https://example.com/webhook".to_string())
        );

        // Test removing webhook URL
        let config = repo.update_webhook_url(tenant_id, None).await.unwrap();
        assert!(config.webhook_url.is_none());
    }

    #[tokio::test]
    async fn test_invalid_webhook_url() {
        let (db, tenant_id) = setup_test_tenant().await;
        let repo = TenantSignalConfigRepository::new(&db);

        // Test HTTP (not HTTPS)
        let result = repo
            .update_webhook_url(tenant_id, Some("http://example.com/webhook".to_string()))
            .await;
        assert!(result.is_err());

        // Test invalid URL
        let result = repo
            .update_webhook_url(tenant_id, Some("not-a-url".to_string()))
            .await;
        assert!(result.is_err());

        // Test URL too long
        let long_url = format!("https://example.com/{}", "a".repeat(2048));
        let result = repo.update_webhook_url(tenant_id, Some(long_url)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_threshold_with_fallback() {
        let (db, tenant_id) = setup_test_tenant().await;
        let repo = TenantSignalConfigRepository::new(&db);

        // Should return default threshold for new tenant
        let threshold = repo.get_threshold(tenant_id).await.unwrap();
        assert_eq!(threshold, 0.7);

        // Update threshold and verify
        repo.update_threshold(tenant_id, 0.9).await.unwrap();
        let threshold = repo.get_threshold(tenant_id).await.unwrap();
        assert_eq!(threshold, 0.9);
    }

    #[tokio::test]
    async fn test_get_scoring_weights_with_fallback() {
        let (db, tenant_id) = setup_test_tenant().await;
        let repo = TenantSignalConfigRepository::new(&db);

        // Should return default weights for new tenant
        let weights = repo.get_scoring_weights(tenant_id).await.unwrap();
        let default_weights = ScoringWeights::default();
        assert_eq!(weights.impact, default_weights.impact);
        assert_eq!(weights.relevance, default_weights.relevance);
    }
}
