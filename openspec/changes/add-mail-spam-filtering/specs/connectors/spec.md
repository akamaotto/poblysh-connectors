## ADDED Requirements

### Requirement: Shared Mail Spam Filtering
Mail connectors SHALL use a shared spam filtering abstraction to drop malicious messages while allowing promotional or collaboration threads to proceed through the signal pipeline.

#### Scenario: Centralized filter evaluated before emitting signals
- **WHEN** a mail connector (e.g., Gmail, Zoho Mail, Outlook) processes an inbound message
- **THEN** it MUST invoke the shared `MailSpamFilter` with provider metadata before emitting any Signals, and skip signal generation when the verdict is spam

#### Scenario: Configurable thresholds and allowlists
- **WHEN** operators set `POBLYSH_MAIL_SPAM_THRESHOLD`, `POBLYSH_MAIL_SPAM_ALLOWLIST`, or `POBLYSH_MAIL_SPAM_DENYLIST`
- **THEN** the shared filter honors those values without requiring code changes in individual connectors

#### Scenario: Telemetry for rejected messages
- **WHEN** a message is dropped as spam
- **THEN** the connector logs provider, score, and reason so teams can audit and tune spam heuristics
