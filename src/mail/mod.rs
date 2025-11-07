//! Mail spam filtering module
//!
//! Provides a shared spam filtering abstraction that all mail connectors can use
//! to drop malicious messages while allowing legitimate promotional or collaboration
//! threads to proceed through the signal pipeline.

pub mod default;

use std::collections::HashMap;

/// Mail provider enumeration for spam filtering
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MailProvider {
    Gmail,
    ZohoMail,
    Outlook,
    Other(String),
}

impl MailProvider {
    /// Create a MailProvider from a string slug
    pub fn from_slug(slug: &str) -> Self {
        match slug.to_lowercase().as_str() {
            "gmail" => MailProvider::Gmail,
            "zoho-mail" => MailProvider::ZohoMail,
            "outlook" => MailProvider::Outlook,
            other => MailProvider::Other(other.to_string()),
        }
    }

    /// Get the string representation of the provider
    pub fn as_str(&self) -> &str {
        match self {
            MailProvider::Gmail => "gmail",
            MailProvider::ZohoMail => "zoho-mail",
            MailProvider::Outlook => "outlook",
            MailProvider::Other(name) => name,
        }
    }
}

/// Metadata about a mail message for spam evaluation
#[derive(Debug, Clone)]
pub struct MailMetadata {
    /// The mail provider that delivered this message
    pub provider: MailProvider,
    /// Provider-assigned labels (e.g., Gmail's SPAM, TRASH, PROMOTIONS)
    pub labels: Vec<String>,
    /// Message subject line
    pub subject: Option<String>,
    /// Message headers for heuristic analysis
    pub headers: HashMap<String, String>,
    /// Message sender address
    pub from: Option<String>,
    /// Message recipient addresses
    pub to: Vec<String>,
    /// Whether the message has attachments
    pub has_attachments: bool,
    /// Attachment file extensions if present
    pub attachment_extensions: Vec<String>,
}

/// Spam verdict for a mail message
#[derive(Debug, Clone)]
pub struct MailSpamVerdict {
    /// Whether the message is considered spam
    pub is_spam: bool,
    /// Spam confidence score (0.0 = definitely not spam, 1.0 = definitely spam)
    pub score: f32,
    /// Human-readable reason for the verdict
    pub reason: String,
}

impl MailSpamVerdict {
    /// Create a non-spam verdict
    pub fn not_spam(reason: impl Into<String>) -> Self {
        Self {
            is_spam: false,
            score: 0.0,
            reason: reason.into(),
        }
    }

    /// Create a spam verdict with a score
    pub fn spam(score: f32, reason: impl Into<String>) -> Self {
        Self {
            is_spam: true,
            score,
            reason: reason.into(),
        }
    }

    /// Create a spam verdict with maximum confidence
    pub fn definite_spam(reason: impl Into<String>) -> Self {
        Self {
            is_spam: true,
            score: 1.0,
            reason: reason.into(),
        }
    }
}

/// Trait for mail spam filtering implementations
pub trait MailSpamFilter: Send + Sync {
    /// Evaluate a mail message and return a spam verdict
    ///
    /// # Arguments
    /// * `meta` - Metadata about the mail message to evaluate
    ///
    /// # Returns
    /// A `MailSpamVerdict` indicating whether the message is spam and why
    fn evaluate(&self, meta: &MailMetadata) -> MailSpamVerdict;
}

/// Configuration for mail spam filtering (internal module config)
#[derive(Debug, Clone)]
pub struct MailSpamRuntimeConfig {
    /// Spam threshold (0.0 to 1.0). Messages scoring >= threshold are considered spam
    pub threshold: f32,
    /// Domains and email addresses that are always allowed (whitelist)
    pub allowlist: Vec<String>,
    /// Domains and email addresses that are always blocked (blacklist)
    pub denylist: Vec<String>,
}

impl Default for MailSpamRuntimeConfig {
    fn default() -> Self {
        Self {
            threshold: 0.8,
            allowlist: Vec::new(),
            denylist: Vec::new(),
        }
    }
}

impl MailSpamRuntimeConfig {
    /// Create a new config with the specified threshold
    pub fn new(threshold: f32) -> Self {
        Self {
            threshold: threshold.clamp(0.0, 1.0),
            allowlist: Vec::new(),
            denylist: Vec::new(),
        }
    }

    /// Add entries to the allowlist
    pub fn with_allowlist(mut self, entries: Vec<String>) -> Self {
        self.allowlist = entries;
        self
    }

    /// Add entries to the denylist
    pub fn with_denylist(mut self, entries: Vec<String>) -> Self {
        self.denylist = entries;
        self
    }

    /// Check if an email address or domain is in the allowlist
    pub fn is_allowed(&self, email: &str) -> bool {
        let email_lower = email.to_lowercase();
        self.allowlist.iter().any(|entry| {
            let entry_lower = entry.to_lowercase();
            // Exact match or domain match (if entry starts with @)
            email_lower == entry_lower
                || (entry_lower.starts_with('@') && email_lower.ends_with(&entry_lower))
                || entry_lower.starts_with('@')
                    && email_lower
                        .split('@')
                        .nth(1)
                        .map(|domain| format!("@{}", domain) == entry_lower)
                        .unwrap_or(false)
        })
    }

    /// Check if an email address or domain is in the denylist
    pub fn is_denied(&self, email: &str) -> bool {
        let email_lower = email.to_lowercase();
        self.denylist.iter().any(|entry| {
            let entry_lower = entry.to_lowercase();
            // Exact match or domain match (if entry starts with @)
            email_lower == entry_lower
                || (entry_lower.starts_with('@') && email_lower.ends_with(&entry_lower))
                || entry_lower.starts_with('@')
                    && email_lower
                        .split('@')
                        .nth(1)
                        .map(|domain| format!("@{}", domain) == entry_lower)
                        .unwrap_or(false)
        })
    }
}

/// Integration helpers for mail connectors
pub mod integration {
    use super::*;
    use std::sync::Arc;

    /// Create a mail spam filter from application configuration
    pub fn create_spam_filter_from_config(
        config: &crate::config::MailSpamConfig,
    ) -> Arc<dyn MailSpamFilter> {
        Arc::new(default::DefaultMailSpamFilter::new(MailSpamRuntimeConfig {
            threshold: config.threshold,
            allowlist: config.allowlist.clone(),
            denylist: config.denylist.clone(),
        }))
    }

    pub struct MailMetadataParams {
        pub message_id: String,
        pub labels: Vec<String>,
        pub subject: Option<String>,
        pub from: Option<String>,
        pub to: Vec<String>,
        pub headers: std::collections::HashMap<String, String>,
        pub has_attachments: bool,
        pub attachment_extensions: Vec<String>,
    }

    /// Helper function to create MailMetadata from Zoho Mail message data
    pub fn create_zoho_mail_metadata(params: MailMetadataParams) -> MailMetadata {
        MailMetadata {
            provider: MailProvider::ZohoMail,
            labels: params.labels,
            subject: params.subject,
            headers: params.headers,
            from: params.from,
            to: params.to,
            has_attachments: params.has_attachments,
            attachment_extensions: params.attachment_extensions,
        }
    }

    /// Helper function to create MailMetadata from Outlook message data
    pub fn create_outlook_metadata(params: MailMetadataParams) -> MailMetadata {
        MailMetadata {
            provider: MailProvider::Outlook,
            labels: params.labels,
            subject: params.subject,
            headers: params.headers,
            from: params.from,
            to: params.to,
            has_attachments: params.has_attachments,
            attachment_extensions: params.attachment_extensions,
        }
    }

    /// Common spam filtering logic that can be reused across mail connectors
    pub fn should_create_signal(
        spam_filter: &Arc<dyn MailSpamFilter>,
        metadata: &MailMetadata,
        provider_name: &str,
        connection_id: uuid::Uuid,
        message_id: &str,
    ) -> bool {
        let verdict = spam_filter.evaluate(metadata);

        // Log detailed telemetry for spam decisions
        if verdict.is_spam {
            tracing::info!(
                provider = provider_name,
                connection_id = %connection_id,
                message_id = %message_id,
                spam_score = verdict.score,
                spam_reason = %verdict.reason,
                "Message rejected as spam"
            );
            false
        } else {
            tracing::debug!(
                provider = provider_name,
                connection_id = %connection_id,
                message_id = %message_id,
                spam_score = verdict.score,
                "Message passed spam filter"
            );
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mail_provider_from_slug() {
        assert_eq!(MailProvider::from_slug("gmail"), MailProvider::Gmail);
        assert_eq!(MailProvider::from_slug("GMAIL"), MailProvider::Gmail);
        assert_eq!(MailProvider::from_slug("zoho-mail"), MailProvider::ZohoMail);
        assert_eq!(MailProvider::from_slug("outlook"), MailProvider::Outlook);
        assert_eq!(
            MailProvider::from_slug("custom-provider"),
            MailProvider::Other("custom-provider".to_string())
        );
    }

    #[test]
    fn test_mail_spam_verdict_creation() {
        let not_spam = MailSpamVerdict::not_spam("clean message");
        assert!(!not_spam.is_spam);
        assert_eq!(not_spam.score, 0.0);
        assert_eq!(not_spam.reason, "clean message");

        let spam = MailSpamVerdict::spam(0.9, "suspicious content");
        assert!(spam.is_spam);
        assert_eq!(spam.score, 0.9);
        assert_eq!(spam.reason, "suspicious content");

        let definite = MailSpamVerdict::definite_spam("known spam pattern");
        assert!(definite.is_spam);
        assert_eq!(definite.score, 1.0);
        assert_eq!(definite.reason, "known spam pattern");
    }

    #[test]
    fn test_mail_spam_runtime_config() {
        let config = MailSpamRuntimeConfig::new(0.7);
        assert_eq!(config.threshold, 0.7);

        let config_with_lists = config
            .with_allowlist(vec![
                "@example.com".to_string(),
                "trusted@sender.com".to_string(),
            ])
            .with_denylist(vec!["@spam.com".to_string()]);

        assert!(config_with_lists.is_allowed("user@example.com"));
        assert!(config_with_lists.is_allowed("trusted@sender.com"));
        assert!(!config_with_lists.is_allowed("user@other.com"));

        assert!(config_with_lists.is_denied("user@spam.com"));
        assert!(!config_with_lists.is_denied("user@good.com"));
    }
}
