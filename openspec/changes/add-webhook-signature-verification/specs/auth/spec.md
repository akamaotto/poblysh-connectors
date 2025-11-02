## MODIFIED Requirements

### Requirement: Public Endpoints Bypass
The following endpoints SHALL be accessible without authentication or tenant header: `/healthz`, `/readyz`, `/docs`, `/openapi.json`.

#### Scenario: Health without auth
- WHEN calling `GET /healthz` without headers
- THEN the response is HTTP 200

## ADDED Requirements

### Requirement: Public Webhook Bypass With Valid Signature
Webhook endpoints at `POST /webhooks/{provider}/{tenant_id}` SHALL bypass operator bearer authentication when a valid provider signature is present (per the webhooks spec). Requests without a valid signature remain protected.

#### Scenario: Signed webhook bypasses auth
- GIVEN a valid provider signature is present
- WHEN calling `POST /webhooks/github/{tenant}` without `Authorization`
- THEN the request is accepted and processed (subject to signature validation)

#### Scenario: Unsigned webhook requires operator auth
- WHEN calling `POST /webhooks/github/{tenant}` without signature headers
- THEN the request is rejected with HTTP 401 unless `Authorization: Bearer` is provided

