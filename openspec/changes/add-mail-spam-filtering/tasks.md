## 1. Abstraction
- [ ] 1.1 Define `MailSpamVerdict` + `MailSpamFilter` trait (score, reason, metadata).
- [ ] 1.2 Implement default filter that uses provider labels + keyword heuristics.
- [ ] 1.3 Add config knobs (`POBLYSH_MAIL_SPAM_THRESHOLD`, allow/deny lists per domain).

## 2. Integration
- [ ] 2.1 Wire Gmail connector to use the shared filter before emitting Signals.
- [ ] 2.2 Add telemetry hooks so rejected messages log provider, reason, and score.
- [ ] 2.3 Provide stub integration helpers for upcoming Zoho Mail / Outlook connectors.

## 3. Validation
- [ ] 3.1 Unit tests covering thresholding, allowlist overrides, and telemetry output.
- [ ] 3.2 Integration test in Gmail path to ensure spammed messages do not create Signals.
- [ ] 3.3 Update documentation/specs referencing the new spam module and config.
