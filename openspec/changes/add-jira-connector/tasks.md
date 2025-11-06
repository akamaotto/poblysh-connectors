## 1. Implementation
- [ ] 1.1 Add provider metadata and register `jira` in provider registry
- [ ] 1.2 Implement `src/connectors/jira.rs` with `Connector` trait methods
- [ ] 1.3 Wire OAuth start/callback to call `authorize`/`exchange_token` for `jira` (reuses existing endpoints)
- [ ] 1.4 Add provider seeding for `jira`
- [ ] 1.5 Map Jira issue events to normalized Signals (`issue_created`, `issue_updated`)
- [ ] 1.6 Implement incremental sync stub using `cursor` as updated‑since filter
- [ ] 1.7 Add unit tests for webhook mapping and authorize URL shape

## 2. Observability & Docs
- [ ] 2.1 Add tracing logs around webhook and sync
- [ ] 2.2 Document required env vars and manual webhook configuration for local

## 3. Validation
- [ ] 3.1 Run `openspec validate add-jira-connector --strict`
- [ ] 3.2 Ensure QA review items (if any) are resolved

