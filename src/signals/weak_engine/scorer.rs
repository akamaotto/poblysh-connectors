//! # Signal Scorer
//!
//! Implements the six-dimensional scoring model for evaluating signals and promoting
//! them to grounded signals.

use crate::models::signal::Model as Signal;
use crate::models::{ScoringWeights, SignalScores};

/// Signal scorer that applies the six-dimensional scoring model
pub struct SignalScorer {}

impl Default for SignalScorer {
    fn default() -> Self {
        Self::new()
    }
}

impl SignalScorer {
    /// Create a new signal scorer
    pub fn new() -> Self {
        Self {}
    }

    /// Score a signal using the six-dimensional model
    pub async fn score_signal(
        &self,
        signal: &Signal,
        content: &str,
        weights: &ScoringWeights,
    ) -> Result<SignalScores, crate::error::RepositoryError> {
        // Calculate individual dimension scores
        let relevance = self.calculate_relevance(signal, content).await;
        let novelty = self.calculate_novelty(signal, content).await;
        let timeliness = self.calculate_timeliness(signal);
        let impact = self.calculate_impact(signal, content).await;
        let alignment = self.calculate_alignment(signal, content).await;
        let credibility = self.calculate_credibility(signal).await;

        // Calculate weighted total score
        let total = weights.impact * impact
            + weights.relevance * relevance
            + weights.novelty * novelty
            + weights.alignment * alignment
            + weights.timeliness * timeliness
            + weights.credibility * credibility;

        Ok(SignalScores {
            relevance,
            novelty,
            timeliness,
            impact,
            alignment,
            credibility,
            total,
        })
    }

    /// Calculate relevance score - how relevant the signal is to current business context
    async fn calculate_relevance(&self, signal: &Signal, content: &str) -> f32 {
        let mut score: f32 = 0.5; // Base score

        // Boost for specific keywords
        let relevant_keywords = [
            "security",
            "vulnerability",
            "outage",
            "incident",
            "feature",
            "release",
            "launch",
            "partnership",
            "acquisition",
            "merger",
            "investment",
            "funding",
            "customer",
            "revenue",
        ];

        let content_lower = content.to_lowercase();
        let keyword_matches = relevant_keywords
            .iter()
            .filter(|&keyword| content_lower.contains(keyword))
            .count();

        score += (keyword_matches as f32 * 0.1).min(0.3);

        // Boost for high-priority tags
        if let Some(tags) = signal.payload.get("tags").and_then(|v| v.as_array()) {
            for tag in tags {
                if let Some(tag_str) = tag.as_str() {
                    match tag_str {
                        "security" | "critical" | "urgent" => score += 0.2,
                        "feature" | "enhancement" => score += 0.1,
                        "bug" | "issue" => score += 0.05,
                        _ => {}
                    }
                }
            }
        }

        score.min(1.0)
    }

    /// Calculate novelty score - how novel or unexpected the signal is
    async fn calculate_novelty(&self, signal: &Signal, content: &str) -> f32 {
        let mut score = 0.3; // Base score (most signals have some novelty)

        // Higher novelty for unusual signal types
        match signal.kind.as_str() {
            "security_alert" | "outage" => score += 0.4,
            "new_integration" | "partnership" => score += 0.3,
            "feature_launch" | "release" => score += 0.2,
            "regular_activity" | "update" => score += 0.0,
            _ => score += 0.1,
        }

        // Higher novelty for signals from unexpected sources
        match signal.provider_slug.as_str() {
            "github" | "gitlab" => score += 0.0, // Common, lower novelty
            "jira" | "asana" => score += 0.1,
            "slack" | "teams" => score += 0.1,
            "gmail" | "outlook" => score += 0.2,
            _ => score += 0.15, // Less common providers
        }

        // Content-based novelty
        let content_lower = content.to_lowercase();
        let novelty_indicators = [
            "first",
            "new",
            "unexpected",
            "surprising",
            "unprecedented",
            "breakthrough",
            "innovative",
            "pioneering",
            "novel",
            "original",
        ];

        let novelty_matches = novelty_indicators
            .iter()
            .filter(|&indicator| content_lower.contains(indicator))
            .count();

        score += (novelty_matches as f32 * 0.05).min(0.2);

        score.min(1.0)
    }

    /// Calculate timeliness score - how timely the signal is
    fn calculate_timeliness(&self, signal: &Signal) -> f32 {
        let now = chrono::Utc::now();
        let signal_occurred: chrono::DateTime<chrono::Utc> =
            chrono::DateTime::from_naive_utc_and_offset(
                signal.occurred_at.naive_utc(),
                chrono::Utc,
            );
        let signal_age = now.signed_duration_since(signal_occurred);

        // More recent signals get higher scores
        if signal_age.num_hours() < 1 {
            1.0 // Very recent
        } else if signal_age.num_hours() < 6 {
            0.9
        } else if signal_age.num_hours() < 24 {
            0.8
        } else if signal_age.num_hours() < 72 {
            0.6
        } else if signal_age.num_hours() < 168 {
            0.4 // 1 week
        } else {
            0.2 // Older than 1 week
        }
    }

    /// Calculate impact score - potential impact of the signal
    async fn calculate_impact(&self, signal: &Signal, content: &str) -> f32 {
        let mut score = 0.2; // Base score

        // High impact indicators
        let content_lower = content.to_lowercase();
        let high_impact_keywords = [
            "critical",
            "severe",
            "major",
            "significant",
            "massive",
            "huge",
            "important",
            "urgent",
            "emergency",
            "outage",
            "downtime",
            "security",
            "breach",
            "vulnerability",
        ];

        let high_impact_matches = high_impact_keywords
            .iter()
            .filter(|&keyword| content_lower.contains(keyword))
            .count();

        score += (high_impact_matches as f32 * 0.15).min(0.4);

        // Impact by signal type
        match signal.kind.as_str() {
            "security_alert" | "outage" => score += 0.4,
            "compliance_issue" | "legal" => score += 0.3,
            "feature_launch" | "release" => score += 0.2,
            "partnership" | "acquisition" => score += 0.35,
            "customer_issue" | "bug" => score += 0.25,
            _ => score += 0.1,
        }

        // Scale by audience size if available
        if let Some(audience) = signal.payload.get("audience_size").and_then(|v| v.as_u64()) {
            let audience_score = (audience as f32 / 10000.0).min(0.3); // Cap at 0.3
            score += audience_score;
        }

        score.min(1.0)
    }

    /// Calculate alignment score - alignment with strategic goals
    async fn calculate_alignment(&self, _signal: &Signal, content: &str) -> f32 {
        let mut score = 0.4; // Base score

        // Strategic alignment indicators
        let content_lower = content.to_lowercase();
        let alignment_keywords = [
            "strategic",
            "goal",
            "objective",
            "roadmap",
            "planned",
            "priority",
            "initiative",
            "transformation",
            "innovation",
            "growth",
            "expansion",
            "optimization",
            "efficiency",
        ];

        let alignment_matches = alignment_keywords
            .iter()
            .filter(|&keyword| content_lower.contains(keyword))
            .count();

        score += (alignment_matches as f32 * 0.08).min(0.3);

        // Alignment by department/team if mentioned
        if content_lower.contains("engineering") || content_lower.contains("development") {
            score += 0.1;
        }
        if content_lower.contains("product") {
            score += 0.1;
        }
        if content_lower.contains("marketing") || content_lower.contains("sales") {
            score += 0.1;
        }
        if content_lower.contains("security") {
            score += 0.15;
        }

        score.min(1.0)
    }

    /// Calculate credibility score - credibility of the signal source
    async fn calculate_credibility(&self, signal: &Signal) -> f32 {
        let mut score: f32 = 0.5; // Base score

        // Credibility by provider
        match signal.provider_slug.as_str() {
            "github" | "gitlab" => score += 0.3, // High credibility (code changes)
            "jira" | "asana" => score += 0.25,   // High credibility (official work tracking)
            "gmail" | "outlook" => score += 0.2, // Medium credibility (email)
            "slack" | "teams" => score += 0.15,  // Medium credibility (chat)
            _ => score += 0.1,
        }

        // Boost for official/automated sources
        if signal.kind.contains("bot") || signal.kind.contains("automated") {
            score += 0.1;
        }

        // Source credibility if available in payload
        if let Some(user) = signal.payload.get("user")
            && let Some(authority) = user.get("authority").and_then(|v| v.as_str())
        {
            match authority {
                "admin" | "owner" => score += 0.2,
                "maintainer" | "lead" => score += 0.15,
                "member" | "contributor" => score += 0.1,
                _ => {}
            }
        }

        score.min(1.0)
    }
}

/// TF-IDF vectorizer for text analysis
pub struct TFIDFVectorizer {
    // In a real implementation, this would maintain document frequency statistics
    // For now, we'll use a simplified approach
}

impl TFIDFVectorizer {
    /// Create a new TF-IDF vectorizer
    pub fn new() -> Self {
        Self {}
    }

    /// Vectorize text content using TF-IDF
    pub fn vectorize(&self, text: &str) -> Vec<f32> {
        // Simplified TF-IDF implementation
        // In a real system, this would use precomputed document frequencies
        // and proper TF-IDF calculations

        let words: Vec<String> = text
            .to_lowercase()
            .split_whitespace()
            .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
            .filter(|w| w.len() > 2)
            .collect();

        // Simple frequency-based vector (fixed size for demo)
        let mut vector = vec![0.0; 768]; // Standard BERT embedding size

        // Create a simple hash-based embedding
        for word in words.iter().take(100) {
            let hash = hash_string(word) % 768;
            vector[hash] += 1.0 / (words.len() as f32).sqrt();
        }

        vector
    }

    /// Calculate cosine similarity between two vectors
    pub fn cosine_similarity(&self, vec1: &[f32], vec2: &[f32]) -> f32 {
        if vec1.len() != vec2.len() {
            return 0.0;
        }

        let dot_product: f32 = vec1.iter().zip(vec2.iter()).map(|(a, b)| a * b).sum();
        let magnitude1: f32 = vec1.iter().map(|a| a * a).sum::<f32>().sqrt();
        let magnitude2: f32 = vec2.iter().map(|a| a * a).sum::<f32>().sqrt();

        if magnitude1 == 0.0 || magnitude2 == 0.0 {
            0.0
        } else {
            dot_product / (magnitude1 * magnitude2)
        }
    }
}

/// Simple string hash function
fn hash_string(s: &str) -> usize {
    let mut hash: usize = 0;
    for (i, c) in s.chars().enumerate() {
        hash = hash.wrapping_mul(31).wrapping_add(c as usize);
        hash = hash.wrapping_add(i);
    }
    hash
}

impl Default for TFIDFVectorizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;
    use crate::db::init_pool;
    use crate::models::connection::ActiveModel as ConnectionActiveModel;
    use crate::models::signal::ActiveModel as SignalActiveModel;
    use crate::models::tenant::ActiveModel as TenantActiveModel;
    use chrono::Utc;
    use sea_orm::ActiveModelTrait;
    use uuid::Uuid;

    async fn setup_test_signal() -> Signal {
        let config = AppConfig {
            profile: "test".to_string(),
            ..Default::default()
        };
        let db = init_pool(&config).await.expect("Failed to init test DB");

        // Create tenant
        let tenant_id = Uuid::new_v4();
        let tenant = TenantActiveModel {
            id: sea_orm::Set(tenant_id),
            ..Default::default()
        };
        tenant.insert(&db).await.unwrap();

        let connection_id = Uuid::new_v4();
        let connection = ConnectionActiveModel {
            id: sea_orm::Set(connection_id),
            tenant_id: sea_orm::Set(tenant_id),
            provider_slug: sea_orm::Set("github".to_string()),
            external_id: sea_orm::Set("test-connection".to_string()),
            status: sea_orm::Set("active".to_string()),
            created_at: sea_orm::Set(Utc::now().into()),
            updated_at: sea_orm::Set(Utc::now().into()),
            ..Default::default()
        };
        connection.insert(&db).await.unwrap();

        let signal_payload = serde_json::json!({
            "title": "Critical security vulnerability discovered",
            "description": "A severe security issue was found in the authentication system",
            "tags": ["security", "critical", "urgent"],
            "user": { "authority": "admin" },
            "audience_size": 50000
        });

        let signal = SignalActiveModel {
            id: sea_orm::Set(Uuid::new_v4()),
            tenant_id: sea_orm::Set(tenant_id),
            provider_slug: sea_orm::Set("github".to_string()),
            connection_id: sea_orm::Set(connection_id),
            kind: sea_orm::Set("security_alert".to_string()),
            occurred_at: sea_orm::Set(Utc::now().into()),
            received_at: sea_orm::Set(Utc::now().into()),
            payload: sea_orm::Set(signal_payload),
            ..Default::default()
        };

        signal.insert(&db).await.unwrap()
    }

    #[tokio::test]
    async fn test_signal_scoring() {
        let signal = setup_test_signal().await;
        let scorer = SignalScorer::new();
        let content = "Critical security vulnerability discovered in authentication system";

        let weights = ScoringWeights::default();
        let scores = scorer
            .score_signal(&signal, content, &weights)
            .await
            .unwrap();

        // Security alert should have high scores
        assert!(scores.impact > 0.7);
        assert!(scores.credibility > 0.7);
        assert!(scores.total > 0.5);

        // Verify total score calculation
        let expected_total = weights.impact * scores.impact
            + weights.relevance * scores.relevance
            + weights.novelty * scores.novelty
            + weights.alignment * scores.alignment
            + weights.timeliness * scores.timeliness
            + weights.credibility * scores.credibility;

        assert!((scores.total - expected_total).abs() < 0.001);
    }
}
