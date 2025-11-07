# Jira Connector Setup

This runbook explains how to supply configuration for the Jira connector and how to point Jira Cloud webhooks at a local Connectors API instance.

## Required Environment

Set the following variables (or their `POBLYSH_` prefixed equivalents) before starting the service:

- `JIRA_CLIENT_ID`: Atlassian OAuth client identifier.
- `JIRA_CLIENT_SECRET`: Atlassian OAuth client secret.
- `JIRA_OAUTH_BASE` (optional): Override for the Atlassian OAuth base URL. Defaults to `https://auth.atlassian.com`.
- `JIRA_API_BASE` (optional): Override for the Atlassian REST API base. Defaults to `https://api.atlassian.com`.
- `WEBHOOK_JIRA_SECRET` (optional): Shared secret used for webhook basic authentication. When omitted, public webhook routes skip verification (operator-protected routes always accept the payload).

The configuration loader does not inject placeholder Jira credentials. Provide real `JIRA_CLIENT_ID` and `JIRA_CLIENT_SECRET` for any environment where the Jira connector is enabled.

## Local OAuth Flow

1. Ensure the OAuth app registered in the Atlassian developer console lists the redirect URI used by the tenant (for local testing the backend falls back to `http://localhost:3000/callback`).
2. Start the Connectors API with the variables above. The registry will register the Jira connector even with placeholder credentials, enabling the `/connect/jira` endpoints.

## Local Webhook Configuration

1. Expose your local server (for example with `ngrok http 8080`) so Jira can reach it.
2. In Jira Cloud, create or update the webhook endpoint to point at `https://<public-host>/webhooks/jira/<tenant_id>`.
3. If you set `WEBHOOK_JIRA_SECRET`, configure the same value in Jira and ensure webhooks send `Authorization: Bearer <secret>`. Only the bearer form is accepted; legacy `X-Webhook-Secret` is not supported.
4. For local development you may leave `WEBHOOK_JIRA_SECRET` unset. Public webhook routes automatically bypass verification when the secret is absent and the profile is `local` or `test`.
5. Restart the Connectors API after updating secrets so the registry picks up the changes.

With the environment configured, the Jira connector supports authorization flows, webhook ingestion, and incremental sync using the same normalized signals emitted in production.
