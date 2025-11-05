use anyhow::{Context, Result, anyhow};
use chrono::Utc;
use connectors::{
    config::ConfigLoader,
    crypto::{CryptoKey, encrypt_bytes, is_encrypted_payload},
    db,
    models::connection,
};
use sea_orm::{ActiveModelTrait, EntityTrait, Set};

#[tokio::main]
async fn main() -> Result<()> {
    let loader = ConfigLoader::new();
    let config = loader.load().context("loading configuration")?;

    let key_bytes = config
        .crypto_key
        .clone()
        .context("crypto key not present in configuration")?;
    let crypto_key = CryptoKey::new(key_bytes).context("initializing crypto key")?;

    let db = db::init_pool(&config)
        .await
        .context("initializing database connection pool")?;

    let connections = connection::Entity::find()
        .all(&db)
        .await
        .context("querying connections")?;

    let mut updated_count = 0usize;

    for conn in connections {
        let connection_id = conn.id;
        let aad = format!(
            "{}|{}|{}",
            conn.tenant_id, conn.provider_slug, conn.external_id
        );

        let mut new_access_cipher = None;
        if let Some(access) = conn.access_token_ciphertext.as_ref()
            && !access.is_empty()
            && !is_encrypted_payload(access)
        {
            let ciphertext = encrypt_bytes(&crypto_key, aad.as_bytes(), access).map_err(|err| {
                anyhow!(
                    "failed to encrypt access token for {}: {}",
                    connection_id,
                    err
                )
            })?;
            new_access_cipher = Some(ciphertext);
        }

        let mut new_refresh_cipher = None;
        if let Some(refresh) = conn.refresh_token_ciphertext.as_ref()
            && !refresh.is_empty()
            && !is_encrypted_payload(refresh)
        {
            let ciphertext =
                encrypt_bytes(&crypto_key, aad.as_bytes(), refresh).map_err(|err| {
                    anyhow!(
                        "failed to encrypt refresh token for {}: {}",
                        connection_id,
                        err
                    )
                })?;
            new_refresh_cipher = Some(ciphertext);
        }

        if new_access_cipher.is_none() && new_refresh_cipher.is_none() {
            continue;
        }

        let mut active: connection::ActiveModel = conn.into();
        if let Some(cipher) = new_access_cipher {
            active.access_token_ciphertext = Set(Some(cipher));
        }
        if let Some(cipher) = new_refresh_cipher {
            active.refresh_token_ciphertext = Set(Some(cipher));
        }
        active.updated_at = Set(Utc::now().into());

        active
            .update(&db)
            .await
            .with_context(|| format!("updating connection {}", connection_id))?;
        updated_count += 1;
    }

    println!(
        "Re-encrypted {} connection(s) containing legacy plaintext tokens.",
        updated_count
    );

    Ok(())
}
