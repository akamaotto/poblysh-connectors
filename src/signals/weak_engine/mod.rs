//! # Weak Signal Engine
//!
//! A background service that processes normalized signals, applies scoring models,
//! and promotes high-confidence candidates to grounded signals with recommendations.

use crate::error::RepositoryError;
use crate::models::signal::Model as Signal;
use crate::models::{GroundedSignalResponse, ScoringWeights, SignalScores};
use crate::repositories::{
    GroundedSignalRepository, SignalRepository, TenantSignalConfigRepository,
};
use sea_orm::DatabaseConnection;
use sha2::{Digest, Sha256};
use std::cmp::Ordering;
use std::collections::HashSet;
use std::sync::Arc;
use tracing::{debug, error, info};
use uuid::Uuid;

mod notifier;
mod scorer;

#[cfg(test)]
mod tests;

pub use notifier::Notifier;
pub use scorer::{SignalScorer, TFIDFVectorizer};

#[derive(Clone)]
struct ClusterSignal<'a> {
    signal: &'a Signal,
    content: String,
    vector: Vec<f32>,
    occurred_at: chrono::DateTime<chrono::Utc>,
}

struct SignalCluster<'a> {
    tenant_id: Uuid,
    first_occurred: chrono::DateTime<chrono::Utc>,
    centroid: Vec<f32>,
    signals: Vec<ClusterSignal<'a>>,
}

impl<'a> SignalCluster<'a> {
    fn new(initial: ClusterSignal<'a>) -> Self {
        let centroid = initial.vector.clone();
        let first_occurred = initial.occurred_at;
        let tenant_id = initial.signal.tenant_id;
        Self {
            tenant_id,
            first_occurred,
            centroid,
            signals: vec![initial],
        }
    }
    fn add_signal(&mut self, cluster_signal: ClusterSignal<'a>) {
        let len = self.signals.len() as f32;
        for (idx, value) in self.centroid.iter_mut().enumerate() {
            let candidate = cluster_signal.vector.get(idx).copied().unwrap_or(0.0);
            *value = ((*value * len) + candidate) / (len + 1.0);
        }
        if cluster_signal.occurred_at < self.first_occurred {
            self.first_occurred = cluster_signal.occurred_at;
        }
        self.signals.push(cluster_signal);
    }
}

/// Configuration for the weak signal engine
#[derive(Debug, Clone)]
pub struct WeakSignalEngineConfig {
    /// Default threshold for promoting signals to grounded signals
    pub default_threshold: f32,
    /// Batch size for processing signals
    pub batch_size: i64,
    /// Maximum age of signals to consider (in hours)
    pub max_signal_age_hours: i64,
    /// Maximum hours that signals can be apart and still belong to the same cluster
    pub cluster_window_hours: i64,
    /// Minimum cosine similarity for signals to join the same cluster
    pub cluster_similarity_threshold: f32,
    /// Whether to enable notification webhook
    pub enable_notifications: bool,
    /// Webhook timeout in seconds
    pub webhook_timeout_seconds: u64,
}

impl Default for WeakSignalEngineConfig {
    fn default() -> Self {
        Self {
            default_threshold: 0.7,
            batch_size: 100,
            max_signal_age_hours: 24,
            cluster_window_hours: 6,
            cluster_similarity_threshold: 0.8,
            enable_notifications: true,
            webhook_timeout_seconds: 10,
        }
    }
}

/// Weak Signal Engine that processes signals and creates grounded signals
pub struct WeakSignalEngine {
    db: Arc<DatabaseConnection>,
    scorer: SignalScorer,
    notifier: Notifier,
    config: WeakSignalEngineConfig,
    vectorizer: TFIDFVectorizer,
}

impl WeakSignalEngine {
    /// Create a new weak signal engine instance
    pub fn new(db: Arc<DatabaseConnection>, config: WeakSignalEngineConfig) -> Self {
        let scorer = SignalScorer::new();
        let notifier = Notifier::new(config.clone());
        let vectorizer = TFIDFVectorizer::new();

        Self {
            db,
            scorer,
            notifier,
            config,
            vectorizer,
        }
    }

    /// Process new signals and create grounded signals for those that meet thresholds
    pub async fn process_signals(&self) -> Result<(), RepositoryError> {
        info!("Starting weak signal processing cycle");

        // Get recent signals that haven't been processed yet
        let cutoff_time = (chrono::Utc::now()
            - chrono::Duration::hours(self.config.max_signal_age_hours))
        .naive_utc();

        // This is a simplified approach - in production you'd want to track which signals
        // have been processed to avoid reprocessing
        let signal_repo = SignalRepository::new(&*self.db);
        let recent_signals = signal_repo
            .list_signals(
                Uuid::default(), // Will be filtered by tenants later
                None,
                None,
                None,
                Some(chrono::DateTime::from_naive_utc_and_offset(
                    cutoff_time,
                    chrono::Utc,
                )),
                None,
                None,
                self.config.batch_size,
                true, // Include payloads for processing
            )
            .await?;

        if recent_signals.is_empty() {
            debug!("No recent signals to process");
            return Ok(());
        }

        info!("Processing {} recent signals", recent_signals.len());

        // Group signals by tenant for batch processing
        let mut tenant_signals: std::collections::HashMap<Uuid, Vec<&Signal>> =
            std::collections::HashMap::new();

        for signal in &recent_signals {
            tenant_signals
                .entry(signal.tenant_id)
                .or_default()
                .push(signal);
        }

        // Process each tenant's signals
        for (tenant_id, signals) in tenant_signals {
            if let Err(e) = self.process_tenant_signals(tenant_id, &signals).await {
                error!("Failed to process signals for tenant {}: {}", tenant_id, e);
            }
        }

        info!("Completed weak signal processing cycle");
        Ok(())
    }

    /// Process signals for a specific tenant
    async fn process_tenant_signals(
        &self,
        tenant_id: Uuid,
        signals: &[&Signal],
    ) -> Result<(), RepositoryError> {
        debug!(
            "Processing {} signals for tenant {}",
            signals.len(),
            tenant_id
        );

        // Create repositories
        let tenant_config_repo = TenantSignalConfigRepository::new(&*self.db);
        let grounded_signal_repo = GroundedSignalRepository::new(&*self.db);

        // Get tenant configuration
        let threshold = tenant_config_repo
            .get_threshold(tenant_id)
            .await
            .unwrap_or(self.config.default_threshold);

        let scoring_weights = tenant_config_repo
            .get_scoring_weights(tenant_id)
            .await
            .unwrap_or_default();

        // Check for webhook configuration
        let webhook_url = tenant_config_repo
            .get_webhook_url(tenant_id)
            .await
            .ok()
            .flatten();

        let clusters = self.cluster_signals(&signals);

        for cluster in clusters {
            let grounded_signal = self
                .process_signal_cluster(
                    &grounded_signal_repo,
                    &cluster,
                    &scoring_weights,
                    threshold,
                )
                .await?;

            if let Some(gs) = grounded_signal {
                info!(
                    "Created grounded signal {} for tenant {} (cluster size {})",
                    gs.id,
                    cluster.tenant_id,
                    cluster.signals.len()
                );

                if self.config.enable_notifications {
                    if let Some(ref url) = webhook_url {
                        let webhook_url_str: &str = url.as_str();
                        let grounded_signal_ref: &GroundedSignalResponse = &gs;
                        if let Err(e) = self
                            .notifier
                            .send_notification(webhook_url_str, grounded_signal_ref)
                            .await
                        {
                            error!(
                                "Failed to send notification for grounded signal {}: {}",
                                gs.id, e
                            );
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn process_signal_cluster(
        &self,
        grounded_signal_repo: &GroundedSignalRepository<'_>,
        cluster: &SignalCluster<'_>,
        weights: &ScoringWeights,
        threshold: f32,
    ) -> Result<Option<GroundedSignalResponse>, RepositoryError> {
        let mut scored_signals = Vec::with_capacity(cluster.signals.len());
        for entry in &cluster.signals {
            let scores = self
                .scorer
                .score_signal(entry.signal, &entry.content, weights)
                .await?;
            scored_signals.push((entry, scores));
        }

        let best = scored_signals
            .iter()
            .max_by(|(_, a), (_, b)| a.total.partial_cmp(&b.total).unwrap_or(Ordering::Equal));

        let Some((best_signal, best_scores)) = best else {
            return Ok(None);
        };

        if best_scores.total < threshold {
            return Ok(None);
        }

        let evidence = self.create_evidence(best_signal.signal, best_scores, cluster);
        let recommendation = self.generate_recommendation(best_signal.signal, best_scores);
        let idempotency_key = self.compute_cluster_idempotency(cluster.tenant_id, cluster);

        let grounded_signal = grounded_signal_repo
            .create(
                best_signal.signal.id,
                best_signal.signal.tenant_id,
                best_scores,
                crate::models::GroundedSignalStatus::Recommended,
                evidence,
                recommendation,
                Some(idempotency_key),
            )
            .await?;

        Ok(Some(grounded_signal))
    }

    fn cluster_signals<'signal>(&self, signals: &[&'signal Signal]) -> Vec<SignalCluster<'signal>> {
        let mut clusters: Vec<SignalCluster<'signal>> = Vec::new();

        for signal in signals {
            let cluster_signal = self.build_cluster_signal(signal);
            let mut placed = false;
            for existing in clusters.iter_mut() {
                if existing.tenant_id != cluster_signal.signal.tenant_id {
                    continue;
                }

                if !self.within_cluster_window(existing.first_occurred, cluster_signal.occurred_at)
                {
                    continue;
                }

                let similarity = self
                    .vectorizer
                    .cosine_similarity(&cluster_signal.vector, &existing.centroid);
                if similarity >= self.config.cluster_similarity_threshold {
                    existing.add_signal(cluster_signal.clone());
                    placed = true;
                    break;
                }
            }

            if !placed {
                clusters.push(SignalCluster::new(cluster_signal));
            }
        }

        clusters
    }

    fn build_cluster_signal<'signal>(&self, signal: &'signal Signal) -> ClusterSignal<'signal> {
        let content = self.extract_signal_content(signal);
        let vector = self.vectorizer.vectorize(&content);
        let occurred_at = chrono::DateTime::from_naive_utc_and_offset(
            signal.occurred_at.naive_utc(),
            chrono::Utc,
        );
        ClusterSignal {
            signal,
            content,
            vector,
            occurred_at,
        }
    }

    fn within_cluster_window(
        &self,
        first: chrono::DateTime<chrono::Utc>,
        candidate: chrono::DateTime<chrono::Utc>,
    ) -> bool {
        let delta = candidate - first;
        delta.num_hours().abs() <= self.config.cluster_window_hours
    }

    fn compute_cluster_idempotency(&self, tenant_id: Uuid, cluster: &SignalCluster<'_>) -> String {
        let mut signal_ids: Vec<String> = cluster
            .signals
            .iter()
            .map(|entry| entry.signal.id.to_string())
            .collect();
        signal_ids.sort();

        let bucket = cluster.first_occurred.timestamp() / 3600; // hour-level bucket
        let mut hasher = Sha256::new();
        hasher.update(tenant_id.as_bytes());
        hasher.update(bucket.to_be_bytes());
        for id in signal_ids {
            hasher.update(id.as_bytes());
        }

        format!("{:x}", hasher.finalize())
    }

    /// Extract relevant content from signal for scoring
    fn extract_signal_content(&self, signal: &Signal) -> String {
        // Extract text content from signal payload
        let payload = &signal.payload;

        // Try to get title, description, or content fields
        let content_fields = [
            "title",
            "description",
            "content",
            "body",
            "message",
            "text",
            "summary",
        ];

        let mut content_parts = Vec::new();

        for field in &content_fields {
            if let Some(value) = payload.get(field) {
                if let Some(text) = value.as_str() {
                    content_parts.push(text);
                }
            }
        }

        // Also include the signal kind and provider slug for context
        content_parts.push(&signal.kind);
        content_parts.push(&signal.provider_slug);

        // Join all content
        content_parts.join(" ")
    }

    /// Create evidence object for grounded signal
    fn create_evidence(
        &self,
        signal: &Signal,
        scores: &SignalScores,
        cluster: &SignalCluster,
    ) -> serde_json::Value {
        let keywords = self.aggregate_keywords(cluster);
        let sources: Vec<String> = cluster
            .signals
            .iter()
            .map(|entry| entry.signal.provider_slug.clone())
            .collect();
        let related_signals: Vec<serde_json::Value> = cluster
            .signals
            .iter()
            .map(|entry| {
                serde_json::json!({
                    "id": entry.signal.id,
                    "provider": entry.signal.provider_slug,
                    "kind": entry.signal.kind,
                })
            })
            .collect();

        serde_json::json!({
            "source_signal": {
                "id": signal.id,
                "kind": signal.kind,
                "provider": signal.provider_slug,
                "occurred_at": signal.occurred_at,
            },
            "score_breakdown": {
                "relevance": scores.relevance,
                "novelty": scores.novelty,
                "timeliness": scores.timeliness,
                "impact": scores.impact,
                "alignment": scores.alignment,
                "credibility": scores.credibility,
                "total": scores.total,
            },
            "keywords": keywords,
            "related_entities": self.extract_entities(signal),
            "related_signals": related_signals,
            "sources": sources,
            "cluster_size": cluster.signals.len(),
        })
    }

    fn aggregate_keywords(&self, cluster: &SignalCluster<'_>) -> Vec<String> {
        let mut keywords = HashSet::new();
        for entry in &cluster.signals {
            for keyword in self.extract_keywords(entry.signal) {
                keywords.insert(keyword);
            }
        }
        let mut collected: Vec<String> = keywords.into_iter().collect();
        collected.sort();
        collected.truncate(10);
        collected
    }

    /// Extract keywords from signal payload
    fn extract_keywords(&self, signal: &Signal) -> Vec<String> {
        let payload = &signal.payload;
        let mut keywords = Vec::new();

        // Look for common keyword fields
        if let Some(tags) = payload.get("tags").and_then(|v| v.as_array()) {
            for tag in tags {
                if let Some(tag_str) = tag.as_str() {
                    keywords.push(tag_str.to_string());
                }
            }
        }

        if let Some(labels) = payload.get("labels").and_then(|v| v.as_array()) {
            for label in labels {
                if let Some(label_str) = label.as_str() {
                    keywords.push(label_str.to_string());
                }
            }
        }

        // Simple keyword extraction from title/description
        for field in ["title", "description", "content"] {
            if let Some(text) = payload.get(field).and_then(|v| v.as_str()) {
                // Simple keyword extraction - split on spaces and filter common words
                let words: Vec<String> = text
                    .split_whitespace()
                    .map(|w| w.to_lowercase())
                    .filter(|w| w.len() > 3 && !is_common_word(w))
                    .take(5) // Limit to top 5 keywords
                    .collect();
                keywords.extend(words);
            }
        }

        keywords
    }

    /// Extract entities from signal payload
    fn extract_entities(&self, signal: &Signal) -> Vec<serde_json::Value> {
        let payload = &signal.payload;
        let mut entities = Vec::new();

        // Extract user/author information
        if let Some(user) = payload.get("user").or_else(|| payload.get("author")) {
            entities.push(serde_json::json!({
                "type": "person",
                "data": user
            }));
        }

        // Extract repository/project information
        if let Some(repo) = payload.get("repository").or_else(|| payload.get("project")) {
            entities.push(serde_json::json!({
                "type": "repository",
                "data": repo
            }));
        }

        // Extract organization information
        if let Some(org) = payload.get("organization") {
            entities.push(serde_json::json!({
                "type": "organization",
                "data": org
            }));
        }

        entities
    }

    /// Generate recommendation based on signal content and scores
    fn generate_recommendation(&self, signal: &Signal, scores: &SignalScores) -> Option<String> {
        let content = self.extract_signal_content(signal).to_lowercase();

        // Generate recommendation based on signal type and high-scoring dimensions
        let recommendation = if scores.impact > 0.8 && content.contains("security") {
            Some(
                "URGENT: Potential security issue detected. Immediate investigation recommended."
                    .to_string(),
            )
        } else if scores.timeliness > 0.8 && content.contains("outage") {
            Some("Service disruption detected. Coordinate with incident response team.".to_string())
        } else if scores.alignment > 0.8 && content.contains("feature") {
            Some(
                "New feature opportunity aligned with strategic goals. Consider prioritization."
                    .to_string(),
            )
        } else if scores.novelty > 0.8 {
            Some(
                "Novel activity detected. May indicate emerging trend or market shift.".to_string(),
            )
        } else if scores.impact > 0.7 && scores.credibility > 0.7 {
            Some(
                "High-impact signal from credible source. Recommended for PR team review."
                    .to_string(),
            )
        } else if scores.total > 0.8 {
            Some("High-confidence signal detected. Further investigation recommended.".to_string())
        } else {
            None // Don't provide recommendation for borderline cases
        };

        recommendation
    }
}

/// Check if a word is too common to be useful as a keyword
fn is_common_word(word: &str) -> bool {
    matches!(
        word,
        "the"
            | "and"
            | "or"
            | "but"
            | "in"
            | "on"
            | "at"
            | "to"
            | "for"
            | "of"
            | "with"
            | "by"
            | "this"
            | "that"
            | "these"
            | "those"
            | "from"
            | "they"
            | "them"
            | "their"
            | "have"
            | "has"
            | "had"
            | "been"
            | "was"
            | "were"
            | "are"
            | "is"
            | "will"
            | "would"
            | "could"
            | "should"
            | "may"
            | "might"
            | "can"
            | "shall"
    )
}
