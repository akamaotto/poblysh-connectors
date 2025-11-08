## MODIFIED Requirements

### Requirement: Public Endpoints Bypass

The following endpoints SHALL be accessible without authentication or tenant header: `/healthz`, `/readyz`, `/docs`, `/openapi.json`.

#### Scenario: Health without auth

- WHEN calling `GET /healthz` without headers
- THEN the response is HTTP 200

## ADDED Requirements

### Requirement: Public Webhook Auth Bypass With Valid Signature

The system SHALL allow specific webhook endpoints to bypass operator bearer authentication when a valid provider signature is present, as defined in the `api-webhooks` capability.

Endpoints:

- `POST /webhooks/{provider}/{tenant_id}`: Public, signature-gated variant.
- `POST /webhooks/{provider}`: Operator-auth-protected variant retained for local/test and controlled environments.

Behavior (normative):

- Public path with signature:
  - When a request targets `POST /webhooks/{provider}/{tenant_id}`:
    - AND the `provider` is recognized and supported.
    - AND the corresponding provider webhook verification secret is configured (per `config` capability).
    - AND the provider-specific signature is present and valid (per `api-webhooks` signature verification requirements).
    - THEN the request SHALL be accepted without requiring `Authorization: Bearer` operator credentials.
  - Requests on this path that lack a valid signature:
    - SHALL be rejected with HTTP 401 (Unauthorized) when no valid operator bearer token is provided.
    - SHALL use the unified `application/problem+json` error envelope with a suitable code such as `INVALID_SIGNATURE` or `UNAUTHORIZED` (as defined in the core API error spec).

- Operator-auth precedence:
  - For both `POST /webhooks/{provider}` and `POST /webhooks/{provider}/{tenant_id}`:
    - IF a valid operator bearer token is present:
      - The request SHALL be authorized regardless of signature presence or validity.
      - Signature verification MAY be skipped to avoid double work.
    - This precedence ensures operational overrides and compatibility for internal tooling and testing.

- Missing provider secret:
  - IF the provider’s webhook verification secret is not configured:
    - Public verification for that provider on `POST /webhooks/{provider}/{tenant_id}` SHALL be disabled.
    - Any request to this public path without a valid operator bearer token SHALL be rejected with HTTP 401 (Unauthorized), even if signature headers are present.
    - The operator-protected `POST /webhooks/{provider}` path SHALL continue to function as defined by the existing auth spec.

- Unsupported providers:
  - IF `provider` does not map to a supported provider with a defined verification strategy:
    - Requests to either `/webhooks/{provider}` or `/webhooks/{provider}/{tenant_id}` SHALL result in HTTP 404 (Not Found) with the standard problem+json envelope.

- Consistency with auth rules:
  - No other endpoints SHALL inherit this signature-based bypass implicitly.
  - Public unauthenticated access (without signatures) remains restricted to the explicitly listed health/docs endpoints in the base `auth` spec.

#### Scenario: Signed public webhook bypasses auth

- GIVEN a supported provider `github` with `POBLYSH_WEBHOOK_GITHUB_SECRET` configured
- AND a request to `POST /webhooks/github/{tenant_id}` with a valid HMAC-SHA256 signature (per `api-webhooks` / GitHub verification rules)
- WHEN the request does NOT include an `Authorization: Bearer` operator token
- THEN the request is accepted (HTTP 202) based solely on the valid signature
- AND the response uses the standard webhook ingest success format

#### Scenario: Unsigned public webhook requires operator auth

- GIVEN a supported provider `github` with `POBLYSH_WEBHOOK_GITHUB_SECRET` configured
- WHEN calling `POST /webhooks/github/{tenant_id}` without required signature headers
- AND without a valid `Authorization: Bearer` operator token
- THEN the request is rejected with HTTP 401 (Unauthorized) using the problem+json envelope
- AND the error `code` is a suitable auth-related code (e.g., `INVALID_SIGNATURE` or `UNAUTHORIZED`) defined in the core spec

#### Scenario: Invalid signature on public webhook is rejected

- GIVEN a supported provider `slack` with `POBLYSH_WEBHOOK_SLACK_SIGNING_SECRET` configured
- WHEN calling `POST /webhooks/slack/{tenant_id}` with malformed, mismatched, or expired Slack signature headers
- AND without a valid operator bearer token
- THEN the request is rejected with HTTP 401 (Unauthorized) using problem+json
- AND verification metrics record a failure outcome

#### Scenario: Operator token overrides signature requirements

- GIVEN a supported provider `github` with `POBLYSH_WEBHOOK_GITHUB_SECRET` configured
- WHEN calling `POST /webhooks/github/{tenant_id}` with a valid operator `Authorization: Bearer` token
- AND with either missing or invalid signature headers
- THEN the request is accepted (subject to normal authorization and routing rules)
- AND no signature verification is required for this request

#### Scenario: Missing provider secret disables public verification

- GIVEN `POBLYSH_WEBHOOK_GITHUB_SECRET` is NOT configured
- WHEN a request is sent to `POST /webhooks/github/{tenant_id}`:
  - WITHOUT a valid operator bearer token
  - EVEN IF it includes GitHub-style signature headers
- THEN the system SHALL reject the request with HTTP 401 (Unauthorized)
- AND SHALL NOT treat the signature as valid because verification is disabled for that provider
- AND the operator-authenticated `POST /webhooks/github` endpoint continues to function per existing auth rules

#### Scenario: Unsupported provider returns 404

- GIVEN no configured verification strategy for provider slug `unknown`
- WHEN calling `POST /webhooks/unknown/{tenant_id}` with or without signatures or operator auth
- THEN the system responds with HTTP 404 (Not Found) using the problem+json envelope
- AND no webhook processing occurs