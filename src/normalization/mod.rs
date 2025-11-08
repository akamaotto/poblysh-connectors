use std::fmt;

use serde_json::Value;
use thiserror::Error;

/// Canonical registry of supported `Signal.kind` values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SignalKind {
    IssueCreated,
    IssueUpdated,
    IssueClosed,
    IssueReopened,
    IssueResolved,
    IssueComment,
    PrOpened,
    PrClosed,
    PrMerged,
    PrReopened,
    PrUpdated,
    PrReview,
    CodePushed,
    ReleasePublished,
    MessagePosted,
    MessageUpdated,
    MessageDeleted,
    ReactionAdded,
    FileCreated,
    FileUpdated,
    FileDeleted,
    FileMoved,
    CalendarEventCreated,
    CalendarEventUpdated,
    CalendarEventDeleted,
    EmailReceived,
    EmailSent,
    EmailUpdated,
    EmailDeleted,
}

impl SignalKind {
    /// Return the canonical string representation for this kind.
    pub const fn as_str(self) -> &'static str {
        match self {
            SignalKind::IssueCreated => "issue_created",
            SignalKind::IssueUpdated => "issue_updated",
            SignalKind::IssueClosed => "issue_closed",
            SignalKind::IssueReopened => "issue_reopened",
            SignalKind::IssueResolved => "issue_resolved",
            SignalKind::IssueComment => "issue_comment",
            SignalKind::PrOpened => "pr_opened",
            SignalKind::PrClosed => "pr_closed",
            SignalKind::PrMerged => "pr_merged",
            SignalKind::PrReopened => "pr_reopened",
            SignalKind::PrUpdated => "pr_updated",
            SignalKind::PrReview => "pr_review",
            SignalKind::CodePushed => "code_pushed",
            SignalKind::ReleasePublished => "release_published",
            SignalKind::MessagePosted => "message_posted",
            SignalKind::MessageUpdated => "message_updated",
            SignalKind::MessageDeleted => "message_deleted",
            SignalKind::ReactionAdded => "reaction_added",
            SignalKind::FileCreated => "file_created",
            SignalKind::FileUpdated => "file_updated",
            SignalKind::FileDeleted => "file_deleted",
            SignalKind::FileMoved => "file_moved",
            SignalKind::CalendarEventCreated => "calendar_event_created",
            SignalKind::CalendarEventUpdated => "calendar_event_updated",
            SignalKind::CalendarEventDeleted => "calendar_event_deleted",
            SignalKind::EmailReceived => "email_received",
            SignalKind::EmailSent => "email_sent",
            SignalKind::EmailUpdated => "email_updated",
            SignalKind::EmailDeleted => "email_deleted",
        }
    }
}

impl fmt::Display for SignalKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Complete registry of canonical kinds.
pub const ALL_SIGNAL_KINDS: &[SignalKind] = &[
    SignalKind::IssueCreated,
    SignalKind::IssueUpdated,
    SignalKind::IssueClosed,
    SignalKind::IssueReopened,
    SignalKind::IssueResolved,
    SignalKind::IssueComment,
    SignalKind::PrOpened,
    SignalKind::PrClosed,
    SignalKind::PrMerged,
    SignalKind::PrReopened,
    SignalKind::PrUpdated,
    SignalKind::PrReview,
    SignalKind::CodePushed,
    SignalKind::ReleasePublished,
    SignalKind::MessagePosted,
    SignalKind::MessageUpdated,
    SignalKind::MessageDeleted,
    SignalKind::ReactionAdded,
    SignalKind::FileCreated,
    SignalKind::FileUpdated,
    SignalKind::FileDeleted,
    SignalKind::FileMoved,
    SignalKind::CalendarEventCreated,
    SignalKind::CalendarEventUpdated,
    SignalKind::CalendarEventDeleted,
    SignalKind::EmailReceived,
    SignalKind::EmailSent,
    SignalKind::EmailUpdated,
    SignalKind::EmailDeleted,
];

/// Returns `true` when the provided string matches a canonical kind.
pub fn is_canonical_kind(kind: &str) -> bool {
    ALL_SIGNAL_KINDS.iter().any(|k| k.as_str() == kind)
}

/// Return the canonical kind corresponding to the provided string, if any.
pub fn parse_signal_kind(kind: &str) -> Option<SignalKind> {
    ALL_SIGNAL_KINDS
        .iter()
        .copied()
        .find(|k| k.as_str() == kind)
}

/// Errors that can occur while mapping provider payloads to canonical kinds.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum NormalizationError {
    #[error("provider not implemented: {0}")]
    ProviderNotImplemented(&'static str),
    #[error("payload missing required field: {field}")]
    MissingField { field: &'static str },
    #[error("unsupported payload variant: {0}")]
    Unsupported(&'static str),
}

/// Normalize the stub example payloads used in fixtures and sample connectors.
pub fn normalize_example_payload(payload: &Value) -> Result<SignalKind, NormalizationError> {
    if let Some(action) = payload.get("action").and_then(|v| v.as_str()) {
        return match action {
            "created" => Ok(SignalKind::IssueCreated),
            "closed" => {
                let merged = payload
                    .get("merged")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if merged {
                    Ok(SignalKind::PrMerged)
                } else {
                    Ok(SignalKind::PrClosed)
                }
            }
            _ => Err(NormalizationError::Unsupported("example.action")),
        };
    }

    if let Some(message_type) = payload.get("type").and_then(|v| v.as_str()) {
        return match message_type {
            "message" => Ok(SignalKind::MessagePosted),
            _ => Err(NormalizationError::Unsupported("example.message_type")),
        };
    }

    Err(NormalizationError::MissingField {
        field: "action|type",
    })
}

/// Normalize Jira webhook payloads into canonical kinds.
pub fn normalize_jira_webhook_kind(payload: &Value) -> Option<SignalKind> {
    let event_type = payload.get("webhookEvent").and_then(|v| v.as_str())?;

    match event_type {
        "jira:issue_created" => Some(SignalKind::IssueCreated),
        "jira:issue_updated" => Some(SignalKind::IssueUpdated),
        _ => None,
    }
}

/// Normalize Zoho Cliq webhook payloads into canonical kinds.
pub fn normalize_zoho_cliq_webhook_kind(payload: &Value) -> Result<SignalKind, NormalizationError> {
    let event_type = payload.get("event_type").and_then(|v| v.as_str()).ok_or(
        NormalizationError::MissingField {
            field: "event_type",
        },
    )?;

    match event_type {
        "message_posted" => Ok(SignalKind::MessagePosted),
        "message_updated" => Ok(SignalKind::MessageUpdated),
        "message_deleted" => Ok(SignalKind::MessageDeleted),
        _ => Err(NormalizationError::Unsupported("zoho_cliq.event_type")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn registry_has_unique_entries() {
        let mut seen = HashSet::new();
        for kind in ALL_SIGNAL_KINDS {
            assert!(seen.insert(kind.as_str()), "duplicate kind {}", kind);
        }
    }

    #[test]
    fn parse_round_trips() {
        for kind in ALL_SIGNAL_KINDS {
            let parsed = parse_signal_kind(kind.as_str()).expect("kind should parse");
            assert_eq!(*kind, parsed);
        }
    }
}
