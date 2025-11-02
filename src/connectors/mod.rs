//! Connectors module
//!
//! This module provides the Connector SDK including:
//! - The `Connector` trait defining the interface for all provider implementations
//! - Provider metadata and registry for discovery and lookup
//! - Individual connector implementations

pub mod example;
pub mod google_drive;
pub mod jira;
pub mod metadata;
pub mod registry;
pub mod trait_;

pub use example::{ExampleConnector, register_example_connector};
pub use google_drive::{GoogleDriveConnector, register_google_drive_connector};
pub use jira::{JiraConnector, register_jira_connector};
pub use metadata::{AuthType, ProviderMetadata};
pub use registry::{Registry, RegistryError};
pub use trait_::{
    AuthorizeParams, Connector, Cursor, ExchangeTokenParams, SyncParams, WebhookParams,
};
