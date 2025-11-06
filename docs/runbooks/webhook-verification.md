# Webhook Signature Verification Runbook

## Overview

This runbook covers the configuration, operation, and troubleshooting of webhook signature verification for GitHub and Slack providers in the Connectors API.

## Configuration

### Environment Variables

The webhook signature verification system uses the following environment variables:

#### GitHub Webhooks
```bash
POBLYSH_WEBHOOK_GITHUB_SECRET=your_github_webhook_secret
```
- **Required**: Yes (for public GitHub webhook verification)
- **Format**: String (the webhook secret configured in GitHub)
- **Source**: Centralized secrets manager (recommended) or encrypted local file

#### Slack Webhooks
```bash
POBLYSH_WEBHOOK_SLACK_SIGNING_SECRET=your_slack_signing_secret
POBLYSH_WEBHOOK_SLACK_TOLERANCE_SECONDS=300
```
- **POBLYSH_WEBHOOK_SLACK_SIGNING_SECRET**: Required for public Slack webhook verification
- **POBLYSH_WEBHOOK_SLACK_TOLERANCE_SECONDS**: Optional, defaults to 300 seconds (5 minutes)

### Secrets Management

#### Production Environment
1. **Source**: Use your organization's approved centralized secrets manager
2. **Encryption**: AES-256 encryption at rest
3. **Access**: Audited access with role-based permissions
4. **Rotation**: 30-90 day cadence with dual-key grace period

#### Local Development (Non-Production)
```bash
# Create encrypted local secrets file (for development only)
echo "your_webhook_secret" | openssl enc -aes-256-cbc -base64 > .webhook-secrets.enc

# WARNING: This file is FOR LOCAL DEVELOPMENT ONLY
# DO NOT deploy to production environments
```

## Webhook Endpoints

### Public Webhook Routes
- **GitHub**: `POST /webhooks/github/{tenant_id}`
- **Slack**: `POST /webhooks/slack/{tenant_id}`

### Signature Requirements

#### GitHub
- **Header**: `X-Hub-Signature-256`
- **Format**: `sha256=<hex_digest>`
- **Algorithm**: HMAC-SHA256 of raw request body

#### Slack
- **Headers**: 
  - `X-Slack-Signature`: `v0=<hex_digest>`
  - `X-Slack-Request-Timestamp`: Unix timestamp
- **Format**: HMAC-SHA256 of `v0:{timestamp}:{raw_body}`
- **Tolerance**: ±5 minutes (configurable)

## Provider Configuration

### GitHub Webhooks
1. Go to your repository Settings → Webhooks
2. Add webhook URL: `https://your-api-domain/webhooks/github/{tenant_id}`
3. Set Content type: `application/json`
4. Add secret: Use the same value as `POBLYSH_WEBHOOK_GITHUB_SECRET`
5. Select events: Choose which events trigger webhooks

### Slack Webhooks
1. Create Slack app or use existing one
2. Enable Interactivity & Shortcuts
3. Set Request URL: `https://your-api-domain/webhooks/slack/{tenant_id}`
4. Use the Signing Secret as `POBLYSH_WEBHOOK_SLACK_SIGNING_SECRET`

## Monitoring and Alerting

### Key Metrics
Monitor the following metrics in your observability platform:

1. **Signature Verification Success Rate**
   - `webhook_verification_success_total{provider="github|slack"}`
   - Target: >95%

2. **Signature Verification Failure Rate**
   - `webhook_verification_failure_total{provider="github|slack",reason="missing|invalid_format|failed|timestamp"}`
   - Alert: >5% failure rate over 5 minutes

3. **Request Processing Time**
   - `webhook_verification_duration_seconds{provider="github|slack"}`
   - Alert: >1 second P95

### Structured Logs
Key log fields to monitor:

```json
{
  "timestamp": "2025-01-03T10:00:00Z",
  "level": "info",
  "provider": "github",
  "tenant_id": "550e8400-e29b-41d4-a716-446655440000",
  "body_size": 1024,
  "message": "Webhook signature verified successfully"
}
```

```json
{
  "timestamp": "2025-01-03T10:00:00Z",
  "level": "error",
  "provider": "slack",
  "error": "Signature verification failed",
  "message": "Webhook signature verification failed"
}
```

## Troubleshooting

### Common Issues

#### 1. 401 Unauthorized - Missing Signature
**Symptoms**: Webhook requests rejected with 401 status
**Causes**: 
- Missing signature headers
- Provider not configured with secret

**Resolution**:
```bash
# Check if secrets are configured
curl -s "http://localhost:8080/healthz" | jq '.config.webhook_github_secret'

# Verify provider configuration
grep -E "POBLYSH_WEBHOOK_" .env.local
```

#### 2. 401 Unauthorized - Invalid Signature
**Symptoms**: Valid webhooks being rejected
**Causes**:
- Secret mismatch between provider and API
- Request body modification

**Resolution**:
1. Verify secrets match in both systems
2. Check for request body modifications (proxies, CDNs)
3. Test with webhook signing tools

#### 3. 401 Unauthorized - Timestamp Too Old
**Symptoms**: Slack webhooks rejected with timestamp errors
**Causes**:
- Clock skew between systems
- Request processing delays

**Resolution**:
```bash
# Check server time synchronization
ntpq -p

# Increase tolerance if needed
export POBLYSH_WEBHOOK_SLACK_TOLERANCE_SECONDS=600
```

#### 4. 404 Not Found - Unsupported Provider
**Symptoms**: Webhook requests for unsupported providers
**Causes**: Provider not implemented

**Resolution**: Currently only GitHub and Slack are supported. Contact engineering for additional providers.

### Debugging Tools

#### Test GitHub Signature
```bash
#!/bin/bash
# Test GitHub webhook signature
SECRET="your_webhook_secret"
PAYLOAD='{"event": "push", "repository": {"name": "test"}}'

# Generate signature
SIGNATURE="sha256=$(echo -n "$PAYLOAD" | openssl dgst -sha256 -hmac "$SECRET" | cut -d' ' -f2)"

# Send test request
curl -X POST \
  -H "Content-Type: application/json" \
  -H "X-Hub-Signature-256: $SIGNATURE" \
  -d "$PAYLOAD" \
  "https://your-api-domain/webhooks/github/your-tenant-id"
```

#### Test Slack Signature
```bash
#!/bin/bash
# Test Slack webhook signature
SECRET="your_slack_signing_secret"
TIMESTAMP=$(date +%s)
PAYLOAD='{"type": "url_verification", "challenge": "test"}'

# Create base string
BASE_STRING="v0:$TIMESTAMP:$PAYLOAD"

# Generate signature
SIGNATURE="v0=$(echo -n "$BASE_STRING" | openssl dgst -sha256 -hmac "$SECRET" | cut -d' ' -f2)"

# Send test request
curl -X POST \
  -H "Content-Type: application/json" \
  -H "X-Slack-Signature: $SIGNATURE" \
  -H "X-Slack-Request-Timestamp: $TIMESTAMP" \
  -d "$PAYLOAD" \
  "https://your-api-domain/webhooks/slack/your-tenant-id"
```

## Security Considerations

### 1. Secret Rotation
- Follow your organization's secrets rotation policy
- Use dual-key approach during rotation periods
- Update provider configurations before removing old secrets

### 2. Rate Limiting
Public webhook endpoints include basic rate limiting. For production:
- Configure per-IP and global rate limits
- Consider upstream WAF/CDN protection
- Monitor for abuse patterns

### 3. Log Security
- All secrets are redacted from logs
- Signature headers are filtered before persistence
- Monitor for repeated failures from same sources

## Performance Considerations

### 1. Processing Time
- HMAC operations are fast (~0.1ms)
- Most time spent in body reading and validation
- Monitor for unusual processing delays

### 2. Memory Usage
- Request bodies are loaded into memory for verification
- Large webhooks (>10MB) may cause memory pressure
- Consider implementing size limits for very large payloads

## Runbook Checklist

### Deployment Checklist
- [ ] Configure webhook secrets in secrets manager
- [ ] Update provider webhook URLs
- [ ] Verify secret rotation procedures
- [ ] Configure monitoring and alerting
- [ ] Test webhook delivery with signature verification
- [ ] Update firewall rules for public webhook access

### Incident Response
- [ ] Check signature verification failure rates
- [ ] Verify secret configuration
- [ ] Monitor for abuse or DDoS patterns
- [ ] Check for provider-side configuration changes
- [ ] Review system time synchronization
- [ ] Document incident and resolution steps

## References

- [GitHub Webhooks Documentation](https://docs.github.com/en/developers/webhooks-and-events/webhooks/about-webhooks)
- [Slack API Verification](https://api.slack.com/authentication/verifying-requests-from-slack)
- [Connectors API Documentation](http://localhost:8080/docs)