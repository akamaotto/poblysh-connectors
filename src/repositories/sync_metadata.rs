//! Helpers for reading and writing connection `metadata.sync` payloads.
//!
//! The scheduler persists interval cadence, jitter, and activation markers inside
//! `connections.metadata.sync`. This module centralizes parsing, validation, and
//! serialization so background workers and API handlers share the same contract.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value as JsonValue};
use tracing::warn;

use crate::config::SchedulerConfig;

/// Minimum override interval allowed by the scheduler (one minute).
pub const MIN_SYNC_INTERVAL_SECONDS: u64 = 60;

/// Helper to convert between cursor storage format and Cursor type
pub fn cursor_from_json(value: Option<&JsonValue>) -> Option<crate::connectors::Cursor> {
    let value = value?;

    if let JsonValue::Object(map) = value
        && map.len() == 1
        && let Some(inner) = map.get("value")
    {
        return Some(crate::connectors::Cursor::from_json(inner.clone()));
    }

    Some(crate::connectors::Cursor::from_json(value.clone()))
}

/// Helper to convert Cursor to JSON storage format
pub fn cursor_to_json(cursor: Option<&crate::connectors::Cursor>) -> Option<JsonValue> {
    cursor.map(|c| c.as_json().clone())
}

/// Metadata stored under `connections.metadata.sync`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ConnectionSyncMetadata {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interval_seconds: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_run_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_jitter_seconds: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_activated_at: Option<DateTime<Utc>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_cursor_opt"
    )]
    pub cursor: Option<crate::connectors::Cursor>,
}

fn deserialize_cursor_opt<'de, D>(
    deserializer: D,
) -> Result<Option<crate::connectors::Cursor>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Option::<JsonValue>::deserialize(deserializer)?;
    Ok(cursor_from_json(value.as_ref()))
}

impl ConnectionSyncMetadata {
    /// Parse metadata from the given JSON value, falling back to defaults on errors.
    pub fn from_connection_metadata(metadata: Option<&JsonValue>) -> Self {
        let Some(metadata) = metadata else {
            return Self::default();
        };

        match metadata {
            JsonValue::Object(obj) => {
                if let Some(sync_value) = obj.get("sync") {
                    // Try to deserialize the full sync metadata
                    serde_json::from_value::<Self>(sync_value.clone()).unwrap_or_else(|_| {
                        warn!(
                            sync_value = ?sync_value,
                            "Failed to parse sync metadata; using defaults"
                        );
                        Self::default()
                    })
                } else {
                    Self::default()
                }
            }
            other => {
                warn!(
                    value = ?other,
                    "Unexpected connection metadata format; expected object with sync payload"
                );
                Self::default()
            }
        }
    }

    /// Serialize the metadata back into the existing metadata object.
    ///
    /// Unknown metadata keys are preserved.
    pub fn into_connection_metadata(&self, existing: Option<&JsonValue>) -> JsonValue {
        let mut root = match existing {
            Some(JsonValue::Object(map)) => map.clone(),
            Some(value) => {
                warn!(
                    value = ?value,
                    "Unexpected connection metadata structure; replacing with object"
                );
                Map::<String, JsonValue>::new()
            }
            None => Map::<String, JsonValue>::new(),
        };

        if self.is_empty() {
            root.remove("sync");
        } else {
            // Serialize the sync metadata directly
            let sync_value = serde_json::to_value(self).unwrap_or(JsonValue::Object(Map::new()));
            root.insert("sync".to_string(), sync_value);
        }

        JsonValue::Object(root)
    }

    /// Ensure the interval override respects scheduler bounds.
    ///
    /// Returns `true` if the metadata was modified.
    pub fn sanitize_interval(&mut self, scheduler: &SchedulerConfig) -> bool {
        if let Some(value) = self.interval_seconds
            && (value < MIN_SYNC_INTERVAL_SECONDS
                || value > scheduler.max_overridden_interval_seconds)
        {
            warn!(
                interval_seconds = value,
                max_allowed = scheduler.max_overridden_interval_seconds,
                "Invalid sync interval override; reverting to scheduler default"
            );
            self.interval_seconds = None;
            return true;
        }
        false
    }

    /// Calculate the effective base interval in seconds based on overrides and defaults.
    pub fn effective_interval_seconds(&self, scheduler: &SchedulerConfig) -> u64 {
        self.interval_seconds
            .filter(|value| {
                *value >= MIN_SYNC_INTERVAL_SECONDS
                    && *value <= scheduler.max_overridden_interval_seconds
            })
            .unwrap_or(scheduler.default_interval_seconds)
    }

    fn is_empty(&self) -> bool {
        self.interval_seconds.is_none()
            && self.next_run_at.is_none()
            && self.last_jitter_seconds.is_none()
            && self.first_activated_at.is_none()
            && self.cursor.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> SchedulerConfig {
        SchedulerConfig {
            tick_interval_seconds: 60,
            default_interval_seconds: 900,
            jitter_pct_min: 0.0,
            jitter_pct_max: 0.2,
            max_overridden_interval_seconds: 86400,
        }
    }

    #[test]
    fn parses_sync_metadata_from_object() {
        let raw = serde_json::json!({
            "sync": {
                "interval_seconds": 600,
                "last_jitter_seconds": 45,
                "next_run_at": "2025-01-01T12:00:00Z",
                "first_activated_at": "2024-12-31T12:00:00Z"
            },
            "other": { "value": 1 }
        });

        let metadata = ConnectionSyncMetadata::from_connection_metadata(Some(&raw));
        assert_eq!(metadata.interval_seconds, Some(600));
        assert_eq!(metadata.last_jitter_seconds, Some(45));
        assert_eq!(
            metadata.next_run_at,
            Some(
                DateTime::parse_from_rfc3339("2025-01-01T12:00:00Z")
                    .unwrap()
                    .with_timezone(&Utc)
            )
        );
        assert_eq!(
            metadata.first_activated_at,
            Some(
                DateTime::parse_from_rfc3339("2024-12-31T12:00:00Z")
                    .unwrap()
                    .with_timezone(&Utc)
            )
        );
    }

    #[test]
    fn ignores_invalid_sync_payload() {
        let raw = serde_json::json!({
            "sync": {
                "interval_seconds": "bad",
            }
        });

        let metadata = ConnectionSyncMetadata::from_connection_metadata(Some(&raw));
        assert_eq!(metadata, ConnectionSyncMetadata::default());
    }

    #[test]
    fn sanitize_invalid_interval() {
        let mut metadata = ConnectionSyncMetadata {
            interval_seconds: Some(10),
            ..Default::default()
        };
        assert!(metadata.sanitize_interval(&test_config()));
        assert_eq!(metadata.interval_seconds, None);
    }

    #[test]
    fn effective_interval_prefers_override() {
        let metadata = ConnectionSyncMetadata {
            interval_seconds: Some(1800),
            ..Default::default()
        };

        assert_eq!(metadata.effective_interval_seconds(&test_config()), 1800);
    }

    #[test]
    fn effective_interval_falls_back_to_default() {
        let metadata = ConnectionSyncMetadata {
            interval_seconds: Some(10),
            ..Default::default()
        };

        assert_eq!(metadata.effective_interval_seconds(&test_config()), 900);
    }

    #[test]
    fn updates_existing_metadata_object() {
        let existing = serde_json::json!({
            "sync": {
                "interval_seconds": 900
            },
            "other": { "value": 1 }
        });

        let metadata = ConnectionSyncMetadata {
            next_run_at: Some(
                DateTime::parse_from_rfc3339("2025-01-01T13:00:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
            last_jitter_seconds: Some(60),
            ..Default::default()
        };

        let updated = metadata.into_connection_metadata(Some(&existing));
        assert!(updated.get("other").is_some());

        let sync = updated.get("sync").unwrap();
        assert!(sync.get("interval_seconds").is_none());
        assert_eq!(
            sync.get("last_jitter_seconds").unwrap(),
            &JsonValue::from(60)
        );
    }
}
