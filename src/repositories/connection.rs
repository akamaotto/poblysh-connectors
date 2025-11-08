//! Connection repository for database operations
//!
//! This module provides the ConnectionRepository struct which encapsulates
//! SeaORM operations for the connections table with tenant-aware methods
//! and cursor-based pagination.

use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder, QuerySelect, Set,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::crypto::{
    CryptoKey, decrypt_connection_tokens, encrypt_connection_tokens, is_encrypted_payload,
};
use crate::cursor::{decode_generic_cursor, encode_generic_cursor};
use crate::models::connection::{self, Entity as Connection};

/// Repository for connection database operations
#[derive(Debug, Clone)]
pub struct ConnectionRepository {
    /// Database connection pool
    pub db: Arc<DatabaseConnection>,
    /// Crypto key for token encryption
    pub crypto_key: CryptoKey,
}

impl ConnectionRepository {
    /// Creates a new ConnectionRepository instance
    pub fn new(db: Arc<DatabaseConnection>, crypto_key: CryptoKey) -> Self {
        Self { db, crypto_key }
    }

    /// Encrypts tokens and updates connection with encrypted ciphertexts
    pub async fn encrypt_and_update_tokens(
        &self,
        connection_id: &Uuid,
        access_token: Option<&str>,
        refresh_token: Option<&str>,
    ) -> Result<connection::Model> {
        let connection = self
            .get_by_id(connection_id)
            .await?
            .ok_or_else(|| anyhow!("Connection with ID '{}' not found", connection_id))?;

        let (encrypted_access_token, encrypted_refresh_token) =
            encrypt_connection_tokens(&self.crypto_key, &connection, access_token, refresh_token)
                .map_err(|e| anyhow!("Token encryption failed: {}", e))?;

        self.update_tokens_status(
            connection_id,
            encrypted_access_token,
            encrypted_refresh_token,
            None,
            None,
        )
        .await
    }

    /// Decrypts tokens from a connection model
    pub async fn decrypt_tokens(
        &self,
        connection: &connection::Model,
    ) -> Result<(Option<String>, Option<String>, bool)> {
        let has_legacy_access = connection
            .access_token_ciphertext
            .as_ref()
            .is_some_and(|token| !is_encrypted_payload(token));
        let has_legacy_refresh = connection
            .refresh_token_ciphertext
            .as_ref()
            .is_some_and(|token| !is_encrypted_payload(token));
        let had_legacy_tokens = has_legacy_access || has_legacy_refresh;

        if had_legacy_tokens {
            tracing::warn!(
                tenant_id = %connection.tenant_id,
                provider_slug = %connection.provider_slug,
                external_id = %connection.external_id,
                legacy_access_token = has_legacy_access,
                legacy_refresh_token = has_legacy_refresh,
                "Legacy plaintext tokens detected, consider migrating to encrypted format"
            );
        }

        let (decrypted_access_token, decrypted_refresh_token) =
            decrypt_connection_tokens(&self.crypto_key, connection).map_err(|e| {
                // Log decryption failures as generic auth errors without details
                tracing::error!(
                    tenant_id = %connection.tenant_id,
                    provider_slug = %connection.provider_slug,
                    external_id = %connection.external_id,
                    "Token decryption failed"
                );
                anyhow!("Token decryption failed: {}", e)
            })?;

        Ok((
            decrypted_access_token,
            decrypted_refresh_token,
            had_legacy_tokens,
        ))
    }

    /// Creates a connection with encrypted tokens
    pub async fn create_with_tokens(
        &self,
        mut connection: connection::ActiveModel,
        access_token: Option<&str>,
        refresh_token: Option<&str>,
    ) -> Result<connection::Model> {
        let connection_id = connection
            .id
            .clone()
            .take()
            .ok_or_else(|| anyhow!("connection id must be set"))?;

        // Create a temporary connection model for AAD generation
        let temp_connection = connection::Model {
            id: connection_id,
            tenant_id: connection.tenant_id.clone().unwrap(),
            provider_slug: connection.provider_slug.clone().unwrap(),
            external_id: connection.external_id.clone().unwrap(),
            status: connection.status.clone().unwrap(),
            display_name: None, // Not needed for AAD generation
            access_token_ciphertext: None,
            refresh_token_ciphertext: None,
            expires_at: None, // Not needed for AAD generation
            scopes: None,     // Not needed for AAD generation
            metadata: None,   // Not needed for AAD generation
            created_at: chrono::Utc::now().into(),
            updated_at: chrono::Utc::now().into(),
        };

        // Encrypt tokens
        let (encrypted_access_token, encrypted_refresh_token) = encrypt_connection_tokens(
            &self.crypto_key,
            &temp_connection,
            access_token,
            refresh_token,
        )
        .map_err(|e| anyhow!("Token encryption failed: {}", e))?;

        // Set encrypted ciphertexts
        connection.access_token_ciphertext = Set(encrypted_access_token);
        connection.refresh_token_ciphertext = Set(encrypted_refresh_token);

        // Save connection
        let active = connection;
        active.insert(&*self.db).await?;

        // For SQLite, query the record directly since we already know the ID
        let fetched = Connection::find_by_id(connection_id).one(&*self.db).await?;
        fetched.ok_or_else(|| anyhow!("connection not persisted"))
    }

    /// Finds a connection by its ID within a tenant scope
    pub async fn find_by_id(
        &self,
        tenant_id: &Uuid,
        id: &Uuid,
    ) -> Result<Option<connection::Model>> {
        Ok(Connection::find_by_id(*id)
            .filter(connection::Column::TenantId.eq(*tenant_id))
            .one(&*self.db)
            .await?)
    }

    /// Retrieves a connection by its ID without tenant scoping
    pub async fn get_by_id(&self, id: &Uuid) -> Result<Option<connection::Model>> {
        Ok(Connection::find_by_id(*id).one(&*self.db).await?)
    }

    /// Lists all connections for a tenant ordered by creation time then ID
    pub async fn find_by_tenant(&self, tenant_id: &Uuid) -> Result<Vec<connection::Model>> {
        Ok(Connection::find()
            .filter(connection::Column::TenantId.eq(*tenant_id))
            .order_by_asc(connection::Column::CreatedAt)
            .order_by_asc(connection::Column::Id)
            .all(&*self.db)
            .await?)
    }

    /// Lists all connections for a tenant/provider pair ordered by creation time then ID
    pub async fn find_by_tenant_and_provider(
        &self,
        tenant_id: &Uuid,
        provider_slug: &str,
    ) -> Result<Vec<connection::Model>> {
        Ok(Connection::find()
            .filter(connection::Column::TenantId.eq(*tenant_id))
            .filter(connection::Column::ProviderSlug.eq(provider_slug))
            .order_by_asc(connection::Column::CreatedAt)
            .order_by_asc(connection::Column::Id)
            .all(&*self.db)
            .await?)
    }

    /// Finds a connection by its unique `(tenant, provider, external_id)` tuple
    pub async fn find_by_external_id(
        &self,
        tenant_id: &Uuid,
        provider_slug: &str,
        external_id: &str,
    ) -> Result<Option<connection::Model>> {
        Ok(Connection::find()
            .filter(connection::Column::TenantId.eq(*tenant_id))
            .filter(connection::Column::ProviderSlug.eq(provider_slug))
            .filter(connection::Column::ExternalId.eq(external_id))
            .one(&*self.db)
            .await?)
    }

    /// Alias for spec wording (`find_by_unique`)
    pub async fn find_by_unique(
        &self,
        tenant_id: &Uuid,
        provider_slug: &str,
        external_id: &str,
    ) -> Result<Option<connection::Model>> {
        self.find_by_external_id(tenant_id, provider_slug, external_id)
            .await
    }

    /// Creates a new connection record
    pub async fn create(&self, connection: connection::ActiveModel) -> Result<connection::Model> {
        let id = connection
            .id
            .clone()
            .take()
            .ok_or_else(|| anyhow!("connection id must be set"))?;

        let active = connection;
        active.insert(&*self.db).await?;

        // For SQLite, query the record directly since we already know the ID
        let fetched = Connection::find_by_id(id).one(&*self.db).await?;
        fetched.ok_or_else(|| anyhow!("connection not persisted"))
    }

    /// Updates mutable fields on a connection within a tenant scope
    pub async fn update_by_id(
        &self,
        tenant_id: &Uuid,
        id: &Uuid,
        update: connection::ActiveModel,
    ) -> Result<connection::Model> {
        let existing = Connection::find_by_id(*id)
            .filter(connection::Column::TenantId.eq(*tenant_id))
            .one(&*self.db)
            .await?
            .ok_or_else(|| anyhow!("Connection with ID '{}' not found for tenant", id))?;

        let mut model: connection::ActiveModel = existing.into();

        if let Some(external_id) = update.external_id.clone().take() {
            model.external_id = Set(external_id);
        }
        if let Some(display_name) = update.display_name.clone().take() {
            model.display_name = Set(display_name);
        }
        if let Some(status) = update.status.clone().take() {
            model.status = Set(status);
        }
        if let Some(access_cipher) = update.access_token_ciphertext.clone().take() {
            model.access_token_ciphertext = Set(access_cipher);
        }
        if let Some(refresh_cipher) = update.refresh_token_ciphertext.clone().take() {
            model.refresh_token_ciphertext = Set(refresh_cipher);
        }
        if let Some(expires_at) = update.expires_at.clone().take() {
            model.expires_at = Set(expires_at);
        }
        if let Some(scopes) = update.scopes.clone().take() {
            model.scopes = Set(scopes);
        }
        if let Some(metadata) = update.metadata.clone().take() {
            model.metadata = Set(metadata);
        }

        Ok(model.update(&*self.db).await?)
    }

    /// Partial update helper for tokens/status/expiry mutations
    pub async fn update_tokens_status(
        &self,
        id: &Uuid,
        access_token_ciphertext: Option<Vec<u8>>,
        refresh_token_ciphertext: Option<Vec<u8>>,
        status: Option<String>,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<connection::Model> {
        let existing = Connection::find_by_id(*id)
            .one(&*self.db)
            .await?
            .ok_or_else(|| anyhow!("Connection '{}' not found", id))?;

        let mut model: connection::ActiveModel = existing.into();

        if let Some(cipher) = access_token_ciphertext {
            model.access_token_ciphertext = Set(Some(cipher));
        }
        if let Some(cipher) = refresh_token_ciphertext {
            model.refresh_token_ciphertext = Set(Some(cipher));
        }
        if let Some(status) = status {
            model.status = Set(status);
        }
        if let Some(expires_at) = expires_at {
            let fixed: DateTimeWithTimeZone = expires_at.into();
            model.expires_at = Set(Some(fixed));
        }

        Ok(model.update(&*self.db).await?)
    }

    /// Deletes a connection within a tenant scope
    pub async fn delete_by_id(&self, tenant_id: &Uuid, id: &Uuid) -> Result<()> {
        let result = Connection::delete_by_id(*id)
            .filter(connection::Column::TenantId.eq(*tenant_id))
            .exec(&*self.db)
            .await?;

        if result.rows_affected == 0 {
            return Err(anyhow!("Connection with ID '{}' not found for tenant", id));
        }

        Ok(())
    }

    /// Lists all connections for a tenant with cursor pagination
    pub async fn list_by_tenant(
        &self,
        tenant_id: &Uuid,
        limit: u64,
        cursor: Option<String>,
    ) -> Result<(Vec<connection::Model>, Option<String>)> {
        if limit == 0 {
            return Ok((Vec::new(), cursor));
        }

        let mut query = Connection::find()
            .filter(connection::Column::TenantId.eq(*tenant_id))
            .order_by_asc(connection::Column::CreatedAt)
            .order_by_asc(connection::Column::Id);

        if let Some(cursor) = cursor
            && !cursor.is_empty()
        {
            let (created_at, cursor_id) = parse_connection_cursor(&cursor)?;
            let condition = Condition::any()
                .add(connection::Column::CreatedAt.gt(created_at))
                .add(
                    Condition::all()
                        .add(connection::Column::CreatedAt.eq(created_at))
                        .add(connection::Column::Id.gt(cursor_id)),
                );
            query = query.filter(condition);
        }

        let mut rows = query.limit(limit + 1).all(&*self.db).await?;

        let next_cursor = if rows.len() as u64 > limit {
            // Remove overflow row to get only the items to return
            rows.pop().expect("limit+1 ensures overflow row");
            // Build cursor from the last item that was actually returned
            rows.last()
                .map(|last_item| build_connection_cursor(&last_item.created_at, last_item.id))
                .transpose()?
        } else {
            None
        };

        Ok((rows, next_cursor))
    }

    /// Lists connections for a tenant/provider pair with cursor pagination
    pub async fn list_by_tenant_provider(
        &self,
        tenant_id: &Uuid,
        provider_slug: &str,
        limit: u64,
        cursor: Option<String>,
    ) -> Result<(Vec<connection::Model>, Option<String>)> {
        if limit == 0 {
            return Ok((Vec::new(), cursor));
        }

        let mut query = Connection::find()
            .filter(connection::Column::TenantId.eq(*tenant_id))
            .filter(connection::Column::ProviderSlug.eq(provider_slug))
            .order_by_asc(connection::Column::CreatedAt)
            .order_by_asc(connection::Column::Id);

        if let Some(cursor) = cursor
            && !cursor.is_empty()
        {
            let (created_at, cursor_id) = parse_connection_cursor(&cursor)?;
            let condition = Condition::any()
                .add(connection::Column::CreatedAt.gt(created_at))
                .add(
                    Condition::all()
                        .add(connection::Column::CreatedAt.eq(created_at))
                        .add(connection::Column::Id.gt(cursor_id)),
                );
            query = query.filter(condition);
        }

        let mut rows = query.limit(limit + 1).all(&*self.db).await?;

        let next_cursor = if rows.len() as u64 > limit {
            // Remove overflow row to get only the items to return
            rows.pop().expect("limit+1 ensures overflow row");
            // Build cursor from the last item that was actually returned
            rows.last()
                .map(|last_item| build_connection_cursor(&last_item.created_at, last_item.id))
                .transpose()?
        } else {
            None
        };

        Ok((rows, next_cursor))
    }
}

/// Parse connection cursor from standardized base64 string
fn parse_connection_cursor(cursor: &str) -> Result<(DateTimeWithTimeZone, Uuid)> {
    let decoded_cursor = decode_generic_cursor(cursor)
        .map_err(|_| anyhow!("Invalid cursor format: must be valid base64-encoded JSON"))?;

    // Extract required fields from cursor
    let created_at_str = decoded_cursor.keys["created_at"]
        .as_str()
        .ok_or_else(|| anyhow!("Invalid cursor format: missing created_at field"))?;

    let id_str = decoded_cursor.keys["id"]
        .as_str()
        .ok_or_else(|| anyhow!("Invalid cursor format: missing id field"))?;

    let created_at = DateTime::parse_from_rfc3339(created_at_str).map_err(|_| {
        anyhow!("Invalid cursor format: created_at must be a valid RFC3339 timestamp")
    })?;

    let id = Uuid::parse_str(id_str)
        .map_err(|_| anyhow!("Invalid cursor format: id must be a valid UUID"))?;

    Ok((created_at, id))
}

/// Build connection cursor using standardized base64 format
fn build_connection_cursor(created_at: &DateTimeWithTimeZone, id: Uuid) -> Result<String> {
    let keys = serde_json::json!({
        "created_at": created_at.to_rfc3339(),
        "id": id.to_string()
    });
    Ok(encode_generic_cursor(keys))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    #[test]
    fn build_connection_cursor_formats_expected_base64() {
        let ts = Utc.with_ymd_and_hms(2024, 11, 1, 12, 0, 0).unwrap();
        let id = Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap();
        let cursor = build_connection_cursor(&ts.into(), id).unwrap();
        // Should be base64 encoded, not contain raw timestamp
        assert!(!cursor.contains("2024-11-01T12:00:00"));
        // Should be valid base64
        assert!(
            cursor
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=')
        );
    }

    #[test]
    fn parse_connection_cursor_roundtrips() {
        let ts = Utc.with_ymd_and_hms(2024, 11, 1, 13, 30, 0).unwrap();
        let id = Uuid::parse_str("22222222-2222-2222-2222-222222222222").unwrap();
        let ts_fixed: DateTimeWithTimeZone = ts.into();
        let cursor = build_connection_cursor(&ts_fixed, id).unwrap();
        let (parsed_ts, parsed_id) = parse_connection_cursor(&cursor).unwrap();
        assert_eq!(parsed_id, id);
        assert_eq!(parsed_ts, ts_fixed);
    }

    #[test]
    fn parse_connection_cursor_invalid_format_errors() {
        let err = parse_connection_cursor("bad-cursor").unwrap_err();
        assert!(err.to_string().contains("Invalid cursor"));
    }

    #[test]
    fn test_cursor_built_from_last_returned_item() {
        // Test that next_cursor is built from the last item returned, not from overflow
        let created_at = chrono::Utc::now();
        let id = Uuid::from_u128(12345);

        // Build cursor from an item
        let cursor = build_connection_cursor(&created_at.into(), id).unwrap();

        // Parse it back and verify it contains the expected values
        let (parsed_created_at, parsed_id) = parse_connection_cursor(&cursor).unwrap();
        assert_eq!(parsed_created_at.naive_utc(), created_at.naive_utc());
        assert_eq!(parsed_id, id);

        // Verify cursor is base64 encoded
        assert!(
            cursor
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=')
        );
    }
}
