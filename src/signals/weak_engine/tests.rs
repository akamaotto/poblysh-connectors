//! Integration tests for the weak signal engine

use crate::config::AppConfig;
use crate::db::init_pool;
use crate::models::signal::ActiveModel as SignalActiveModel;
use crate::models::tenant::ActiveModel as TenantActiveModel;
use crate::signals::weak_engine::{WeakSignalEngine, WeakSignalEngineConfig};
use chrono::Utc;
use sea_orm::ActiveModelTrait;
use std::sync::Arc;
use uuid::Uuid;

#[tokio::test]
async fn test_weak_signal_engine_end_to_end() {
    let config = AppConfig {
        profile: "test".to_string(),
        ..Default::default()
    };

    let db = Arc::new(init_pool(&config).await.expect("Failed to init test DB"));

    // Create tenant
    let tenant_id = Uuid::new_v4();
    let tenant = TenantActiveModel {
        id: sea_orm::Set(tenant_id),
        ..Default::default()
    };
    tenant.insert(&*db).await.unwrap();

    // Create a signal that should pass the threshold
    let signal_payload = serde_json::json!({
        "title": "Critical security vulnerability discovered",
        "description": "A severe security issue was found in the authentication system requiring immediate attention",
        "tags": ["security", "critical", "urgent"],
        "user": {
            "authority": "admin"
        },
        "audience_size": 50000
    });

    let signal = SignalActiveModel {
        id: sea_orm::Set(Uuid::new_v4()),
        tenant_id: sea_orm::Set(tenant_id),
        provider_slug: sea_orm::Set("github".to_string()),
        kind: sea_orm::Set("security_alert".to_string()),
        occurred_at: sea_orm::Set(Utc::now().into()),
        received_at: sea_orm::Set(Utc::now().into()),
        payload: sea_orm::Set(signal_payload),
        ..Default::default()
    };

    let signal_model = signal.insert(&*db).await.unwrap();

    // Set up weak signal engine with low threshold for testing
    let engine_config = WeakSignalEngineConfig {
        default_threshold: 0.5, // Low threshold for testing
        batch_size: 10,
        max_signal_age_hours: 24,
        cluster_window_hours: 6,
        cluster_similarity_threshold: 0.8,
        enable_notifications: false, // Disable notifications for test
        webhook_timeout_seconds: 10,
    };

    let engine = WeakSignalEngine::new(db.clone(), engine_config);

    // Process signals - should create grounded signal
    engine.process_signals().await.unwrap();

    // Verify that a grounded signal was created
    use crate::repositories::GroundedSignalRepository;
    let grounded_repo = GroundedSignalRepository::new(&*db);

    let grounded_signals = grounded_repo
        .list(crate::repositories::ListGroundedSignalsQuery {
            tenant_id,
            status: None,
            min_score: None,
            limit: None,
            offset: None,
        })
        .await
        .unwrap();

    // Should have at least one grounded signal due to high security score
    assert!(
        !grounded_signals.data.is_empty(),
        "Expected at least one grounded signal to be created"
    );

    let created_signal = &grounded_signals.data[0];
    assert_eq!(created_signal.signal_id, signal_model.id);
    assert_eq!(created_signal.tenant_id, tenant_id);
    assert_eq!(
        created_signal.status,
        crate::models::GroundedSignalStatus::Recommended
    );
    assert!(
        created_signal.scores.total >= 0.5,
        "Expected score to exceed threshold"
    );

    // Verify evidence contains security-related keywords
    let evidence = &created_signal.evidence;
    if let Some(keywords) = evidence.get("keywords").and_then(|k| k.as_array()) {
        assert!(
            keywords.iter().any(|kw| {
                if let Some(kw_str) = kw.as_str() {
                    kw_str.contains("security")
                        || kw_str.contains("critical")
                        || kw_str.contains("urgent")
                } else {
                    false
                }
            }),
            "Expected evidence to contain security-related keywords"
        );
    }

    // Verify recommendation is provided for high-security signals
    assert!(
        created_signal.recommendation.is_some(),
        "Expected recommendation for high-scoring security signal"
    );
}

#[tokio::test]
async fn test_weak_signal_engine_below_threshold() {
    let config = AppConfig {
        profile: "test".to_string(),
        ..Default::default()
    };

    let db = Arc::new(init_pool(&config).await.expect("Failed to init test DB"));

    // Create tenant
    let tenant_id = Uuid::new_v4();
    let tenant = TenantActiveModel {
        id: sea_orm::Set(tenant_id),
        ..Default::default()
    };
    tenant.insert(&*db).await.unwrap();

    // Create a low-impact signal that should NOT pass the threshold
    let signal_payload = serde_json::json!({
        "title": "Minor update to documentation",
        "description": "Updated some comments in the README file",
        "tags": ["documentation", "minor"],
        "user": {
            "authority": "contributor"
        }
    });

    let signal = SignalActiveModel {
        id: sea_orm::Set(Uuid::new_v4()),
        tenant_id: sea_orm::Set(tenant_id),
        provider_slug: sea_orm::Set("github".to_string()),
        kind: sea_orm::Set("documentation".to_string()),
        occurred_at: sea_orm::Set(Utc::now().into()),
        received_at: sea_orm::Set(Utc::now().into()),
        payload: sea_orm::Set(signal_payload),
        ..Default::default()
    };

    signal.insert(&*db).await.unwrap();

    // Set up weak signal engine with high threshold
    let engine_config = WeakSignalEngineConfig {
        default_threshold: 0.9, // High threshold
        batch_size: 10,
        max_signal_age_hours: 24,
        cluster_window_hours: 6,
        cluster_similarity_threshold: 0.8,
        enable_notifications: false,
        webhook_timeout_seconds: 10,
    };

    let engine = WeakSignalEngine::new(db.clone(), engine_config);

    // Process signals - should NOT create grounded signal
    engine.process_signals().await.unwrap();

    // Verify that no grounded signal was created
    use crate::repositories::GroundedSignalRepository;
    let grounded_repo = GroundedSignalRepository::new(&*db);

    let grounded_signals = grounded_repo
        .list(crate::repositories::ListGroundedSignalsQuery {
            tenant_id,
            status: None,
            min_score: None,
            limit: None,
            offset: None,
        })
        .await
        .unwrap();

    assert!(
        grounded_signals.data.is_empty(),
        "Expected no grounded signals below threshold"
    );
}
