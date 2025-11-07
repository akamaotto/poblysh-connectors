# Gmail Connector Runbook

This runbook describes how to provision, operate, and troubleshoot the Gmail connector that delivers email Signals via Pub/Sub pushes plus incremental history syncs.

## OAuth Scopes & Watch Lifecycle
- **Required scopes**: `https://www.googleapis.com/auth/gmail.readonly`. Keep the scope list minimal so we only ingest metadata needed for Signals. If additional scopes are ever introduced, update the connector config and re-authorize every existing connection.
- **Per-user `watch`**: Every Gmail account must have an active [`users.watch`](https://developers.google.com/workspace/gmail/api/reference/rest/v1/users/watch) registration targeting the shared Pub/Sub topic. Watches expire within ~7 days; schedule a renewal job that re-issues the watch before the `expiration` timestamp and persists the new `historyId`.
- **Initial cursor**: The `watch` response includes a `historyId`. Persist it immediately so webhook deliveries can jump-start sync. If it’s missing (rare), fall back to a full history scan starting at ID `1`.
- **Operational caveats**:
  - Pause and re-issue the watch whenever the push subscription configuration changes (audience, auth, URL).
  - Gmail throttles `watch` calls; stagger requests per tenant to stay inside quotas.

## Pub/Sub Push Configuration
1. **Service account**: Create or reuse a Google Cloud service account that owns the Pub/Sub push subscription.
2. **Topic & subscription**: Configure the push subscription to POST to `https://{public-host}/webhooks/gmail/{tenant_id}` with OIDC tokens issued by the service account.
3. **Environment variables**:
   - `POBLYSH_PUBSUB_OIDC_AUDIENCE`: Exact `aud` string configured on the push subscription (commonly the webhook URL). Tokens that do not match are rejected with non-2xx responses so Pub/Sub retries.
   - `POBLYSH_PUBSUB_OIDC_ISSUERS`: Comma-separated allow-list, defaults to `accounts.google.com, https://accounts.google.com`.
   - `POBLYSH_PUBSUB_MAX_BODY_KB`: Body-size guardrail (default `256`) to ensure we ack within the 1s budget.
4. **OIDC validation flow**:
   - Pub/Sub signs every push with RS256. The connector fetches `https://www.googleapis.com/oauth2/v3/certs`, caches JWKS by `kid`, and validates `iss`, `aud`, `exp`, and `iat`.
   - Verification errors return HTTP 401; Pub/Sub automatically retries with backoff.
5. **Service account permissions**: Grant the service account the `roles/pubsub.publisher` role on the topic and `roles/iam.serviceAccountTokenCreator` so it can mint OIDC tokens.

## History Cursor Recovery
When Gmail returns `404` or an error mentioning “historyId not found/too old”, the stored cursor is no longer valid.

1. The connector surfaces `SyncError::Transient` with the failing `historyId` in logs.
2. Operators should enqueue a **bounded re-sync** job that:
   - Calls `users.history.list` with progressively older IDs until Gmail returns data.
   - If no valid cursor is found, create a fresh `watch` to obtain a new baseline `historyId` and reset the connector cursor to that value.
3. Document the recovery in incident notes so downstream consumers know Signals may be delayed during backfill.

## Troubleshooting Checklist
- **Webhook 401s**: Usually `aud` mismatch—confirm `POBLYSH_PUBSUB_OIDC_AUDIENCE` equals the subscription’s configured audience string.
- **No Signals after re-auth**: Ensure a watch was issued for the new connection and that the Pub/Sub subscription pushes to the correct tenant path.
- **Frequent 429/403**: Inspect Gmail API quotas in Cloud console. The connector translates these to `SyncError::RateLimited` with retry hints; if they persist, raise quotas or slow the sync schedule.
