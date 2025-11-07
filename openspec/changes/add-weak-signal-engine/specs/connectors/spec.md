## ADDED Requirements

### Requirement: Weak Signal Engine
The platform SHALL provide a weak-signal engine that:
- Consumes normalized Signals from any connector, scoped by tenant.
- Scores candidates using the six-dimension model.
- Promotes high-confidence candidates into grounded signals with actionable recommendations.
- Ensures tenant isolation, durable evidence for audit, and idempotent creation semantics.

#### Scenario: Signals are scored and grounded
- **WHEN** a normalized Signal is emitted by any connector for a given tenant
- **THEN** the weak-signal engine evaluates it using the six-dimension scoring model and, when the total score exceeds the configured (global or per-tenant) threshold, creates a grounded signal for that tenant with:
  - A score breakdown across all six dimensions,
  - Evidence that includes traceable references to contributing Signal IDs and source systems,
  - Recommended next steps for operators.

#### Scenario: Notifications for PR teams
- **WHEN** a grounded signal is created
- **THEN** the system emits an event/log/webhook scoped to the owning tenant that PR teams can subscribe to so they are notified of potential stories before the originating department reports them manually
- **AND** implementations MUST use HTTPS endpoints, MUST NOT log webhook secrets or bearer tokens, and SHOULD avoid logging full webhook URLs containing sensitive data.

#### Scenario: Querying grounded signals
- **WHEN** operators call the `/grounded-signals` endpoint with `tenant_id` and optional filters
- **THEN** they receive only grounded signals for that `tenant_id`, including score breakdown, evidence summaries, status, and pagination so they can prioritize follow-up work
- **AND** the API MUST enforce tenant isolation server-side (ignoring or rejecting any cross-tenant filters)
- **AND** repeated processing of the same underlying signals or clusters MUST NOT create duplicate grounded signals, using an idempotency mechanism defined in the design.
