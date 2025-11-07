//! Connectors module
//!
//! This module provides the Connector SDK including:
//! - The `Connector` trait defining the interface for all provider implementations
//! - Provider metadata and registry for discovery and lookup
//! - Individual connector implementations

pub mod example;
pub mod github;
pub mod gmail;
pub mod google_calendar;
pub mod google_drive;
pub mod jira;
pub mod metadata;
pub mod registry;
pub mod trait_;
pub mod zoho_cliq;
pub mod zoho_mail;

pub use metadata::{AuthType, ProviderMetadata};
pub use registry::{Registry, RegistryError};
pub use trait_::{
    AuthorizeParams, Connector, ConnectorError, Cursor, ExchangeTokenParams, SyncError,
    SyncErrorKind, SyncParams, SyncResult, WebhookParams,
};
pub use zoho_mail::{
    ZOHO_MAIL_PROVIDER_SLUG, ZohoMailConfig, ZohoMailConnector, register_zoho_mail_connector,
};

pub use example::{ExampleConnector, register_example_connector};
pub use github::{GitHubConnector, register_github_connector};
pub use gmail::{GmailConnector, register_gmail_connector};
pub use google_calendar::{GoogleCalendarConnector, register_google_calendar_connector};
pub use google_drive::{GoogleDriveConnector, register_google_drive_connector};
pub use jira::{JiraConnector, register_jira_connector};
pub use zoho_cliq::{ZohoCliqConnector, register_zoho_cliq_connector};
