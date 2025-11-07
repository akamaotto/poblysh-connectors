## Context
- Gmail previously contained inline spam + weak-signal heuristics. We removed those to keep the connector focused on OAuth/webhooks/history.
- Future mail connectors (Zoho Mail, Outlook) will need the same logic. Implementing per connector would create inconsistencies and double maintenance.
- `plan/signals.md` describes how clean Signals feed the keyword → signal → grounded signal journey. A shared spam filter ensures noise is handled before signals enter that pipeline.

## Goals
1. Provide a reusable spam filtering abstraction that all mail connectors can call.
2. Make filtering configurable per environment/tenant without duplicating logic.
3. Emit telemetry so the PR pipeline can audit what was dropped and why.

## Non-Goals
- ML-based spam models (future change). We start with rule + provider label fusion.
- UI controls for tweaking spam thresholds (CLI/envs only for now).
- Weak-signal enrichment (covered by separate `add-weak-signal-engine` change).

## Proposed Architecture
```
mail/
  spam/
    mod.rs           // trait + shared structs
    default.rs       // heuristics implementation (labels, keywords, attachments)
```

### Trait
```rust
pub struct MailMetadata {
    pub provider: MailProvider,
    pub labels: Vec<String>,
    pub subject: Option<String>,
    pub headers: HashMap<String, String>,
}

pub struct MailSpamVerdict {
    pub is_spam: bool,
    pub score: f32,
    pub reason: String,
}

pub trait MailSpamFilter {
    fn evaluate(&self, meta: &MailMetadata) -> MailSpamVerdict;
}
```

- Default filter uses:
  - Provider labels (`SPAM`, `TRASH`, `PROMOTIONS` etc.).
  - Keyword/attachment heuristics (phishing keywords, suspicious extensions).
  - Configured allow/deny lists.
- Threshold is read from config; connectors can override temporarily (e.g., tests).

### Config Additions
- `POBLYSH_MAIL_SPAM_THRESHOLD` (float, default 0.8).
- `POBLYSH_MAIL_SPAM_ALLOWLIST` / `POBLYSH_MAIL_SPAM_DENYLIST` (comma-separated domains or email addresses).

### Connector Wiring
- Gmail obtains `MailSpamFilter` instance from registry or `Arc`.
- Before emitting signals, Gmail builds `MailMetadata` from Google message metadata, runs the filter, and short-circuits if `is_spam`.
- Telemetry event uses `tracing::info!(provider, score, reason)` so ops can tune configs.

### Future Extensibility
- Zoho Mail and Outlook connectors simply adopt the shared trait.
- Later, we can plug in ML scoring by implementing `MailSpamFilter` and injecting via DI.

## Validation Strategy
1. Unit tests for:
   - Label-based short circuit (e.g., Gmail `SPAM` label).
   - Threshold override behavior.
   - Allowlist bypass (journalist email should always pass even if flagged).
2. Integration test:
   - Use in-memory Gmail envelope; ensure spam message results in zero signals + telemetry entry.
3. Config validation:
   - Extend `AppConfig::validate` to ensure thresholds ∈ [0, 1] and allowlist entries are valid emails/domains.
