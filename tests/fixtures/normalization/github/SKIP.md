# GitHub Normalization

## Rationale
Connector mappings exist for GitHub webhooks, but the normalization harness
needs additional work to replay rich payloads (issues, pull requests, reviews)
before we can add stable fixtures.

## Plan
Extract the webhook mapping logic into `normalize_github_webhook_kind`, capture
fixture payloads for `issue_created`, `issue_closed`, `pr_opened`, `pr_closed`,
`pr_merged`, `issue_comment`, and `pr_review`, and re-enable coverage.
