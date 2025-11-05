//! Test modules for the Connectors API.
//!
//! This module contains all test modules for the Connectors API,
//! including integration tests, unit tests, and utility functions.

#[allow(clippy::duplicate_mod)]
pub mod auth_integration_tests;
#[allow(clippy::duplicate_mod)]
pub mod oauth_callback_integration_tests;
#[allow(clippy::duplicate_mod)]
pub mod provider_connection_integration_tests;
#[allow(clippy::duplicate_mod)]
pub mod provider_repository_tests;
#[allow(clippy::duplicate_mod)]
pub mod provider_seeding_tests;

#[allow(clippy::duplicate_mod)]
pub mod tenant_isolation_tests;

// Re-export existing test modules
pub mod integration;
