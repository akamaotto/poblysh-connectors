//! Provider seeding functionality
//!
//! This module provides functionality to seed the providers table with
//! initial data for common OAuth providers.

use anyhow::Result;
use chrono::Utc;
use sea_orm::{DatabaseConnection, Set};
use std::sync::Arc;

use crate::models::provider;
use crate::repositories::ProviderRepository;

/// Seeds the providers table with common OAuth providers
///
/// This function checks if common OAuth providers already exist in the database
/// and creates them if they don't. It's useful for bootstrapping the system
/// with initial provider data.
///
/// # Arguments
///
/// * `db` - Database connection
///
/// # Returns
///
/// Returns a Result indicating success or failure
pub async fn seed_providers(db: &DatabaseConnection) -> Result<()> {
    let repo = ProviderRepository::new(Arc::new(db.clone()));

    // Define common OAuth providers with their configurations
    let providers = vec![
        ProviderConfig {
            slug: "google".to_string(),
            display_name: "Google".to_string(),
            auth_type: "oauth2".to_string(),
        },
        ProviderConfig {
            slug: "github".to_string(),
            display_name: "GitHub".to_string(),
            auth_type: "oauth2".to_string(),
        },
        ProviderConfig {
            slug: "jira".to_string(),
            display_name: "Jira".to_string(),
            auth_type: "oauth2".to_string(),
        },
        ProviderConfig {
            slug: "microsoft".to_string(),
            display_name: "Microsoft".to_string(),
            auth_type: "oauth2".to_string(),
        },
    ];

    for provider_config in providers {
        // Check if provider already exists
        match repo.find_by_slug(&provider_config.slug).await {
            Ok(Some(_)) => {
                log::info!(
                    "Provider '{}' already exists, skipping",
                    provider_config.slug
                );
                continue;
            }
            Ok(None) => {
                // Provider doesn't exist, create it
                log::info!("Creating provider: {}", provider_config.slug);

                let provider = provider::ActiveModel {
                    slug: Set(provider_config.slug.clone()),
                    display_name: Set(provider_config.display_name.clone()),
                    auth_type: Set(provider_config.auth_type.clone()),
                    created_at: Set(Utc::now().into()),
                    updated_at: Set(Utc::now().into()),
                };

                let slug = provider_config.slug.clone();
                match repo.create(provider).await {
                    Ok(_) => {
                        log::info!("Successfully created provider: {}", slug);
                    }
                    Err(e) => {
                        log::error!(
                            "Failed to create provider '{}': {}",
                            provider_config.slug,
                            e
                        );
                        return Err(e);
                    }
                }
            }
            Err(e) => {
                log::error!(
                    "Error checking if provider '{}' exists: {}",
                    provider_config.slug,
                    e
                );
                return Err(e);
            }
        }
    }

    log::info!("Provider seeding completed successfully");
    Ok(())
}

/// Configuration structure for a provider
struct ProviderConfig {
    slug: String,
    display_name: String,
    auth_type: String,
}
