# GitHub Webhook Configuration

This guide explains how to configure GitHub webhooks for the Poblysh Connectors API.

## Prerequisites

- GitHub repository or organization admin access
- Configured Poblysh Connectors API instance
- GitHub OAuth app configured in Poblysh

## Webhook Setup

### 1. Create GitHub OAuth App

1. Go to GitHub Settings → Developer settings → OAuth Apps
2. Click "New OAuth App"
3. Configure:
   - **Application name**: Your app name
   - **Homepage URL**: Your API base URL
   - **Authorization callback URL**: `https://your-api-domain/connect/github/callback`
   - **Enable Device Flow**: Unchecked (not needed for MVP)

### 2. Configure Environment Variables

Set the following environment variables in your Poblysh configuration:

```bash
# GitHub OAuth Configuration
POBLYSH_GITHUB_CLIENT_ID=your_github_client_id
POBLYSH_GITHUB_CLIENT_SECRET=your_github_client_secret

# GitHub Webhook Configuration (optional for public webhooks)
POBLYSH_WEBHOOK_GITHUB_SECRET=your_webhook_secret
```

### 3. Configure Repository Webhooks

For each repository you want to monitor:

1. Go to repository Settings → Webhooks
2. Click "Add webhook"
3. Configure:
   - **Payload URL**: `https://your-api-domain/webhooks/github/{tenant_id}`
     - Replace `{tenant_id}` with your actual tenant UUID
   - **Content type**: `application/json`
   - **Secret**: Use the same value as `POBLYSH_WEBHOOK_GITHUB_SECRET`
   - **SSL verification**: Enabled (recommended)

### 4. Select Events

Choose which events trigger webhooks. For MVP, we support:

#### Issues Events
- Issues opened: `issues` → `opened`
- Issues closed: `issues` → `closed` 
- Issues reopened: `issues` → `reopened`
- Issues edited: `issues` → `edited`

#### Pull Request Events
- Pull requests opened: `pull_request` → `opened`
- Pull requests closed: `pull_request` → `closed`
- Pull requests reopened: `pull_request` → `reopened`
- Pull requests edited: `pull_request` → `edited`

**Recommended Event Selection for MVP:**
- ✅ Issues
- ✅ Pull requests

## Webhook URL Format

### Tenant-Specific Webhooks (Recommended)
```
POST https://your-api-domain/webhooks/github/{tenant_id}
```

- Uses tenant ID from URL path
- Bypasses operator authentication when signature is valid
- Recommended for production

### Operator-Authenticated Webhooks
```
POST https://your-api-domain/webhooks/github
Headers:
- Authorization: Bearer {operator_token}
- X-Tenant-Id: {tenant_uuid}
```

- Requires operator authentication
- Uses tenant ID from header
- Useful for testing and local development

## Signature Verification

GitHub webhooks use HMAC-SHA256 signature verification:

1. GitHub sends `X-Hub-Signature-256` header with format: `sha256={hex_signature}`
2. API verifies signature using webhook secret
3. Requests with invalid signatures are rejected

## Supported Event Mappings

The GitHub connector maps webhook events to normalized signal types:

| GitHub Event | Action | Signal Type | Description |
|-------------|--------|-------------|-------------|
| `issues` | `opened` | `issue_created` | New issue created |
| `issues` | `closed` | `issue_closed` | Issue closed |
| `issues` | `reopened` | `issue_reopened` | Issue reopened |
| `issues` | `edited` | `issue_updated` | Issue updated |
| `pull_request` | `opened` | `pr_created` | Pull request opened |
| `pull_request` | `closed` (merged) | `pr_merged` | Pull request merged |
| `pull_request` | `closed` (not merged) | `pr_closed` | Pull request closed |
| `pull_request` | `reopened` | `pr_reopened` | Pull request reopened |
| `pull_request` | `edited` | `pr_updated` | Pull request updated |

## Rate Limits

- GitHub webhooks have no rate limits for delivery
- API respects GitHub API rate limits during sync operations
- Sync backfill handles rate limits with exponential backoff

## Troubleshooting

### Webhook Not Received
1. Check webhook URL is correct and accessible
2. Verify SSL certificate is valid
3. Check webhook secret matches configuration
4. Review GitHub webhook delivery logs

### Signature Verification Failed
1. Ensure `POBLYSH_WEBHOOK_GITHUB_SECRET` matches webhook configuration
2. Check webhook payload isn't modified in transit
3. Verify timezone and timestamp handling

### Events Not Processed
1. Check that event types are selected in webhook configuration
2. Review API logs for processing errors
3. Verify tenant ID is correct in webhook URL

### Rate Limiting During Sync
1. Monitor API logs for rate limit warnings
2. Adjust sync frequency if needed
3. Check GitHub API rate limit status

## Security Considerations

- Always use HTTPS for webhook URLs
- Keep webhook secret secure and rotate regularly
- Monitor webhook deliveries for suspicious activity
- Use tenant-specific URLs to prevent cross-tenant data access

## Testing Webhooks

### Local Testing
Use tools like ngrok to expose your local API:
```bash
ngrok http 8080
# Configure webhook with: https://abc123.ngrok.io/webhooks/github/{tenant_id}
```

### Webhook Testing Tools
- Use GitHub's "Redeliver" feature in webhook settings
- Test with sample payloads using curl:
```bash
curl -X POST https://your-api-domain/webhooks/github/{tenant_id} \
  -H "Content-Type: application/json" \
  -H "X-Hub-Signature-256: sha256={computed_signature}" \
  -d '{"action":"opened","issue":{"id":123}}'
```

## Monitoring

Monitor webhook processing through:
- API logs showing webhook ingestion
- Signal counts in your monitoring dashboard
- GitHub webhook delivery status in repository settings
- Rate limit metrics from GitHub API calls