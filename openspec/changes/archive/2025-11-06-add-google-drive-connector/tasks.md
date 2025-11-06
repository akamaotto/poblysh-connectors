## 1. Implementation
- [x] 1.1 Add provider metadata and register `google-drive` in provider registry
- [x] 1.2 Implement `src/connectors/google_drive.rs` with `Connector` trait methods
- [x] 1.3 Wire registry initialization to include Google Drive connector
- [x] 1.4 Map channel webhook payload to normalized Signals (created/updated/trashed/moved)
- [x] 1.5 Implement polling fallback `sync` stub with cursors

## 2. Validation & QA
- [x] 2.1 Run `openspec validate add-google-drive-connector --strict`
- [x] 2.2 Complete proposal QA review and address findings

## 3. Docs
- [x] 3.1 Document required scopes and webhook channel headers in inline code comments

