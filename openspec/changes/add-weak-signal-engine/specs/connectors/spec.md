## ADDED Requirements

### Requirement: Weak Signal Engine
The platform SHALL provide a weak-signal engine that consumes normalized Signals from any connector, scores them, and promotes high-confidence candidates into grounded signals with actionable recommendations.

#### Scenario: Signals are scored and grounded
- **WHEN** a normalized Signal is emitted by any connector
- **THEN** the weak-signal engine evaluates it using the six-dimension scoring model and, when the total score exceeds the configured threshold, creates a grounded signal with evidence, score breakdown, and recommended next steps

#### Scenario: Notifications for PR teams
- **WHEN** a grounded signal is created
- **THEN** the system emits an event/log/webhook that PR teams can subscribe to so they are notified of potential stories before the originating department reports them manually

#### Scenario: Querying grounded signals
- **WHEN** operators call the Signals API (or a dedicated endpoint) with the grounded filter
- **THEN** they receive grounded signals with score telemetry, evidence summaries, and status so they can prioritize follow-up work
