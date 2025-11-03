//! Token encryption module using AES-256-GCM
//!
//! This module provides encryption and decryption utilities for access tokens
//! and refresh tokens stored in the database, using AES-256-GCM with additional
//! authenticated data (AAD) for context binding.

#![allow(deprecated)]

use aes_gcm::{
    Aes256Gcm, Key, Nonce,
    aead::{Aead, AeadCore, KeyInit, OsRng, Payload},
};
use thiserror::Error;
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::models::connection::Model as ConnectionModel;

const VERSION_ENCRYPTED: u8 = 0x01;
const VERSION_FIELD_LEN: usize = 1;
const NONCE_LEN: usize = 12;
const TAG_LEN: usize = 16;
const MIN_ENCRYPTED_LEN: usize = VERSION_FIELD_LEN + NONCE_LEN + TAG_LEN;

/// Crypto error types
#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("encryption failed: {0}")]
    EncryptionFailed(String),
    #[error("decryption failed: {0}")]
    DecryptionFailed(String),
    #[error("invalid ciphertext format")]
    InvalidFormat,
    #[error("empty ciphertext")]
    EmptyCiphertext,
}

/// Secure wrapper for encryption keys with zeroization
#[derive(Debug, Clone, Zeroize, ZeroizeOnDrop)]
pub struct ZeroizingKey(Vec<u8>);

/// Type alias for crypto keys
pub type CryptoKey = ZeroizingKey;

impl CryptoKey {
    /// Create a new crypto key from bytes
    pub fn new(bytes: Vec<u8>) -> Result<Self, CryptoError> {
        if bytes.len() != 32 {
            return Err(CryptoError::EncryptionFailed(
                "Invalid key length: expected 32 bytes".to_string(),
            ));
        }
        Ok(ZeroizingKey(bytes))
    }

    /// Get the key as bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// Encrypt bytes using AES-256-GCM
pub fn encrypt_bytes(
    key: &CryptoKey,
    aad: &[u8],
    plaintext: &[u8],
) -> Result<Vec<u8>, CryptoError> {
    // Create cipher
    let cipher_key = Key::<Aes256Gcm>::from_slice(key.as_bytes());
    let cipher = Aes256Gcm::new(cipher_key);

    // Generate random nonce
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    // Encrypt with AAD
    let mut ciphertext = cipher
        .encrypt(
            &nonce,
            Payload {
                msg: plaintext,
                aad,
            },
        )
        .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;

    // Prepend version byte and nonce to ciphertext
    let mut result = Vec::with_capacity(VERSION_FIELD_LEN + NONCE_LEN + ciphertext.len());
    result.push(VERSION_ENCRYPTED); // Version byte for encrypted tokens
    result.extend_from_slice(&nonce);
    result.append(&mut ciphertext);

    Ok(result)
}

/// Decrypt bytes using AES-256-GCM
pub fn decrypt_bytes(
    key: &CryptoKey,
    aad: &[u8],
    ciphertext: &[u8],
) -> Result<Vec<u8>, CryptoError> {
    if ciphertext.is_empty() {
        return Err(CryptoError::EmptyCiphertext);
    }

    // Detect legacy plaintext payloads (no version marker)
    if ciphertext[0] != VERSION_ENCRYPTED {
        return Ok(ciphertext.to_vec());
    }

    // Validate minimum length (version + nonce + tag)
    if ciphertext.len() < MIN_ENCRYPTED_LEN {
        return Err(CryptoError::InvalidFormat);
    }

    // Extract components
    let nonce = Nonce::from_slice(&ciphertext[VERSION_FIELD_LEN..VERSION_FIELD_LEN + NONCE_LEN]);
    let tag_and_ct = &ciphertext[VERSION_FIELD_LEN + NONCE_LEN..];

    debug_assert!(tag_and_ct.len() >= TAG_LEN);

    // Reconstruct ciphertext + tag for decryption
    let mut reconstructed = Vec::with_capacity(tag_and_ct.len());
    reconstructed.extend_from_slice(tag_and_ct);

    // Create cipher
    let cipher_key = Key::<Aes256Gcm>::from_slice(key.as_bytes());
    let cipher = Aes256Gcm::new(cipher_key);

    // Decrypt with AAD
    cipher
        .decrypt(
            nonce,
            Payload {
                msg: &reconstructed,
                aad,
            },
        )
        .map_err(|e| CryptoError::DecryptionFailed(e.to_string()))
}

/// Determine if a payload is using the encrypted format
pub fn is_encrypted_payload(ciphertext: &[u8]) -> bool {
    ciphertext.len() >= MIN_ENCRYPTED_LEN && ciphertext[0] == VERSION_ENCRYPTED
}

/// Type alias for encrypted token result
type EncryptedTokens = Result<(Option<Vec<u8>>, Option<Vec<u8>>), CryptoError>;

/// Encrypt tokens for a connection model
pub fn encrypt_connection_tokens(
    key: &CryptoKey,
    connection: &ConnectionModel,
    access_token: Option<&str>,
    refresh_token: Option<&str>,
) -> EncryptedTokens {
    let aad = format!(
        "{}|{}|{}",
        connection.tenant_id, connection.provider_slug, connection.external_id
    );

    let encrypted_access_token = access_token
        .map(|token| encrypt_bytes(key, aad.as_bytes(), token.as_bytes()))
        .transpose()?;

    let encrypted_refresh_token = refresh_token
        .map(|token| encrypt_bytes(key, aad.as_bytes(), token.as_bytes()))
        .transpose()?;

    Ok((encrypted_access_token, encrypted_refresh_token))
}

/// Type alias for decrypted token result
type DecryptedTokens = Result<(Option<String>, Option<String>), CryptoError>;

/// Decrypt tokens for a connection model
pub fn decrypt_connection_tokens(key: &CryptoKey, connection: &ConnectionModel) -> DecryptedTokens {
    let aad = format!(
        "{}|{}|{}",
        connection.tenant_id, connection.provider_slug, connection.external_id
    );

    let decrypted_access_token = match connection.access_token_ciphertext.as_ref() {
        Some(token) if is_encrypted_payload(token) => decrypt_bytes(key, aad.as_bytes(), token)
            .and_then(|bytes| {
                String::from_utf8(bytes)
                    .map_err(|e| CryptoError::DecryptionFailed(format!("Invalid UTF-8: {}", e)))
            })
            .map(Some),
        Some(token) => String::from_utf8(token.clone())
            .map(Some)
            .map_err(|e| CryptoError::DecryptionFailed(format!("Invalid UTF-8: {}", e))),
        None => Ok(None),
    }?;

    let decrypted_refresh_token = match connection.refresh_token_ciphertext.as_ref() {
        Some(token) if is_encrypted_payload(token) => decrypt_bytes(key, aad.as_bytes(), token)
            .and_then(|bytes| {
                String::from_utf8(bytes)
                    .map_err(|e| CryptoError::DecryptionFailed(format!("Invalid UTF-8: {}", e)))
            })
            .map(Some),
        Some(token) => String::from_utf8(token.clone())
            .map(Some)
            .map_err(|e| CryptoError::DecryptionFailed(format!("Invalid UTF-8: {}", e))),
        None => Ok(None),
    }?;

    Ok((decrypted_access_token, decrypted_refresh_token))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    fn test_key() -> CryptoKey {
        CryptoKey::new(vec![0u8; 32]).expect("valid test key")
    }

    fn sample_connection(
        access_token_ciphertext: Option<Vec<u8>>,
        refresh_token_ciphertext: Option<Vec<u8>>,
    ) -> ConnectionModel {
        ConnectionModel {
            id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            provider_slug: "test-provider".to_string(),
            external_id: "external-123".to_string(),
            status: "active".to_string(),
            display_name: None,
            access_token_ciphertext,
            refresh_token_ciphertext,
            expires_at: None,
            scopes: None,
            metadata: None,
            created_at: Utc::now().into(),
            updated_at: Utc::now().into(),
        }
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = test_key();
        let aad = b"test-aad";
        let plaintext = b"secret message";

        let encrypted = encrypt_bytes(&key, aad, plaintext).expect("encryption succeeds");
        let decrypted = decrypt_bytes(&key, aad, &encrypted).expect("decryption succeeds");

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_different_aad_fails() {
        let key = test_key();
        let aad1 = b"test-aad-1";
        let aad2 = b"test-aad-2";
        let plaintext = b"secret message";

        let encrypted = encrypt_bytes(&key, aad1, plaintext).expect("encryption succeeds");
        let result = decrypt_bytes(&key, aad2, &encrypted);

        assert!(result.is_err());
    }

    #[test]
    fn test_modified_ciphertext_fails() {
        let key = test_key();
        let aad = b"test-aad";
        let plaintext = b"secret message";

        let mut encrypted = encrypt_bytes(&key, aad, plaintext).expect("encryption succeeds");
        // Modify a byte in the ciphertext
        encrypted[13] ^= 0x01;

        let result = decrypt_bytes(&key, aad, &encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_plaintext_works() {
        let key = test_key();
        let aad = b"test-aad";
        let plaintext = b"";

        let encrypted = encrypt_bytes(&key, aad, plaintext).expect("encryption succeeds");
        let decrypted = decrypt_bytes(&key, aad, &encrypted).expect("decryption succeeds");

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_nonce_uniqueness() {
        let key = test_key();
        let aad = b"test-aad";
        let plaintext = b"secret message";

        let encrypted1 = encrypt_bytes(&key, aad, plaintext).expect("encryption succeeds");
        let encrypted2 = encrypt_bytes(&key, aad, plaintext).expect("encryption succeeds");

        // Nonces (bytes 1-13) should be different
        assert_ne!(&encrypted1[1..13], &encrypted2[1..13]);
        // But both should decrypt correctly
        let decrypted1 = decrypt_bytes(&key, aad, &encrypted1).expect("decryption succeeds");
        let decrypted2 = decrypt_bytes(&key, aad, &encrypted2).expect("decryption succeeds");
        assert_eq!(decrypted1, plaintext);
        assert_eq!(decrypted2, plaintext);
    }

    #[test]
    fn test_legacy_token_passthrough() {
        let key = test_key();
        let aad = b"test-aad";
        let legacy_ciphertext = b"legacy-token".to_vec(); // No version marker

        let result =
            decrypt_bytes(&key, aad, &legacy_ciphertext).expect("legacy plaintext is returned");
        assert_eq!(result, legacy_ciphertext);
    }

    #[test]
    fn test_is_encrypted_payload_detection() {
        let key = test_key();
        let aad = b"test-aad";
        let encrypted = encrypt_bytes(&key, aad, b"secret").expect("encryption succeeds");

        assert!(is_encrypted_payload(&encrypted));
        assert!(!is_encrypted_payload(b"legacy"));
    }

    #[test]
    fn test_decrypt_connection_tokens_handles_legacy_plaintext() {
        let key = test_key();
        let mut connection = sample_connection(Some(b"legacy-access".to_vec()), None);
        let aad = format!(
            "{}|{}|{}",
            connection.tenant_id, connection.provider_slug, connection.external_id
        );

        let refresh_ciphertext =
            encrypt_bytes(&key, aad.as_bytes(), b"refresh-token").expect("encryption succeeds");
        connection.refresh_token_ciphertext = Some(refresh_ciphertext.clone());

        let (access, refresh) =
            decrypt_connection_tokens(&key, &connection).expect("decryption succeeds");

        assert_eq!(access.as_deref(), Some("legacy-access"));
        assert_eq!(refresh.as_deref(), Some("refresh-token"));
        assert!(is_encrypted_payload(&refresh_ciphertext));
        assert!(!is_encrypted_payload(b"legacy-access"));
    }

    #[test]
    fn test_non_versioned_payload_passthrough() {
        let key = test_key();
        let aad = b"test-aad";
        let invalid_ciphertext = vec![0xFF, 0x01, 0x02, 0x03]; // No 0x01 version marker

        let result = decrypt_bytes(&key, aad, &invalid_ciphertext)
            .expect("non-versioned payload returned as plaintext");
        assert_eq!(result, invalid_ciphertext);
    }

    #[test]
    fn test_invalid_key_length_rejected() {
        let result = CryptoKey::new(vec![0u8; 16]); // Too short
        assert!(result.is_err());

        let result = CryptoKey::new(vec![0u8; 64]); // Too long
        assert!(result.is_err());
    }

    #[test]
    fn test_insufficient_ciphertext_length() {
        let key = test_key();
        let aad = b"test-aad";
        let short_ciphertext = vec![VERSION_ENCRYPTED, 0x02]; // Too short for nonce + tag

        let result = decrypt_bytes(&key, aad, &short_ciphertext);
        assert!(matches!(result, Err(CryptoError::InvalidFormat)));
    }
}
