//! # Repository Layer
//!
//! This module contains repository implementations that encapsulate SeaORM operations
//! for database entities, providing a clean API for data access with tenant-aware methods.

pub mod connection;
pub mod oauth_state;
pub mod provider;
pub mod sync_job;
pub mod sync_metadata;

pub use connection::ConnectionRepository;
pub use oauth_state::OAuthStateRepository;
pub use provider::ProviderRepository;
pub use sync_job::{ListJobsConfig, ListJobsResult, SyncJobRepository};
pub use sync_metadata::{ConnectionSyncMetadata, MIN_SYNC_INTERVAL_SECONDS};
