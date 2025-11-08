//! # Common API Types
//!
//! This module contains shared types used across multiple API handlers,
//! including common response structures and pagination utilities.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Generic paginated response wrapper for list endpoints
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PaginatedResponse<T> {
    /// List of items for the current page
    pub data: Vec<T>,
    /// Opaque cursor for fetching the next page (null if this is the last page)
    pub next_cursor: Option<String>,
    /// Convenience field indicating if more pages exist
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_more: Option<bool>,
}

impl<T> PaginatedResponse<T> {
    /// Create a new paginated response
    pub fn new(data: Vec<T>, next_cursor: Option<String>) -> Self {
        let has_more = next_cursor.is_some();
        Self {
            data,
            next_cursor,
            has_more: Some(has_more),
        }
    }

    /// Create a response with no more pages
    pub fn final_page(data: Vec<T>) -> Self {
        Self {
            data,
            next_cursor: None,
            has_more: Some(false),
        }
    }
}
