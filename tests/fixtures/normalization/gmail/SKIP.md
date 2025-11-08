# Gmail Normalization

## Rationale
Gmail normalization currently happens inside the connector while processing
History API records. The harness requires a lightweight adapter that can feed
synthetic history entries through that logic before fixtures are meaningful.

## Plan
Expose a `normalize_gmail_history_record` helper, add fixtures covering
`email_updated` and `email_deleted`, and wire the harness to exercise it.
