//! # Notification System
//!
//! Handles sending notifications when grounded signals are created.

use crate::models::GroundedSignalResponse;
use reqwest::Client;
use serde_json::json;
use std::time::Duration;
use tracing::{error, info, warn};
use url::Url;

use super::WeakSignalEngineConfig;

/// Notification system for sending grounded signal alerts
pub struct Notifier {
    client: Client,
}

impl Notifier {
    /// Create a new notifier
    pub fn new(config: WeakSignalEngineConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.webhook_timeout_seconds))
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }

    /// Validate webhook URL according to security and reliability constraints:
    /// - Must be HTTPS
    /// - Length must be reasonable (<= 2048 chars)
    fn validate_webhook_url(&self, webhook_url: &str) -> bool {
        if webhook_url.len() > 2048 {
            warn!(
                "Webhook URL exceeds maximum length: target={} length={}",
                self.redacted_target(webhook_url),
                webhook_url.len()
            );
            return false;
        }

        if !webhook_url.to_lowercase().starts_with("https://") {
            warn!(
                "Rejected non-HTTPS webhook URL: {}",
                self.redacted_target(webhook_url)
            );
            return false;
        }

        true
    }

    /// Send notification for a grounded signal
    pub async fn send_notification(
        &self,
        webhook_url: &str,
        grounded_signal: &GroundedSignalResponse,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !self.validate_webhook_url(webhook_url) {
            return Err("Invalid webhook URL: must be HTTPS and <= 2048 characters".into());
        }

        info!(
            "Sending notification for grounded signal {} to {}",
            grounded_signal.id,
            self.redacted_target(webhook_url)
        );

        let payload = self.build_webhook_payload(grounded_signal);

        // Implement retry logic with exponential backoff
        let max_retries = 3;
        let mut delay = Duration::from_secs(1);

        for attempt in 1..=max_retries {
            match self.client.post(webhook_url).json(&payload).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        info!(
                            "Successfully sent notification for grounded signal {} (attempt {})",
                            grounded_signal.id, attempt
                        );
                        return Ok(());
                    } else {
                        warn!(
                            "Webhook returned status {} for grounded signal {} (attempt {})",
                            response.status(),
                            grounded_signal.id,
                            attempt
                        );

                        if attempt == max_retries {
                            return Err(format!(
                                "Webhook failed after {} attempts with status {}",
                                max_retries,
                                response.status()
                            )
                            .into());
                        }
                    }
                }
                Err(e) => {
                    error!(
                        "Failed to send webhook for grounded signal {} (attempt {}): {}",
                        grounded_signal.id, attempt, e
                    );

                    if attempt == max_retries {
                        return Err(format!(
                            "Webhook failed after {} attempts: {}",
                            max_retries, e
                        )
                        .into());
                    }
                }
            }

            // Exponential backoff
            tokio::time::sleep(delay).await;
            delay *= 2;
        }

        Ok(())
    }

    fn redacted_target(&self, webhook_url: &str) -> String {
        Url::parse(webhook_url)
            .ok()
            .map(|parsed| {
                let scheme = parsed.scheme();
                let host = parsed.host_str().unwrap_or("unknown");
                format!("{}://{}", scheme, host)
            })
            .unwrap_or_else(|| "[invalid-url]".to_string())
    }

    /// Build webhook payload for grounded signal
    fn build_webhook_payload(&self, grounded_signal: &GroundedSignalResponse) -> serde_json::Value {
        json!({
            "grounded_signal_id": grounded_signal.id,
            "signal_id": grounded_signal.signal_id,
            "tenant_id": grounded_signal.tenant_id,
            "total_score": grounded_signal.scores.total,
            "scores": {
                "relevance": grounded_signal.scores.relevance,
                "novelty": grounded_signal.scores.novelty,
                "timeliness": grounded_signal.scores.timeliness,
                "impact": grounded_signal.scores.impact,
                "alignment": grounded_signal.scores.alignment,
                "credibility": grounded_signal.scores.credibility,
            },
            "status": grounded_signal.status,
            "recommendation": grounded_signal.recommendation,
            "evidence_summary": self.extract_evidence_summary(&grounded_signal.evidence),
            "signal_source": grounded_signal.evidence
                .get("source_signal")
                .and_then(|s| s.get("provider"))
                .and_then(|p| p.as_str())
                .unwrap_or("unknown"),
            "occurred_at": grounded_signal.evidence
                .get("source_signal")
                .and_then(|s| s.get("occurred_at")),
            "created_at": grounded_signal.created_at,
            "updated_at": grounded_signal.updated_at,
        })
    }

    /// Extract a summary of evidence for the webhook payload
    fn extract_evidence_summary(&self, evidence: &serde_json::Value) -> serde_json::Value {
        let source_signal = evidence
            .get("source_signal")
            .unwrap_or(&serde_json::Value::Null);
        let empty_vec = vec![];
        let keywords = evidence
            .get("keywords")
            .and_then(|k| k.as_array())
            .unwrap_or(&empty_vec);
        let related_entities = evidence
            .get("related_entities")
            .and_then(|e| e.as_array())
            .unwrap_or(&empty_vec);

        json!({
            "signal_kind": source_signal.get("kind"),
            "signal_provider": source_signal.get("provider"),
            "keywords": keywords,
            "related_entities": related_entities,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{GroundedSignalStatus, SignalScores};
    use chrono::Utc;
    use uuid::Uuid;

    fn create_test_grounded_signal() -> GroundedSignalResponse {
        let id = Uuid::new_v4();
        let signal_id = Uuid::new_v4();
        let tenant_id = Uuid::new_v4();
        let now = Utc::now();

        GroundedSignalResponse {
            id,
            signal_id,
            tenant_id,
            scores: SignalScores {
                relevance: 0.8,
                novelty: 0.6,
                timeliness: 0.9,
                impact: 0.7,
                alignment: 0.8,
                credibility: 0.75,
                total: 0.77,
            },
            status: GroundedSignalStatus::Recommended,
            evidence: serde_json::json!({
                "source_signal": {
                    "id": signal_id,
                    "kind": "security_alert",
                    "provider": "github",
                    "occurred_at": now,
                },
                "score_breakdown": {
                    "relevance": 0.8,
                    "novelty": 0.6,
                    "timeliness": 0.9,
                    "impact": 0.7,
                    "alignment": 0.8,
                    "credibility": 0.75,
                    "total": 0.77,
                },
                "keywords": ["security", "vulnerability", "critical"],
                "related_entities": [
                    {
                        "type": "person",
                        "data": {"name": "John Doe", "authority": "admin"}
                    }
                ],
            }),
            recommendation: Some(
                "URGENT: Security issue requires immediate investigation".to_string(),
            ),
            created_at: now.into(),
            updated_at: now.into(),
        }
    }

    #[test]
    fn test_build_webhook_payload() {
        let config = WeakSignalEngineConfig::default();
        let notifier = Notifier::new(config);
        let grounded_signal = create_test_grounded_signal();

        let payload = notifier.build_webhook_payload(&grounded_signal);

        assert_eq!(
            payload["grounded_signal_id"],
            grounded_signal.id.to_string()
        );
        assert_eq!(payload["signal_id"], grounded_signal.signal_id.to_string());
        assert_eq!(payload["tenant_id"], grounded_signal.tenant_id.to_string());
        assert_eq!(payload["total_score"], grounded_signal.scores.total);
        assert_eq!(payload["status"], "recommended");
        assert_eq!(
            payload["recommendation"],
            grounded_signal.recommendation.unwrap()
        );
        assert_eq!(payload["signal_source"], "github");

        // Check evidence summary
        let evidence_summary = &payload["evidence_summary"];
        assert_eq!(evidence_summary["signal_kind"], "security_alert");
        assert_eq!(evidence_summary["signal_provider"], "github");

        let keywords = evidence_summary["keywords"].as_array().unwrap();
        assert_eq!(keywords.len(), 3);
        assert!(keywords.contains(&serde_json::Value::String("security".to_string())));
    }

    #[test]
    fn test_extract_evidence_summary() {
        let config = WeakSignalEngineConfig::default();
        let notifier = Notifier::new(config);
        let grounded_signal = create_test_grounded_signal();

        let summary = notifier.extract_evidence_summary(&grounded_signal.evidence);

        assert_eq!(summary["signal_kind"], "security_alert");
        assert_eq!(summary["signal_provider"], "github");

        let keywords = summary["keywords"].as_array().unwrap();
        assert_eq!(keywords.len(), 3);
        assert!(keywords.contains(&serde_json::Value::String("security".to_string())));

        let related_entities = summary["related_entities"].as_array().unwrap();
        assert_eq!(related_entities.len(), 1);
        assert_eq!(related_entities[0]["type"], "person");
    }
}
