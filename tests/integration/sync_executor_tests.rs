//! Simple test for sync executor core functionality.

#[path = "../test_utils/mod.rs"]
mod test_utils;
use anyhow::Result;

use connectors::config::RateLimitPolicyConfig;
use connectors::connectors::registry::Registry;
use connectors::repositories::ConnectionRepository;
use connectors::seeds::seed_providers;
use connectors::sync_executor::{ExecutorConfig, SyncExecutor};
use connectors::token_refresh::TokenRefreshService;
use test_utils::{create_test_tenant, setup_test_db};

#[tokio::test]
async fn test_sync_executor_basic_functionality() -> Result<()> {
    let db = setup_test_db().await?;
    let _tenant_id = create_test_tenant(&db, None).await?;

    // Seed providers and initialize registry
    seed_providers(&db).await?;
    Registry::initialize();

    // Create executor
    let config = ExecutorConfig::default();
    let rate_limit_policy = RateLimitPolicyConfig::default();
    let registry = Registry::global().read().unwrap().clone();

    // Create required dependencies for TokenRefreshService
    let crypto_key = connectors::crypto::CryptoKey::new(vec![0u8; 32])
        .expect("Failed to create crypto key for test");
    let connection_repo = ConnectionRepository::new(std::sync::Arc::new(db.clone()), crypto_key);

    // Create TokenRefreshService
    let token_refresh_service = std::sync::Arc::new(TokenRefreshService::new(
        std::sync::Arc::new(connectors::config::AppConfig::default()),
        std::sync::Arc::new(db.clone()),
        std::sync::Arc::new(connection_repo),
        registry.clone(),
    ));

    let executor = SyncExecutor::new(
        db.clone(),
        registry,
        config,
        rate_limit_policy,
        token_refresh_service,
    );

    // Test that executor was created successfully
    assert_eq!(
        executor.config().concurrency,
        10,
        "Default concurrency should be 10"
    );
    assert_eq!(
        executor.config().claim_batch,
        50,
        "Default claim_batch should be 50"
    );

    // Test basic job execution (no jobs to run)
    let executed_count = executor
        .claim_and_run_jobs()
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    assert_eq!(
        executed_count, 0,
        "Should execute 0 jobs when none are available"
    );

    Ok(())
}
