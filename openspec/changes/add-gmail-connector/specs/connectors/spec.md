## ADDED Requirements
### Requirement: Gmail Connector
The system SHALL provide a Gmail connector that supports OAuth2 authorization, Pub/Sub push webhook ingestion with OIDC verification, and incremental synchronization using Gmail History.

#### Scenario: Provider metadata registered
- **WHEN** listing providers
- **THEN** `gmail` appears with `auth_type="oauth2"`, `webhooks=true`, and scopes including `https://www.googleapis.com/auth/gmail.readonly`

#### Scenario: Authorization URL generated
- **WHEN** `authorize(tenant)` is called
- **THEN** a Google OAuth URL is returned with the Gmail scope, `response_type=code`, and `access_type=offline` when supported

#### Scenario: Token exchange persists connection
- **WHEN** `exchange_token(code)` succeeds
- **THEN** a `connections` row is created with `provider_slug='gmail'`, access/refresh tokens, expiry, and stored scopes

#### Scenario: Webhook push verified and acked fast
- **WHEN** a Pub/Sub push is received for Gmail
- **THEN** the request is verified via Google OIDC (issuer, audience, signature) and, on success, the handler decodes the base64 `data` envelope and responds with `202 Accepted` within one second while enqueueing sync (any 2xx acks Pub/Sub; `202` is our standard)

#### Scenario: Incremental sync via history
- **WHEN** `sync(connection, cursor?)` runs
- **THEN** it calls `users.history.list` starting from the cursor or payload `historyId`, emits Signals for updates/deletes, and advances the cursor to the highest processed `historyId`

#### Scenario: Idempotent delivery
- **WHEN** duplicate Pub/Sub deliveries occur
- **THEN** the system avoids duplicate work using idempotent enqueueing keyed by Pub/Sub `messageId` or `(connection_id, historyId)`

#### Scenario: Invalid history cursor recovery
- **WHEN** Gmail returns an invalid/too‑old `historyId`
- **THEN** the connector records the condition and initiates a documented bounded re‑sync strategy rather than failing silently
