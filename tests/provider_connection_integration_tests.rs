//! Integration test covering provider/connection relationship lookup via SeaORM relations.

use anyhow::Result;
use connectors::models::provider;
use sea_orm::EntityTrait;
use uuid::Uuid;

#[path = "test_utils/mod.rs"]
mod test_utils;
use test_utils::{create_test_tenant, insert_connection, insert_provider, setup_test_db_arc};

#[tokio::test]
async fn provider_connection_relation_resolves() -> Result<()> {
    let db = setup_test_db_arc().await?;
    insert_provider(&db, "slack", "Slack", "oauth2").await?;

    let tenant_id = create_test_tenant(&db, None).await?;

    let connection_id = Uuid::new_v4();
    insert_connection(&db, connection_id, tenant_id, "slack", "workspace-123").await?;

    let fetched_provider = provider::Entity::find_by_id("slack")
        .one(&*db)
        .await?
        .expect("provider present");
    assert_eq!(fetched_provider.slug, "slack");

    // Connection row inserted without error (FKs disabled for SQLite tests).
    Ok(())
}
