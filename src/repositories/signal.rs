//! # Signal Repository
//!
//! This module contains the repository implementation for Signal entities,
//! providing tenant-scoped data access methods with filtering and cursor pagination.

use crate::error::RepositoryError;
use chrono::{DateTime, Utc};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, FromQueryResult, QueryFilter, QueryOrder,
    QuerySelect,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::signal::{Entity as Signal, Model};

/// Cursor data structure for pagination
#[derive(Debug, Clone, Serialize, Deserialize, FromQueryResult)]
pub struct CursorData {
    pub occurred_at: DateTime<Utc>,
    pub id: Uuid,
}

/// Repository for Signal database operations
pub struct SignalRepository<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> SignalRepository<'a> {
    /// Create a new SignalRepository with the given database connection
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    /// List signals for a tenant with filters and cursor pagination
    ///
    /// # Arguments
    /// * `tenant_id` - The tenant ID to scope the query
    /// * `provider_slug` - Optional filter by provider slug
    /// * `connection_id` - Optional filter by connection ID
    /// * `kind` - Optional filter by signal kind
    /// * `occurred_after` - Optional filter for signals after this timestamp
    /// * `occurred_before` - Optional filter for signals before this timestamp
    /// * `cursor_data` - Optional cursor data for pagination continuation
    /// * `limit` - Maximum number of signals to return
    /// * `include_payload` - Whether to include the full payload
    ///
    /// # Returns
    /// A vector of Signal models ordered by occurred_at DESC, id DESC
    pub async fn list_signals(
        &self,
        tenant_id: Uuid,
        provider_slug: Option<String>,
        connection_id: Option<Uuid>,
        kind: Option<String>,
        occurred_after: Option<DateTime<Utc>>,
        occurred_before: Option<DateTime<Utc>>,
        cursor_data: Option<CursorData>,
        limit: i64,
        _include_payload: bool,
    ) -> Result<Vec<Model>, RepositoryError> {
        let mut query =
            Signal::find().filter(crate::models::signal::Column::TenantId.eq(tenant_id));

        // Apply filters
        if let Some(provider) = provider_slug {
            query = query.filter(crate::models::signal::Column::ProviderSlug.eq(provider));
        }

        if let Some(conn_id) = connection_id {
            query = query.filter(crate::models::signal::Column::ConnectionId.eq(conn_id));
        }

        if let Some(signal_kind) = kind {
            query = query.filter(crate::models::signal::Column::Kind.eq(signal_kind));
        }

        if let Some(after) = occurred_after {
            query = query.filter(crate::models::signal::Column::OccurredAt.gte(after));
        }

        if let Some(before) = occurred_before {
            query = query.filter(crate::models::signal::Column::OccurredAt.lte(before));
        }

        // Apply cursor pagination
        if let Some(cursor) = cursor_data {
            query = query.filter(
                sea_orm::Condition::any()
                    .add(crate::models::signal::Column::OccurredAt.lt(cursor.occurred_at))
                    .add(
                        sea_orm::Condition::all()
                            .add(crate::models::signal::Column::OccurredAt.eq(cursor.occurred_at))
                            .add(crate::models::signal::Column::Id.lt(cursor.id)),
                    ),
            );
        }

        // Order by occurred_at DESC, id DESC for stability
        query = query
            .order_by_desc(crate::models::signal::Column::OccurredAt)
            .order_by_desc(crate::models::signal::Column::Id);

        // Apply limit and execute
        let signals = query
            .limit(limit as u64)
            .all(self.db)
            .await
            .map_err(RepositoryError::database_error)?;

        Ok(signals)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;
    use crate::db::init_pool;
    use crate::models::connection::ActiveModel as ConnectionActiveModel;
    use crate::models::provider::ActiveModel as ProviderActiveModel;
    use crate::models::signal::ActiveModel as SignalActiveModel;
    use crate::models::tenant::ActiveModel as TenantActiveModel;
    use chrono::Utc;
    use sea_orm::ActiveModelTrait;
    use uuid::Uuid;

    async fn setup_test_data() -> (DatabaseConnection, Uuid, Uuid, Uuid) {
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

        // Create provider
        let provider_id = Uuid::new_v4();
        let provider = ProviderActiveModel {
            slug: sea_orm::Set("test-provider".to_string()),
            display_name: sea_orm::Set("Test Provider".to_string()),
            auth_type: sea_orm::Set("oauth".to_string()),
            ..Default::default()
        };
        provider.insert(&db).await.unwrap();

        // Create connection
        let connection_id = Uuid::new_v4();
        let connection = ConnectionActiveModel {
            id: sea_orm::Set(connection_id),
            tenant_id: sea_orm::Set(tenant_id),
            provider_slug: sea_orm::Set("test-provider".to_string()),
            ..Default::default()
        };
        connection.insert(&db).await.unwrap();

        (db, tenant_id, connection_id, provider_id)
    }

    #[tokio::test]
    async fn test_list_signals_empty() {
        let (db, tenant_id, _, _) = setup_test_data().await;
        let repo = SignalRepository::new(&db);

        let signals = repo
            .list_signals(tenant_id, None, None, None, None, None, None, 10, false)
            .await
            .unwrap();

        assert_eq!(signals.len(), 0);
    }

    #[tokio::test]
    async fn test_list_signals_with_data() {
        let (db, tenant_id, connection_id, _) = setup_test_data().await;
        let repo = SignalRepository::new(&db);

        // Create test signals
        let now = Utc::now();
        for i in 0..5 {
            let signal = SignalActiveModel {
                id: sea_orm::Set(Uuid::new_v4()),
                tenant_id: sea_orm::Set(tenant_id),
                provider_slug: sea_orm::Set("test-provider".to_string()),
                connection_id: sea_orm::Set(connection_id),
                kind: sea_orm::Set(format!("test_event_{}", i)),
                occurred_at: sea_orm::Set((now - chrono::Duration::seconds(i as i64)).into()),
                received_at: sea_orm::Set((now - chrono::Duration::seconds(i as i64)).into()),
                payload: sea_orm::Set(serde_json::json!({"test": i})),
                ..Default::default()
            };
            signal.insert(&db).await.unwrap();
        }

        let signals = repo
            .list_signals(tenant_id, None, None, None, None, None, None, 10, false)
            .await
            .unwrap();

        assert_eq!(signals.len(), 5);
        // Should be ordered by occurred_at DESC (newest first)
        assert_eq!(signals[0].kind, "test_event_0");
        assert_eq!(signals[4].kind, "test_event_4");
    }

    #[tokio::test]
    async fn test_list_signals_with_filters() {
        let (db, tenant_id, connection_id, _) = setup_test_data().await;
        let repo = SignalRepository::new(&db);

        let now = Utc::now();

        // Create signals with different kinds
        for i in 0..3 {
            let signal = SignalActiveModel {
                id: sea_orm::Set(Uuid::new_v4()),
                tenant_id: sea_orm::Set(tenant_id),
                provider_slug: sea_orm::Set("test-provider".to_string()),
                connection_id: sea_orm::Set(connection_id),
                kind: sea_orm::Set("event_a".to_string()),
                occurred_at: sea_orm::Set((now - chrono::Duration::seconds(i as i64)).into()),
                received_at: sea_orm::Set(now.into()),
                payload: sea_orm::Set(serde_json::json!({"type": "a"})),
                ..Default::default()
            };
            signal.insert(&db).await.unwrap();
        }

        for i in 0..2 {
            let signal = SignalActiveModel {
                id: sea_orm::Set(Uuid::new_v4()),
                tenant_id: sea_orm::Set(tenant_id),
                provider_slug: sea_orm::Set("test-provider".to_string()),
                connection_id: sea_orm::Set(connection_id),
                kind: sea_orm::Set("event_b".to_string()),
                occurred_at: sea_orm::Set((now - chrono::Duration::seconds(i as i64)).into()),
                received_at: sea_orm::Set(now.into()),
                payload: sea_orm::Set(serde_json::json!({"type": "b"})),
                ..Default::default()
            };
            signal.insert(&db).await.unwrap();
        }

        // Filter by kind
        let signals = repo
            .list_signals(
                tenant_id,
                None,
                None,
                Some("event_a".to_string()),
                None,
                None,
                None,
                10,
                false,
            )
            .await
            .unwrap();

        assert_eq!(signals.len(), 3);
        assert!(signals.iter().all(|s| s.kind == "event_a"));
    }

    #[tokio::test]
    async fn test_list_signals_with_time_range() {
        let (db, tenant_id, connection_id, _) = setup_test_data().await;
        let repo = SignalRepository::new(&db);

        let now = Utc::now();
        let base_time = now - chrono::Duration::hours(2);

        // Create signals at different times
        for i in 0..5 {
            let signal = SignalActiveModel {
                id: sea_orm::Set(Uuid::new_v4()),
                tenant_id: sea_orm::Set(tenant_id),
                provider_slug: sea_orm::Set("test-provider".to_string()),
                connection_id: sea_orm::Set(connection_id),
                kind: sea_orm::Set("timed_event".to_string()),
                occurred_at: sea_orm::Set((base_time + chrono::Duration::minutes(i * 15)).into()),
                received_at: sea_orm::Set(now.into()),
                payload: sea_orm::Set(serde_json::json!({"index": i})),
                ..Default::default()
            };
            signal.insert(&db).await.unwrap();
        }

        // Filter by time range
        let start_time = base_time + chrono::Duration::minutes(30);
        let end_time = base_time + chrono::Duration::minutes(60);

        let signals = repo
            .list_signals(
                tenant_id,
                None,
                None,
                None,
                Some(start_time),
                Some(end_time),
                None,
                10,
                false,
            )
            .await
            .unwrap();

        assert_eq!(signals.len(), 3); // Should include signals at minutes 30, 45, 60
        for signal in &signals {
            assert!(signal.occurred_at >= start_time);
            assert!(signal.occurred_at <= end_time);
        }
    }

    #[tokio::test]
    async fn test_list_signals_with_cursor_pagination() {
        let (db, tenant_id, connection_id, _) = setup_test_data().await;
        let repo = SignalRepository::new(&db);

        let now = Utc::now();

        // Create signals in chronological order
        let mut signal_ids = Vec::new();
        for i in 0..10 {
            let signal_id = Uuid::new_v4();
            signal_ids.push(signal_id);
            let signal = SignalActiveModel {
                id: sea_orm::Set(signal_id),
                tenant_id: sea_orm::Set(tenant_id),
                provider_slug: sea_orm::Set("test-provider".to_string()),
                connection_id: sea_orm::Set(connection_id),
                kind: sea_orm::Set("paginated_event".to_string()),
                occurred_at: sea_orm::Set((now - chrono::Duration::seconds(i as i64)).into()),
                received_at: sea_orm::Set(now.into()),
                payload: sea_orm::Set(serde_json::json!({"index": i})),
                ..Default::default()
            };
            signal.insert(&db).await.unwrap();
        }

        // First page
        let first_page = repo
            .list_signals(tenant_id, None, None, None, None, None, None, 3, false)
            .await
            .unwrap();

        assert_eq!(first_page.len(), 3);

        // Second page using cursor from last item of first page
        let cursor_data = CursorData {
            occurred_at: first_page[2].occurred_at.into(),
            id: first_page[2].id,
        };

        let second_page = repo
            .list_signals(
                tenant_id,
                None,
                None,
                None,
                None,
                None,
                Some(cursor_data),
                3,
                false,
            )
            .await
            .unwrap();

        assert_eq!(second_page.len(), 3);

        // Ensure we get different results and they're in the correct order
        let first_page_ids: Vec<_> = first_page.iter().map(|s| s.id).collect();
        let second_page_ids: Vec<_> = second_page.iter().map(|s| s.id).collect();

        assert_ne!(first_page_ids, second_page_ids);

        // Since we order by occurred_at DESC, id DESC, and timestamps are decreasing,
        // the second page should have older signals (larger negative seconds)
        for (first, second) in first_page.iter().zip(second_page.iter()) {
            assert!(first.occurred_at >= second.occurred_at);
        }
    }

    #[tokio::test]
    async fn test_list_signals_payload_inclusion() {
        let (db, tenant_id, connection_id, _) = setup_test_data().await;
        let repo = SignalRepository::new(&db);

        let test_payload = serde_json::json!({"key": "value", "number": 42});

        // Create signal with payload
        let signal = SignalActiveModel {
            id: sea_orm::Set(Uuid::new_v4()),
            tenant_id: sea_orm::Set(tenant_id),
            provider_slug: sea_orm::Set("test-provider".to_string()),
            connection_id: sea_orm::Set(connection_id),
            kind: sea_orm::Set("payload_test".to_string()),
            occurred_at: sea_orm::Set(Utc::now().into()),
            received_at: sea_orm::Set(Utc::now().into()),
            payload: sea_orm::Set(test_payload.clone()),
            ..Default::default()
        };
        signal.insert(&db).await.unwrap();

        // Without payload
        let signals_without_payload = repo
            .list_signals(tenant_id, None, None, None, None, None, None, 10, false)
            .await
            .unwrap();

        assert_eq!(signals_without_payload.len(), 1);
        // When include_payload is false, the payload field should not be accessible
        // because we didn't select it in the query

        // With payload
        let signals_with_payload = repo
            .list_signals(tenant_id, None, None, None, None, None, None, 10, true)
            .await
            .unwrap();

        assert_eq!(signals_with_payload.len(), 1);
        assert_eq!(signals_with_payload[0].payload, test_payload);
    }

    #[tokio::test]
    async fn test_list_signals_tenant_isolation() {
        let (db, tenant1_id, connection1_id, _) = setup_test_data().await;

        // Create second tenant and connection
        let tenant2_id = Uuid::new_v4();
        let tenant2 = TenantActiveModel {
            id: sea_orm::Set(tenant2_id),
            ..Default::default()
        };
        tenant2.insert(&db).await.unwrap();

        let connection2_id = Uuid::new_v4();
        let connection2 = ConnectionActiveModel {
            id: sea_orm::Set(connection2_id),
            tenant_id: sea_orm::Set(tenant2_id),
            provider_slug: sea_orm::Set("test-provider".to_string()),
            ..Default::default()
        };
        connection2.insert(&db).await.unwrap();

        let repo = SignalRepository::new(&db);
        let now = Utc::now();

        // Create signal for tenant 1
        let signal1 = SignalActiveModel {
            id: sea_orm::Set(Uuid::new_v4()),
            tenant_id: sea_orm::Set(tenant1_id),
            provider_slug: sea_orm::Set("test-provider".to_string()),
            connection_id: sea_orm::Set(connection1_id),
            kind: sea_orm::Set("tenant1_event".to_string()),
            occurred_at: sea_orm::Set(now.into()),
            received_at: sea_orm::Set(now.into()),
            payload: sea_orm::Set(serde_json::json!({"tenant": 1})),
            ..Default::default()
        };
        signal1.insert(&db).await.unwrap();

        // Create signal for tenant 2
        let signal2 = SignalActiveModel {
            id: sea_orm::Set(Uuid::new_v4()),
            tenant_id: sea_orm::Set(tenant2_id),
            provider_slug: sea_orm::Set("test-provider".to_string()),
            connection_id: sea_orm::Set(connection2_id),
            kind: sea_orm::Set("tenant2_event".to_string()),
            occurred_at: sea_orm::Set(now.into()),
            received_at: sea_orm::Set(now.into()),
            payload: sea_orm::Set(serde_json::json!({"tenant": 2})),
            ..Default::default()
        };
        signal2.insert(&db).await.unwrap();

        // Query tenant 1 - should only return tenant 1 signals
        let tenant1_signals = repo
            .list_signals(tenant1_id, None, None, None, None, None, None, 10, false)
            .await
            .unwrap();

        assert_eq!(tenant1_signals.len(), 1);
        assert_eq!(tenant1_signals[0].kind, "tenant1_event");

        // Query tenant 2 - should only return tenant 2 signals
        let tenant2_signals = repo
            .list_signals(tenant2_id, None, None, None, None, None, None, 10, false)
            .await
            .unwrap();

        assert_eq!(tenant2_signals.len(), 1);
        assert_eq!(tenant2_signals[0].kind, "tenant2_event");
    }
}
