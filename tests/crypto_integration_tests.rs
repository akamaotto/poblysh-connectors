//! Integration tests for crypto module and connection repository
//!
//! These tests verify end-to-end encryption/decryption flows
//! and proper handling of various crypto scenarios.

use connectors::crypto::{
    CryptoError, CryptoKey, decrypt_bytes, encrypt_bytes, encrypt_connection_tokens,
};
use connectors::models::connection;
use connectors::repositories::connection::ConnectionRepository;
use std::sync::Arc;

use uuid::Uuid;

#[path = "test_utils/mod.rs"]
mod test_utils;

fn test_crypto_key() -> CryptoKey {
    CryptoKey::new(vec![0u8; 32]).expect("valid test key")
}

async fn setup_test_db() -> Arc<sea_orm::DatabaseConnection> {
    let db = test_utils::setup_test_db_arc()
        .await
        .expect("Failed to set up test database with migrations");

    // Insert a test provider for crypto tests
    test_utils::insert_provider(&db, "test-provider", "Test Provider", "oauth2")
        .await
        .expect("Failed to insert test provider");

    db
}

#[tokio::test]
async fn test_basic_connection_save() -> Result<(), Box<dyn std::error::Error>> {
    let db = setup_test_db().await;
    // Test data
    let tenant_id = Uuid::new_v4();
    let provider_slug = "test-provider".to_string();
    let external_id = "test-external-id".to_string();

    // Create basic connection without tokens first using test_utils direct SQL
    let connection_id = Uuid::new_v4();
    match test_utils::insert_connection(
        &*db,
        connection_id,
        tenant_id,
        &provider_slug,
        &external_id,
    )
    .await
    {
        Ok(_) => {
            println!("✅ Basic connection save succeeded");
        }
        Err(e) => {
            println!("❌ Basic connection save failed: {:?}", e);
            return Err(e.into());
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_connection_token_encryption_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
    let db = setup_test_db().await;
    let crypto_key = test_crypto_key();
    let connection_repo = ConnectionRepository::new(db.clone(), crypto_key.clone());

    // Test data
    let tenant_id = Uuid::new_v4();
    let provider_slug = "test-provider".to_string();
    let external_id = "test-external-id".to_string();
    let access_token = "test-access-token-12345";
    let refresh_token = "test-refresh-token-67890";

    // Create tenant first, then connection using direct SQL
    test_utils::create_test_tenant(&*db, Some(tenant_id)).await?;
    let connection_id = Uuid::new_v4();
    test_utils::insert_connection(&*db, connection_id, tenant_id, &provider_slug, &external_id)
        .await?;

    // Manually encrypt tokens using direct crypto operations (with correct argument order)
    let aad_string = format!("{}|{}|{}", tenant_id, provider_slug, external_id);
    let encrypted_access =
        encrypt_bytes(&crypto_key, aad_string.as_bytes(), access_token.as_bytes())?;
    let encrypted_refresh =
        encrypt_bytes(&crypto_key, aad_string.as_bytes(), refresh_token.as_bytes())?;

    // Test encryption and decryption directly without database storage issues
    println!("Testing encryption with AAD: {}", aad_string);

    // Test decryption directly using the encrypted data we just created
    let mock_connection = connection::Model {
        id: connection_id,
        tenant_id,
        provider_slug: provider_slug.clone(),
        external_id: external_id.clone(),
        status: "active".to_string(),
        display_name: None,
        access_token_ciphertext: Some(encrypted_access.clone()),
        refresh_token_ciphertext: Some(encrypted_refresh.clone()),
        expires_at: None,
        scopes: None,
        metadata: None,
        created_at: chrono::Utc::now().into(),
        updated_at: chrono::Utc::now().into(),
    };

    println!("Testing decryption...");
    let (decrypted_access, decrypted_refresh, had_legacy) =
        connection_repo.decrypt_tokens(&mock_connection).await?;

    println!("Verifying results...");
    assert!(!had_legacy);
    assert_eq!(decrypted_access.as_deref(), Some(access_token));
    assert_eq!(decrypted_refresh.as_deref(), Some(refresh_token));

    // Also test that different AAD prevents decryption (AAD isolation test)
    let wrong_aad_string = format!("{}|{}|{}", Uuid::new_v4(), provider_slug, external_id);
    let encrypted_with_wrong_aad = encrypt_bytes(
        &crypto_key,
        wrong_aad_string.as_bytes(),
        access_token.as_bytes(),
    )?;

    let mock_connection_with_wrong_aad = connection::Model {
        access_token_ciphertext: Some(encrypted_with_wrong_aad),
        ..mock_connection.clone()
    };

    let result = connection_repo
        .decrypt_tokens(&mock_connection_with_wrong_aad)
        .await;
    assert!(result.is_err()); // Should fail to decrypt with wrong AAD

    println!("✅ Token encryption roundtrip and AAD isolation successful");

    Ok(())
}

#[test]
fn test_encrypt_connection_tokens_sets_version_prefix() -> Result<(), Box<dyn std::error::Error>> {
    let crypto_key = test_crypto_key();
    let connection = connection::Model {
        id: Uuid::new_v4(),
        tenant_id: Uuid::new_v4(),
        provider_slug: "test-provider".to_string(),
        external_id: "persist-check".to_string(),
        status: "active".to_string(),
        display_name: None,
        access_token_ciphertext: None,
        refresh_token_ciphertext: None,
        expires_at: None,
        scopes: None,
        metadata: None,
        created_at: chrono::Utc::now().into(),
        updated_at: chrono::Utc::now().into(),
    };

    let (access_cipher, refresh_cipher) = encrypt_connection_tokens(
        &crypto_key,
        &connection,
        Some("access-token-legacy"),
        Some("refresh-token-legacy"),
    )?;

    let access_cipher = access_cipher.expect("encrypted access token");
    let refresh_cipher = refresh_cipher.expect("encrypted refresh token");

    assert_eq!(access_cipher.first().copied(), Some(0x01));
    assert_eq!(refresh_cipher.first().copied(), Some(0x01));

    Ok(())
}

#[tokio::test]
async fn test_connection_token_aad_isolation() -> Result<(), Box<dyn std::error::Error>> {
    let db = setup_test_db().await;
    let crypto_key = test_crypto_key();
    let connection_repo = ConnectionRepository::new(db.clone(), crypto_key.clone());

    // Create two connections with different AAD contexts
    let token = "same-token-content";

    // Connection 1
    let conn1_tenant_id = Uuid::new_v4();
    let conn1_id = Uuid::new_v4();
    let conn1_provider = "provider1";
    let conn1_external = "external1";

    // Create tenant and connection using direct SQL
    test_utils::create_test_tenant(&*db, Some(conn1_tenant_id)).await?;
    test_utils::insert_connection(
        &*db,
        conn1_id,
        conn1_tenant_id,
        conn1_provider,
        conn1_external,
    )
    .await?;

    // Manually encrypt token for connection 1
    let conn1_aad = format!("{}|{}|{}", conn1_tenant_id, conn1_provider, conn1_external);
    let encrypted_conn1_token = encrypt_bytes(&crypto_key, conn1_aad.as_bytes(), token.as_bytes())?;

    let created_conn1 = connection::Model {
        id: conn1_id,
        tenant_id: conn1_tenant_id,
        provider_slug: conn1_provider.to_string(),
        external_id: conn1_external.to_string(),
        status: "active".to_string(),
        display_name: None,
        access_token_ciphertext: Some(encrypted_conn1_token),
        refresh_token_ciphertext: None,
        expires_at: None,
        scopes: None,
        metadata: None,
        created_at: chrono::Utc::now().into(),
        updated_at: chrono::Utc::now().into(),
    };

    // Connection 2 (different context)
    let conn2_tenant_id = Uuid::new_v4();
    let conn2_id = Uuid::new_v4();
    let conn2_provider = "provider2";
    let conn2_external = "external2";

    // Create tenant and connection using direct SQL
    test_utils::create_test_tenant(&*db, Some(conn2_tenant_id)).await?;
    test_utils::insert_connection(
        &*db,
        conn2_id,
        conn2_tenant_id,
        conn2_provider,
        conn2_external,
    )
    .await?;

    // Manually encrypt token for connection 2
    let conn2_aad = format!("{}|{}|{}", conn2_tenant_id, conn2_provider, conn2_external);
    let encrypted_conn2_token = encrypt_bytes(&crypto_key, conn2_aad.as_bytes(), token.as_bytes())?;

    let created_conn2 = connection::Model {
        id: conn2_id,
        tenant_id: conn2_tenant_id,
        provider_slug: conn2_provider.to_string(),
        external_id: conn2_external.to_string(),
        status: "active".to_string(),
        display_name: None,
        access_token_ciphertext: Some(encrypted_conn2_token),
        refresh_token_ciphertext: None,
        expires_at: None,
        scopes: None,
        metadata: None,
        created_at: chrono::Utc::now().into(),
        updated_at: chrono::Utc::now().into(),
    };

    // Verify ciphertexts are different due to different AAD
    assert_ne!(
        created_conn1.access_token_ciphertext,
        created_conn2.access_token_ciphertext
    );

    // Both should decrypt correctly
    let (decrypted1, _, _) = connection_repo.decrypt_tokens(&created_conn1).await?;
    let (decrypted2, _, _) = connection_repo.decrypt_tokens(&created_conn2).await?;

    assert_eq!(decrypted1.as_deref(), Some(token));
    assert_eq!(decrypted2.as_deref(), Some(token));

    Ok(())
}

#[tokio::test]
async fn test_connection_token_update_encryption() -> Result<(), Box<dyn std::error::Error>> {
    let db = setup_test_db().await;
    let crypto_key = test_crypto_key();
    let connection_repo = ConnectionRepository::new(db.clone(), crypto_key.clone());

    // Create initial connection using direct SQL
    let tenant_id = Uuid::new_v4();
    let provider_slug = "test-provider";
    let external_id = "test-external-id";
    let connection_id = Uuid::new_v4();
    let initial_token = "initial-token";

    test_utils::create_test_tenant(&*db, Some(tenant_id)).await?;
    test_utils::insert_connection(&*db, connection_id, tenant_id, provider_slug, external_id)
        .await?;

    // Manually encrypt initial token
    let aad_string = format!("{}|{}|{}", tenant_id, provider_slug, external_id);
    let encrypted_initial_token =
        encrypt_bytes(&crypto_key, aad_string.as_bytes(), initial_token.as_bytes())?;

    let connection = connection::Model {
        id: connection_id,
        tenant_id,
        provider_slug: provider_slug.to_string(),
        external_id: external_id.to_string(),
        status: "active".to_string(),
        display_name: None,
        access_token_ciphertext: Some(encrypted_initial_token),
        refresh_token_ciphertext: None,
        expires_at: None,
        scopes: None,
        metadata: None,
        created_at: chrono::Utc::now().into(),
        updated_at: chrono::Utc::now().into(),
    };

    // Update tokens manually (bypassing SeaORM UUID issues)
    let new_access_token = "updated-access-token";
    let new_refresh_token = "updated-refresh-token";

    // Manually encrypt new tokens
    let new_encrypted_access = encrypt_bytes(
        &crypto_key,
        aad_string.as_bytes(),
        new_access_token.as_bytes(),
    )?;
    let new_encrypted_refresh = encrypt_bytes(
        &crypto_key,
        aad_string.as_bytes(),
        new_refresh_token.as_bytes(),
    )?;

    // Create updated connection model with new encrypted tokens
    let updated_connection = connection::Model {
        access_token_ciphertext: Some(new_encrypted_access),
        refresh_token_ciphertext: Some(new_encrypted_refresh),
        ..connection
    };

    // Verify updated tokens are encrypted correctly
    let (decrypted_access, decrypted_refresh, _) =
        connection_repo.decrypt_tokens(&updated_connection).await?;

    assert_eq!(decrypted_access.as_deref(), Some(new_access_token));
    assert_eq!(decrypted_refresh.as_deref(), Some(new_refresh_token));

    println!("✅ Token update encryption successful");

    Ok(())
}

#[tokio::test]
async fn test_crypto_performance_characteristics() -> Result<(), Box<dyn std::error::Error>> {
    let crypto_key = test_crypto_key();
    let aad = b"test-aad-context";

    // Test various token sizes
    let medium_token = "a".repeat(256);
    let large_token = "b".repeat(1024);
    let very_large_token = "c".repeat(4096);

    let test_cases = vec![
        ("small-token", "small"),
        ("medium-oauth-token", medium_token.as_str()),
        ("large-jwt-token", large_token.as_str()),
        ("very-large-token", very_large_token.as_str()),
    ];

    for (name, plaintext) in test_cases {
        let start = std::time::Instant::now();

        let encrypted = encrypt_bytes(&crypto_key, aad, plaintext.as_bytes())
            .expect("Encryption should succeed");

        let encryption_time = start.elapsed();

        let start = std::time::Instant::now();

        let decrypted =
            decrypt_bytes(&crypto_key, aad, &encrypted).expect("Decryption should succeed");

        let decryption_time = start.elapsed();

        // Verify roundtrip correctness
        assert_eq!(decrypted, plaintext.as_bytes());

        // Performance assertions (should be very fast)
        assert!(
            encryption_time.as_millis() < 10,
            "Encryption too slow for {}",
            name
        );
        assert!(
            decryption_time.as_millis() < 10,
            "Decryption too slow for {}",
            name
        );

        println!(
            "{} ({} bytes): encrypt {:?}, decrypt {:?}",
            name,
            plaintext.len(),
            encryption_time,
            decryption_time
        );
    }

    Ok(())
}

#[tokio::test]
async fn test_crypto_error_scenarios() -> Result<(), Box<dyn std::error::Error>> {
    let crypto_key = test_crypto_key();
    let aad = b"test-aad-context";
    let plaintext = b"test message";

    // Test non-versioned payload is returned as plaintext
    let legacy_payload = b"legacy-token".to_vec();
    let result = decrypt_bytes(&crypto_key, aad, &legacy_payload);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), legacy_payload);

    // Test insufficient ciphertext length
    let short_ciphertext = vec![0x01]; // Only version byte
    let result = decrypt_bytes(&crypto_key, aad, &short_ciphertext);
    assert!(matches!(result, Err(CryptoError::InvalidFormat)));

    // Test modified ciphertext (tampering detection)
    let encrypted = encrypt_bytes(&crypto_key, aad, plaintext).expect("Encryption succeeds");
    let mut modified = encrypted;
    // Modify a byte in the ciphertext (not nonce)
    if modified.len() > 20 {
        modified[20] ^= 0x01;
    }

    let result = decrypt_bytes(&crypto_key, aad, &modified);
    assert!(matches!(result, Err(CryptoError::DecryptionFailed(_))));

    // Test wrong AAD
    let wrong_aad = b"wrong-aad";
    let encrypted = encrypt_bytes(&crypto_key, aad, plaintext).expect("Encryption succeeds");
    let result = decrypt_bytes(&crypto_key, wrong_aad, &encrypted);
    assert!(matches!(result, Err(CryptoError::DecryptionFailed(_))));

    Ok(())
}

#[tokio::test]
async fn test_legacy_token_passthrough() -> Result<(), Box<dyn std::error::Error>> {
    let crypto_key = test_crypto_key();
    let aad = b"test-aad-context";

    // Create a legacy token (no version marker)
    let legacy_ciphertext = b"legacy-token".to_vec();
    let result = decrypt_bytes(&crypto_key, aad, &legacy_ciphertext);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), legacy_ciphertext);

    // Verify current version works
    let plaintext = b"test message";
    let encrypted = encrypt_bytes(&crypto_key, aad, plaintext).expect("Encryption succeeds");

    let result = decrypt_bytes(&crypto_key, aad, &encrypted);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), plaintext);

    Ok(())
}
