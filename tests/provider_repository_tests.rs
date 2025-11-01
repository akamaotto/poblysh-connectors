//! Integration tests for ProviderRepository aligned with the new provider schema.

use anyhow::Result;
use connectors::models::provider;
use connectors::repositories::ProviderRepository;
use sea_orm::Set;

#[path = "test_utils/mod.rs"]
mod test_utils;
use test_utils::setup_test_db_arc;

#[tokio::test]
async fn upsert_creates_and_updates_provider() -> Result<()> {
    let db = setup_test_db_arc().await?;
    let repo = ProviderRepository::new(db.clone());

    // Upsert a provider
    let created = repo
        .upsert("github", "GitHub", "oauth2")
        .await
        .expect("upsert succeeds");
    assert_eq!(created.slug, "github");
    assert_eq!(created.display_name, "GitHub");
    assert_eq!(created.auth_type, "oauth2");

    // Update auth type via upsert
    let updated = repo
        .upsert("github", "GitHub", "webhook-only")
        .await
        .expect("second upsert succeeds");
    assert_eq!(updated.auth_type, "webhook-only");

    // List
    let all = repo.list_all().await?;
    assert_eq!(all.len(), 1);
    Ok(())
}

#[tokio::test]
async fn create_and_find_roundtrip() -> Result<()> {
    let db = setup_test_db_arc().await?;
    let repo = ProviderRepository::new(db.clone());

    let provider = provider::ActiveModel {
        slug: Set("test-provider".to_string()),
        display_name: Set("Test Provider".to_string()),
        auth_type: Set("oauth2".to_string()),
        ..Default::default()
    };

    let created = repo.create(provider).await?;
    assert_eq!(created.slug, "test-provider");
    assert_eq!(created.display_name, "Test Provider");

    let found = repo.get_by_slug("test-provider").await?;
    assert!(found.is_some());
    assert_eq!(found.unwrap().display_name, "Test Provider");
    Ok(())
}

#[tokio::test]
async fn delete_by_slug_removes_provider() -> Result<()> {
    let db = setup_test_db_arc().await?;
    let repo = ProviderRepository::new(db.clone());

    let provider = provider::ActiveModel {
        slug: Set("to-delete".to_string()),
        display_name: Set("Delete Me".to_string()),
        auth_type: Set("oauth2".to_string()),
        ..Default::default()
    };
    repo.create(provider).await?;

    repo.delete_by_slug("to-delete").await?;
    assert!(repo.get_by_slug("to-delete").await?.is_none());
    Ok(())
}
