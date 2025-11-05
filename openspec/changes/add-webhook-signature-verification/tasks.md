## 0. Research
- [ ] 0.1 Execute the deep research algorithm: parallel search Slack/GitHub verification docs, RustCrypto `hmac`/`sha2` guidance, and in-repo constant-time patterns; reinforce with focused follow-up queries; capture synthesized notes before implementation.

## 1. Implementation
- [ ] 1.1 Add config for `POBLYSH_WEBHOOK_GITHUB_SECRET`, `POBLYSH_WEBHOOK_SLACK_SIGNING_SECRET`, and `POBLYSH_WEBHOOK_SLACK_TOLERANCE_SECONDS` (default 300)
- [ ] 1.2 Add verification helpers: `verify_github(body_bytes, header)`, `verify_slack(body_bytes, ts_header, sig_header, tolerance)` using constant‑time compare
- [ ] 1.3 Add public route `POST /webhooks/{provider}/{tenant_id}` that bypasses operator auth when signature is valid
- [ ] 1.4 Wire provider dispatch: choose verification strategy by `provider` slug; reject unsupported providers with 404
- [ ] 1.5 Ensure the handler receives the raw request body for HMAC computation
- [ ] 1.6 Update OpenAPI: document signature headers per provider; mark public path without bearer auth
- [ ] 1.7 Implement structured verification logs and metrics counters/histograms; redact secrets.
- [ ] 1.8 Enforce configurable per-IP and global rate limiting ahead of signature evaluation on the public route; document upstream WAF/CDN integration points.
- [ ] 1.9 Produce operational documentation covering secrets manager integration, rotation cadence (30–90 day with dual-key grace), and encrypted local fallback warnings.

## 2. Validation
- [ ] 2.1 Unit tests for verification helpers (known vectors)
- [ ] 2.2 Integration tests: valid GitHub/Slack signatures → 202; invalid/missing signatures → 401; Slack timestamp too old → 401
- [ ] 2.3 Backward compatibility: operator‑auth path `/webhooks/{provider}` still accepts authenticated requests
