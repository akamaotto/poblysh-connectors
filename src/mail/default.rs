//! Default mail spam filter implementation
//!
//! Provides a rule-based spam filter that uses provider labels, keyword analysis,
//! and attachment heuristics to identify spam messages.

use std::collections::HashSet;

use crate::mail::{
    MailMetadata, MailProvider, MailSpamFilter, MailSpamRuntimeConfig, MailSpamVerdict,
};

/// Default implementation of mail spam filtering
///
/// This filter uses a combination of:
/// - Provider-assigned labels (SPAM, TRASH, etc.)
/// - Keyword heuristics for phishing and scam patterns
/// - Attachment analysis for suspicious file types
/// - Allowlist/denylist enforcement
#[derive(Debug, Clone)]
pub struct DefaultMailSpamFilter {
    config: MailSpamRuntimeConfig,
}

impl DefaultMailSpamFilter {
    /// Create a new filter with the given configuration
    pub fn new(config: MailSpamRuntimeConfig) -> Self {
        Self { config }
    }

    /// Check if provider labels indicate spam
    fn check_provider_labels(&self, meta: &MailMetadata) -> Option<MailSpamVerdict> {
        let labels: HashSet<String> = meta.labels.iter().map(|s| s.to_lowercase()).collect();

        // High-confidence spam labels
        let spam_labels = ["spam", "junk", "trash", "bulk"];
        for label in &spam_labels {
            if labels.contains(*label) {
                return Some(MailSpamVerdict::definite_spam(format!(
                    "Provider marked as spam (label: {label})"
                )));
            }
        }

        // Moderate-confidence spam indicators
        let suspicious_labels = ["promotions", "social", "updates", "forums"];
        for label in &suspicious_labels {
            if labels.contains(*label) {
                return Some(MailSpamVerdict::spam(
                    0.6,
                    format!("Suspicious provider label: {label}"),
                ));
            }
        }

        None
    }

    /// Check if sender is in allowlist or denylist
    fn check_sender_lists(&self, meta: &MailMetadata) -> Option<MailSpamVerdict> {
        if let Some(from) = &meta.from {
            // Check denylist first - highest priority
            if self.config.is_denied(from) {
                return Some(MailSpamVerdict::definite_spam(format!(
                    "Sender in denylist: {from}"
                )));
            }

            // Check allowlist - always allowed
            if self.config.is_allowed(from) {
                return Some(MailSpamVerdict::not_spam(format!(
                    "Sender in allowlist: {from}"
                )));
            }
        }

        None
    }

    /// Analyze subject for spam indicators
    fn analyze_subject(&self, subject: &str) -> f32 {
        let subject_lower = subject.to_lowercase();
        let mut score = 0.0;

        // Urgency indicators
        let urgency_words = [
            "urgent",
            "immediate",
            "action required",
            "verify now",
            "limited time",
            "expiring",
            "expires soon",
            "last chance",
            "don't miss",
            "act now",
        ];
        for word in &urgency_words {
            if subject_lower.contains(word) {
                score += 0.15;
            }
        }

        // Financial indicators
        let financial_words = [
            "congratulations",
            "winner",
            "lottery",
            "prize",
            "claim",
            "reward",
            "million",
            "thousand",
            "cash",
            "payment",
            "transfer",
            "inheritance",
        ];
        for word in &financial_words {
            if subject_lower.contains(word) {
                score += 0.2;
            }
        }

        // Phishing indicators
        let phishing_words = [
            "verify",
            "confirm",
            "update",
            "suspend",
            "locked",
            "compromised",
            "security",
            "alert",
            "unusual",
            "activity",
            "account",
            "click here",
        ];
        for word in &phishing_words {
            if subject_lower.contains(word) {
                score += 0.18;
            }
        }

        // All caps or excessive punctuation
        let len = subject.chars().count().max(1) as f32;
        let caps_ratio = subject.chars().filter(|c| c.is_uppercase()).count() as f32 / len;
        if caps_ratio > 0.5 {
            score += 0.25;
        }

        let exclamation_count = subject.matches('!').count();
        if exclamation_count > 2 {
            score += 0.1 * (exclamation_count as f32 - 2.0);
        }

        score.min(1.0)
    }

    /// Analyze attachments for suspicious patterns
    fn analyze_attachments(&self, meta: &MailMetadata) -> f32 {
        if !meta.has_attachments {
            return 0.0;
        }

        let mut score: f32 = 0.0;

        // Suspicious file extensions
        let suspicious_extensions = [
            "exe", "bat", "com", "pif", "scr", "vbs", "js", "jar", "app", "deb", "rpm", "dmg",
            "pkg", "msi", "msp", "reg", "inf", "sys", "dll",
        ];

        for ext in &meta.attachment_extensions {
            let ext_lower = ext.to_lowercase();
            if suspicious_extensions.contains(&ext_lower.as_str()) {
                score += 0.8;
            } else if ext_lower == "zip" || ext_lower == "rar" || ext_lower == "7z" {
                // Archives are moderately suspicious
                score += 0.3;
            }
        }

        // Many attachments can be suspicious
        if meta.attachment_extensions.len() > 3 {
            score += 0.2;
        }

        score.min(1.0)
    }

    /// Check headers for spam indicators
    fn analyze_headers(&self, meta: &MailMetadata) -> f32 {
        let mut score: f32 = 0.0;

        // Check for missing standard headers
        if !meta.headers.contains_key("received") {
            score += 0.3;
        }

        if !meta.headers.contains_key("date") {
            score += 0.2;
        }

        // Check authentication results
        if let Some(auth_results) = meta.headers.get("authentication-results")
            && auth_results.to_lowercase().contains("fail")
        {
            score += 0.5;
        }

        // Check for suspicious content-type
        if let Some(content_type) = meta.headers.get("content-type")
            && content_type.to_lowercase().contains("text/html")
            && meta.subject.is_none()
        {
            // HTML-only with no subject
            score += 0.1;
        }

        score.min(1.0)
    }

    /// Apply provider-specific heuristics
    fn apply_provider_heuristics(&self, meta: &MailMetadata, base_score: f32) -> f32 {
        match meta.provider {
            MailProvider::Gmail => {
                // Gmail has good built-in filtering, so we're more conservative
                base_score * 0.8
            }
            MailProvider::ZohoMail => {
                // Similar to Gmail, decent built-in filtering
                base_score * 0.85
            }
            MailProvider::Outlook => {
                // Outlook has good filtering too
                base_score * 0.8
            }
            MailProvider::Other(_) => {
                // Unknown providers get full weight
                base_score
            }
        }
    }
}

impl Default for DefaultMailSpamFilter {
    /// Create a filter with default configuration
    fn default() -> Self {
        Self::new(MailSpamRuntimeConfig::default())
    }
}

impl MailSpamFilter for DefaultMailSpamFilter {
    fn evaluate(&self, meta: &MailMetadata) -> MailSpamVerdict {
        // Check denylist/allowlist first (highest priority)
        if let Some(verdict) = self.check_sender_lists(meta) {
            return verdict;
        }

        // Check provider labels (high confidence)
        if let Some(verdict) = self.check_provider_labels(meta) {
            return verdict;
        }

        // Calculate spam score from various heuristics
        let mut score: f32 = 0.0;
        let mut reasons = Vec::new();

        // Subject analysis
        if let Some(subject) = &meta.subject {
            let subject_score = self.analyze_subject(subject);
            if subject_score > 0.2 {
                score += subject_score;
                reasons.push(format!("Subject analysis: {:.2}", subject_score));
            }
        }

        // Attachment analysis
        let attachment_score = self.analyze_attachments(meta);
        if attachment_score > 0.1 {
            score += attachment_score;
            reasons.push(format!("Attachment analysis: {:.2}", attachment_score));
        }

        // Header analysis
        let header_score = self.analyze_headers(meta);
        if header_score > 0.1 {
            score += header_score;
            reasons.push(format!("Header analysis: {:.2}", header_score));
        }

        // Apply provider-specific adjustments
        score = self.apply_provider_heuristics(meta, score);

        // Normalize score to [0, 1]
        score = score.min(1.0);

        // Make final decision
        if score >= self.config.threshold {
            let reason = if reasons.is_empty() {
                format!(
                    "Spam score {:.2} exceeds threshold {:.2}",
                    score, self.config.threshold
                )
            } else {
                format!(
                    "Spam score {:.2} (threshold {:.2}) - {}",
                    score,
                    self.config.threshold,
                    reasons.join(", ")
                )
            };
            MailSpamVerdict::spam(score, reason)
        } else {
            let reason = if reasons.is_empty() {
                "Message appears legitimate".to_string()
            } else {
                format!("Low spam score {:.2} - {}", score, reasons.join(", "))
            };
            MailSpamVerdict::not_spam(reason)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_metadata() -> MailMetadata {
        MailMetadata {
            provider: MailProvider::Gmail,
            labels: Vec::new(),
            subject: Some("Test Subject".to_string()),
            headers: HashMap::new(),
            from: Some("test@example.com".to_string()),
            to: vec!["recipient@example.com".to_string()],
            has_attachments: false,
            attachment_extensions: Vec::new(),
        }
    }

    #[test]
    fn test_provider_label_detection() {
        let filter = DefaultMailSpamFilter::default();

        // Test spam label
        let mut spam_meta = create_test_metadata();
        spam_meta.labels.push("SPAM".to_string());
        let verdict = filter.evaluate(&spam_meta);
        assert!(verdict.is_spam);
        assert_eq!(verdict.score, 1.0);
        assert!(verdict.reason.contains("Provider marked as spam"));

        // Test suspicious label
        let mut suspicious_meta = create_test_metadata();
        suspicious_meta.labels.push("PROMOTIONS".to_string());
        let verdict = filter.evaluate(&suspicious_meta);
        assert!(verdict.is_spam);
        assert!(verdict.score >= 0.6);
        assert!(verdict.reason.contains("Suspicious provider label"));
    }

    #[test]
    fn test_allowlist_denylist() {
        let config = MailSpamRuntimeConfig::new(0.8)
            .with_allowlist(vec!["@trusted.com".to_string()])
            .with_denylist(vec!["@spam.com".to_string()]);
        let filter = DefaultMailSpamFilter::new(config);

        // Test allowlist
        let mut allowlist_meta = create_test_metadata();
        allowlist_meta.from = Some("user@trusted.com".to_string());
        let verdict = filter.evaluate(&allowlist_meta);
        assert!(!verdict.is_spam);
        assert!(verdict.reason.contains("allowlist"));

        // Test denylist
        let mut denylist_meta = create_test_metadata();
        denylist_meta.from = Some("user@spam.com".to_string());
        let verdict = filter.evaluate(&denylist_meta);
        assert!(verdict.is_spam);
        assert_eq!(verdict.score, 1.0);
        assert!(verdict.reason.contains("denylist"));
    }

    #[test]
    fn test_subject_analysis() {
        let filter = DefaultMailSpamFilter::default();

        // Test urgent subject
        let mut urgent_meta = create_test_metadata();
        urgent_meta.subject = Some("URGENT: VERIFY YOUR ACCOUNT NOW!!!".to_string());
        let verdict = filter.evaluate(&urgent_meta);
        assert!(verdict.is_spam);
        assert!(verdict.reason.contains("Subject analysis"));

        // Test normal subject
        let mut normal_meta = create_test_metadata();
        normal_meta.subject = Some("Team meeting notes".to_string());
        let verdict = filter.evaluate(&normal_meta);
        assert!(!verdict.is_spam);
    }

    #[test]
    fn test_attachment_analysis() {
        let filter = DefaultMailSpamFilter::default();

        // Test suspicious attachment
        let mut attachment_meta = create_test_metadata();
        attachment_meta.has_attachments = true;
        attachment_meta
            .attachment_extensions
            .push("exe".to_string());
        let verdict = filter.evaluate(&attachment_meta);
        assert!(verdict.is_spam);
        assert!(verdict.reason.contains("Attachment analysis"));

        // Test normal attachment
        let mut normal_attachment_meta = create_test_metadata();
        normal_attachment_meta.has_attachments = true;
        normal_attachment_meta
            .attachment_extensions
            .push("pdf".to_string());
        let verdict = filter.evaluate(&normal_attachment_meta);
        assert!(!verdict.is_spam);
    }

    #[test]
    fn test_provider_specific_heuristics() {
        let filter = DefaultMailSpamFilter::default();

        // Same suspicious content across different providers
        let suspicious_content = "URGENT: Claim your prize now!!!";

        let mut gmail_meta = create_test_metadata();
        gmail_meta.provider = MailProvider::Gmail;
        gmail_meta.subject = Some(suspicious_content.to_string());
        let gmail_verdict = filter.evaluate(&gmail_meta);

        let mut unknown_meta = create_test_metadata();
        unknown_meta.provider = MailProvider::Other("unknown-provider".to_string());
        unknown_meta.subject = Some(suspicious_content.to_string());
        let unknown_verdict = filter.evaluate(&unknown_meta);

        // Unknown provider should get higher spam score
        assert!(unknown_verdict.score > gmail_verdict.score);
    }

    #[test]
    fn test_threshold_configuration() {
        // Low threshold - more aggressive filtering
        let low_threshold_config = MailSpamRuntimeConfig::new(0.3);
        let low_threshold_filter = DefaultMailSpamFilter::new(low_threshold_config);

        let mut meta = create_test_metadata();
        meta.subject = Some("Urgent action required".to_string());
        let verdict = low_threshold_filter.evaluate(&meta);
        assert!(verdict.is_spam);

        // High threshold - less aggressive filtering
        let high_threshold_config = MailSpamRuntimeConfig::new(0.9);
        let high_threshold_filter = DefaultMailSpamFilter::new(high_threshold_config);

        let verdict = high_threshold_filter.evaluate(&meta);
        assert!(!verdict.is_spam);
    }
}
