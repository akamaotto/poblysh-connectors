## ADDED Requirements

### Requirement: Jira Connector
The system SHALL provide a Jira connector implementing OAuth2 authorization, selective webhook handling, and incremental sync for issues while emitting normalized Signals that follow the platform taxonomy.

#### Scenario: OAuth authorize URL includes mandatory parameters
- **WHEN** `authorize(tenant)` is called
- **THEN** the returned URL has host `auth.atlassian.com` and includes `response_type=code`, `client_id` from configuration, `audience=api.atlassian.com`, `prompt=consent`, `access_type=offline`, a non-empty `state`, and a `redirect_uri` that matches an allow-listed callback configured for the tenant

#### Scenario: Token exchange persists refreshable connection
- **WHEN** `exchange_token(code)` succeeds
- **THEN** a `connections` row exists with `provider_slug='jira'`, encrypted access and refresh tokens, token expiry timestamps, the Atlassian cloud/site identifier, and account identity metadata persisted in connection settings

#### Scenario: Webhook filters non-issue events
- **WHEN** `handle_webhook(payload)` receives a non-issue webhook event
- **THEN** it returns an empty signal list and records a trace indicating the event type was ignored

#### Scenario: Webhook emits normalized issue Signals
- **WHEN** `handle_webhook(payload)` receives an issue create or update event
- **THEN** it emits one or more Signals with kinds `issue_created` or `issue_updated`, each containing normalized fields `{ issue_id, issue_key, project_key, summary, status, assignee, url, occurred_at }` and the original payload

#### Scenario: Incremental sync paginates by updated timestamp
- **GIVEN** Jira REST returns multiple pages of issues updated since a cursor
- **WHEN** `sync(connection, cursor?)` is called
- **THEN** it iterates through all pages, emits Signals ordered by `updated` ascending, and advances the stored cursor to the greatest `updated` timestamp processed

#### Scenario: Sync deduplicates against recent webhooks
- **GIVEN** an issue already emitted via webhook since the current cursor
- **WHEN** `sync(connection, cursor?)` is called
- **THEN** it emits at most one Signal per issue per `updated` timestamp, avoiding duplicates across webhook and sync pathways

#### Scenario: Dedupe key generation for consistency
- **WHEN** processing Jira webhook or sync events
- **THEN** dedupe keys SHALL be generated using the format `jira:{signal_kind}:{issue_id}:{updated_timestamp}` where:
  - `signal_kind` is one of `issue_created` or `issue_updated`
  - `issue_id` is the numeric Jira issue identifier
  - `updated_timestamp` is the ISO 8601 timestamp from the issue's `updated` field
- **AND** this ensures consistent deduplication across webhook and sync data sources
