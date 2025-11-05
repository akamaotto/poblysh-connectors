## MODIFIED Requirements
### Requirement: Connector Trait
The `sync` method signature and semantics SHALL support cursor advancement and pagination.

#### Scenario: Sync returns next cursor and has_more
- **WHEN** `sync(connection, cursor?)` is invoked
- **THEN** it returns a `SyncResult` object with fields:
  - `signals: [Signal]` (normalized events)
  - `next_cursor?: Cursor` (opaque provider cursor for the next call)
  - `has_more: bool` (true if additional pages remain)

#### Scenario: Cursor is opaque and serializable
- **WHEN** `next_cursor` is provided
- **THEN** it is an opaque value serializable to JSON and safe to store under `connections.metadata.sync.cursor`
