## ADDED Requirements

### Requirement: Pub/Sub OIDC Verification Configuration
The system SHALL provide configuration for verifying Google Cloud Pub/Sub push OIDC tokens and limiting webhook ingress size.

Details:
- `POBLYSH_PUBSUB_OIDC_AUDIENCE` (string) — required; the JWT `aud` claim MUST exactly match this value. Common patterns include the public webhook URL or a custom audience string set on the subscription.
- `POBLYSH_PUBSUB_OIDC_ISSUERS` (comma‑separated list) — optional; defaults to `accounts.google.com, https://accounts.google.com`. The JWT `iss` claim MUST exactly match one of these values.
- `POBLYSH_PUBSUB_MAX_BODY_KB` (integer) — optional; default `256`. Requests exceeding this size MUST be rejected before verification to preserve fast ack behavior.
- JWKS endpoint: `https://www.googleapis.com/oauth2/v3/certs` SHALL be used to validate the JWT signature by `kid`, and keys SHOULD be cached with ETag support.

#### Scenario: Audience must match exactly
- **WHEN** a push request is received with an OIDC JWT where `aud != POBLYSH_PUBSUB_OIDC_AUDIENCE`
- **THEN** the request is rejected with a non‑2xx status code to trigger Pub/Sub retry and a verification error is logged

#### Scenario: Issuer must be allowed
- **WHEN** a push request is received with an OIDC JWT where `iss` is not in `POBLYSH_PUBSUB_OIDC_ISSUERS`
- **THEN** the request is rejected with a non‑2xx status code to trigger Pub/Sub retry and a verification error is logged

#### Scenario: Expired or not‑yet‑valid token rejected
- **WHEN** the OIDC JWT is expired or has an `iat` outside an acceptable skew
- **THEN** the request is rejected with a non‑2xx status code to trigger Pub/Sub retry

#### Scenario: Webhook request size limited
- **WHEN** a push request body exceeds `POBLYSH_PUBSUB_MAX_BODY_KB`
- **THEN** the request is rejected with a non‑2xx status code and no work is enqueued

