## ADDED Requirements

### Requirement: Signal Kind Taxonomy
Normalized `Signal.kind` values SHALL use action‑first snake_case verbs (e.g., `issue_created`, `pr_merged`, `message_posted`). Kinds MUST be provider‑agnostic and stable over time to support downstream filtering and scoring.

Conventions:
- Verb first, then subject noun (e.g., `file_created`, `calendar_event_updated`).
- Prefer provider‑neutral verbs aligning with user‑visible semantics (e.g., GitHub PR “opened” → `pr_opened`).
- Avoid ambiguous aliases; choose one canonical form per semantic.

#### Scenario: Naming follows verb_first_noun
- **WHEN** a connector emits a Signal for a newly created issue
- **THEN** `kind` is `issue_created` and not a provider‑specific label (e.g., not `issues.opened`)

#### Scenario: Canonical mapping for merge
- **WHEN** a pull request is merged
- **THEN** `kind` is `pr_merged`

### Requirement: Provider Event Mapping
The system SHALL map provider event types into the normalized taxonomy. Examples (non‑exhaustive, MVP focus):

GitHub:
- `issues.opened` → `issue_created`
- `issues.closed` → `issue_closed`
- `pull_request.opened` → `pr_opened`
- `pull_request.closed` with `merged = true` → `pr_merged`
- `pull_request.closed` with `merged = false` → `pr_closed`
- `push` → `code_pushed`
- `release.published` → `release_published`

Jira:
- `issue_created` → `issue_created`
- `issue_updated` with status transition to Done/Resolved → `issue_resolved`

Slack:
- `message` (channels/groups/ims) → `message_posted`
- `reaction_added` → `reaction_added`

Google Drive:
- file created → `file_created`
- file modified → `file_updated`

Google Calendar:
- event created → `calendar_event_created`
- event updated → `calendar_event_updated`

Gmail:
- message received → `email_received`
- message sent → `email_sent`

Zoho Cliq:
- message posted → `message_posted`

Zoho Mail:
- message received → `email_received`

#### Scenario: GitHub PR opened maps to pr_opened
- **GIVEN** a GitHub webhook payload for `pull_request` with `action: opened`
- **WHEN** the normalization mapping runs
- **THEN** `kind` is `pr_opened`

#### Scenario: Slack message maps to message_posted
- **GIVEN** a Slack event payload for a channel message
- **WHEN** normalization runs
- **THEN** `kind` is `message_posted`

#### Scenario: GitHub PR closed without merge maps to pr_closed
- **GIVEN** a GitHub webhook payload for `pull_request` with `action: closed` and `merged: false`
- **WHEN** normalization runs
- **THEN** `kind` is `pr_closed`

### Requirement: Golden Test Fixtures
Normalization MUST be validated via golden fixtures that encode sample provider payloads and expected kinds.

Fixture format (JSON):
```
{
  "provider": "github|jira|slack|...",
  "name": "short_case_name",
  "input": { /* provider payload */ },
  "expected": { "kind": "<normalized_kind>" }
}
```

Location:
- `tests/fixtures/normalization/<provider>/*.json`

#### Scenario: Fixture validates expected kind
- **GIVEN** a fixture file for GitHub PR opened
- **WHEN** the test harness loads the fixture and runs mapping
- **THEN** the test asserts `expected.kind == produced.kind`

### Requirement: Fixture Authoring Rules
Fixtures SHALL be stable, minimal, and representative:
- Only include necessary fields to drive mapping and a few invariants.
- Redact/omit volatile identifiers and timestamps unless required.
- Name files with a concise semantic and provider context.

Required keys:
- `provider` (string), `name` (string), `input` (object), `expected.kind` (string)

#### Scenario: Minimal viable fixture passes
- **WHEN** a fixture omits non‑essential fields
- **THEN** mapping still resolves the same canonical `kind`

#### Scenario: Missing required keys is reported clearly
- **GIVEN** a malformed fixture missing `expected.kind`
- **WHEN** the test harness loads fixtures
- **THEN** it fails with an actionable error naming the file and the missing key
