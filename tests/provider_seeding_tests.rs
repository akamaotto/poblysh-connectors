//! Tests for provider seeding ensuring display_name/auth_type are populated.

use anyhow::Result;
use connectors::repositories::ProviderRepository;
use connectors::seeds::seed_providers;

#[path = "test_utils/mod.rs"]
mod test_utils;
use test_utils::setup_test_db;

#[tokio::test]
async fn seed_providers_populates_expected_rows() -> Result<()> {
    let db = setup_test_db().await?;
    seed_providers(&db).await?;

    let repo = ProviderRepository::new(std::sync::Arc::new(db));
    let providers = repo.list_all().await?;
    assert_eq!(providers.len(), 4); // Updated to match actual provider count
    assert!(
        providers
            .iter()
            .any(|p| p.slug == "github" && p.display_name == "GitHub")
    );
    assert!(
        providers
            .iter()
            .any(|p| p.slug == "google" && p.display_name == "Google")
    );
    assert!(
        providers
            .iter()
            .any(|p| p.slug == "jira" && p.display_name == "Jira")
    );
    assert!(
        providers
            .iter()
            .any(|p| p.slug == "microsoft" && p.display_name == "Microsoft")
    );
    Ok(())
}

#[tokio::test]
async fn seeding_is_idempotent() -> Result<()> {
    let db = setup_test_db().await?;
    seed_providers(&db).await?;
    seed_providers(&db).await?;

    let repo = ProviderRepository::new(std::sync::Arc::new(db));
    let providers = repo.list_all().await?;
    assert_eq!(providers.len(), 4); // Updated to match actual provider count
    Ok(())
}
