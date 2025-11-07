## 1. Implementation
- [x] 1.1 Add provider metadata and register `jira` in provider registry
- [x] 1.2 Implement `src/connectors/jira.rs` with `Connector` trait methods
- [x] 1.3 Wire OAuth start/callback to call `authorize`/`exchange_token` for `jira` (reuses existing endpoints)
- [x] 1.4 Add provider seeding for `jira`
- [x] 1.5 Map Jira issue events to normalized Signals (`issue_created`, `issue_updated`)
- [x] 1.6 Implement incremental sync stub using `cursor` as updated‑since filter
- [x] 1.7 Add unit tests for webhook mapping and authorize URL shape

## 2. Observability & Docs
- [x] 2.1 Add tracing logs around webhook and sync
- [x] 2.2 Document required env vars and manual webhook configuration for local

## 3. Validation
- [x] 3.1 Run `openspec validate add-jira-connector --strict`
- [x] 3.2 Ensure QA review items (if any) are resolved
