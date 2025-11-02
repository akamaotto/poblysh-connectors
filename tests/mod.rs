//! Test modules for the Connectors API.
//!
//! This module contains all test modules for the Connectors API,
//! including integration tests, unit tests, and utility functions.

pub mod auth_integration_tests;
pub mod provider_connection_integration_tests;
pub mod provider_repository_tests;
pub mod provider_seeding_tests;
pub mod tenant_isolation_tests;
pub mod test_utils;

// Re-export existing test modules
pub mod integration;
