## Why
Email connectors (Gmail today, Zoho Mail/Outlook soon) need shared spam intelligence to drop purely malicious payloads while still forwarding legitimate promotional or collaboration threads into the signal pipeline. The Gmail change temporarily embedded spam heuristics directly into its connector, but that broke single-responsibility and prevented other mail connectors from reusing the logic. We need a centralized, configurable spam filtering layer that every mail connector can call before emitting Signals.

## What Changes
- Define a reusable `MailSpamFilter` trait + default implementation that:
  - Scores inbound messages based on provider labels and Poblysh heuristics/ML scores.
  - Supports environment-driven thresholds and allow/deny lists per tenant.
  - Exposes telemetry so the signal pipeline knows why a message was dropped.
- Wire Gmail to call the shared filter before emitting Signals (no business logic in the connector).
- Add extension points for future Zoho Mail and Outlook connectors.
- Surface configuration knobs under the existing config module (`POBLYSH_MAIL_SPAM_THRESHOLD`, optional provider-specific overrides).

## Impact
- Specs: `connectors` capability gains requirements for mail spam filtering abstraction.
- Code: new spam module (likely `src/mail/spam.rs`), config updates, Gmail wiring, unit tests.
- No external API changes; this is internal plumbing that improves consistency.
