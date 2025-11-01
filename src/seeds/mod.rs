//! Database seeding functionality
//!
//! This module provides functionality to seed the database with initial data.
//! It includes seeding for providers and other entities that need to be
//! populated when the application starts.

pub mod provider;

pub use provider::seed_providers;
