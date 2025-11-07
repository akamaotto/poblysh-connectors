## ADDED Requirements

### Requirement: Provider Metadata (Zoho Cliq)
The registry SHALL expose a `zoho-cliq` provider with metadata describing webhook‑only support for MVP.

Details:
- `name = "zoho-cliq"`
- `auth_type = "custom(webhook)"` (no OAuth in MVP)
- `scopes = []`
- `webhooks = true`

#### Scenario: Metadata fields populated
- **WHEN** listing providers
- **THEN** `zoho-cliq` appears with `{ name: "zoho-cliq", auth_type: "custom(webhook)", scopes: [], webhooks: true }`

### Requirement: Zoho Cliq Webhook Handling (Messages)
The connector SHALL handle Zoho Cliq outgoing webhook payloads for message events and emit normalized Signals.

Details (MVP):
- Verification (MVP):
  - Token-based: MUST verify `Authorization: Bearer <POBLYSH_WEBHOOK_ZOHO_CLIQ_TOKEN>` using constant‑time comparison.
  - Do not accept query parameters for secrets.
  - Note: HMAC support MAY be introduced in a follow‑up change if Cliq docs confirm exact header names and signature construction.
- Header forwarding: all request headers MUST be forwarded into `payload.headers` with lower‑case keys to simplify matching
- Signal kinds (MVP): `message_posted`, `message_updated`, `message_deleted`
- Mapping: derive kind from payload action/type fields; include normalized fields `{ message_id, channel_id, user_id, text, occurred_at }`
- Dedupe: if payload contains a stable event or message identifier, set `dedupe_key` to that value

#### Scenario: Valid message_posted produces a Signal
- **WHEN** a verified webhook with a message creation event is received
- **THEN** one `message_posted` Signal is emitted with `occurred_at` from payload and normalized fields

#### Scenario: Valid message_updated produces a Signal
- **WHEN** a verified webhook with a message update event is received
- **THEN** one `message_updated` Signal is emitted

#### Scenario: Valid message_deleted produces a Signal
- **WHEN** a verified webhook with a message deletion event is received
- **THEN** one `message_deleted` Signal is emitted

### Requirement: No OAuth In MVP
The connector SHALL return a clear error or no‑op for OAuth methods in MVP.

#### Scenario: Authorization unsupported
- **WHEN** `authorize` is called for `zoho-cliq`
- **THEN** the connector returns an error indicating OAuth is unsupported in MVP

#### Scenario: Token exchange unsupported
- **WHEN** `exchange_token` is called
- **THEN** the connector returns an error indicating OAuth is unsupported in MVP

### Requirement: Public Webhook Path Usage
The Zoho Cliq webhooks SHALL use the public webhook path variant `POST /webhooks/zoho-cliq/{tenant}` and follow the platform’s public access rules.

#### Scenario: Signed public request accepted
- **WHEN** calling `POST /webhooks/zoho-cliq/{tenant}` without operator auth but with valid `Authorization: Bearer <token>` matching `POBLYSH_WEBHOOK_ZOHO_CLIQ_TOKEN`
- **THEN** the response is HTTP 202 with body `{ "status": "accepted" }` and the request is processed; `payload.headers` MUST contain lower‑case keys

#### Scenario: Invalid signature/token rejected
- **WHEN** calling the public path without valid signature or token
- **THEN** the response is HTTP 401 and the request is not processed
