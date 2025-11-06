## 1. Implementation
- [ ] 1.1 Add provider metadata and register `google-calendar` in provider registry
- [ ] 1.2 Implement `src/connectors/google_calendar.rs` with `Connector` trait methods
- [ ] 1.3 Wire registry initialization to include Google Calendar connector
- [ ] 1.4 Implement webhook handler to accept channel headers and return no-op signals
- [ ] 1.5 Implement incremental `sync` using `events.list` with `syncToken` (stub acceptable for MVP)

## 2. Validation & QA
- [ ] 2.1 Run `openspec validate add-google-calendar-connector --strict`
- [ ] 2.2 Complete proposal QA review and address findings

## 3. Docs
- [ ] 3.1 Document required scopes and webhook channel headers in inline code comments

