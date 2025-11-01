//! Tests ensuring tenant isolation via direct SQL fixtures.

use anyhow::Result;
use uuid::Uuid;

#[path = "test_utils/mod.rs"]
mod test_utils;
use test_utils::{create_test_tenant, insert_connection, insert_provider, setup_test_db_arc};

#[tokio::test]
async fn unique_constraint_scoped_to_tenant() -> Result<()> {
    let db = setup_test_db_arc().await?;
    insert_provider(&db, "github", "GitHub", "oauth2").await?;

    let tenant_a = create_test_tenant(&db, None).await?;
    let tenant_b = create_test_tenant(&db, None).await?;

    insert_connection(&db, Uuid::new_v4(), tenant_a, "github", "ext-shared").await?;
    insert_connection(&db, Uuid::new_v4(), tenant_b, "github", "ext-shared").await?;

    let duplicate = insert_connection(&db, Uuid::new_v4(), tenant_a, "github", "ext-shared").await;

    assert!(duplicate.unwrap_err().to_string().contains("UNIQUE"));
    Ok(())
}
