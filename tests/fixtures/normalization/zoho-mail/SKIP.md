# Zoho Mail Normalization

## Rationale
Zoho Mail currently does not emit normalized signals (webhooks are not
supported in the MVP sync path), so there are no payloads to assert yet.

## Plan
Once Zoho Mail emits `email_received` / `email_sent`, add a normalization helper
for its sync payloads and cover each Signal.kind with fixtures.
