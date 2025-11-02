//! Connectors module
//!
//! This module provides the Connector SDK including:
//! - The `Connector` trait defining the interface for all provider implementations
//! - Provider metadata and registry for discovery and lookup
//! - Individual connector implementations

pub mod example;
pub mod metadata;
pub mod registry;
pub mod trait_;

pub use example::{ExampleConnector, register_example_connector};
pub use metadata::{AuthType, ProviderMetadata};
pub use registry::{Registry, RegistryError};
pub use trait_::{
    AuthorizeParams, Connector, Cursor, ExchangeTokenParams, SyncParams, WebhookParams,
};
