# connectors Specification

## Purpose
TBD - created by archiving change add-connector-trait-and-registry. Update Purpose after archive.
## Requirements
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

### Requirement: Provider Metadata Structure
The system SHALL define a provider metadata structure for discovery and documentation.

#### Scenario: Metadata fields
- **WHEN** metadata is retrieved for a provider
- **THEN** it includes `name` (string), `auth_type` (enum string), `scopes` (string array), and `webhooks` (boolean)

### Requirement: In-memory Provider Registry
The system SHALL provide a read-only, in-memory registry mapping provider `name -> { connector, metadata }`.

#### Scenario: Resolve connector by name
- **WHEN** `get(name)` is called for a known provider
- **THEN** the registry returns a handle to the connector instance

#### Scenario: Unknown provider returns error
- **WHEN** `get(name)` is called for an unknown provider
- **THEN** the registry returns a typed error indicating unknown provider

#### Scenario: List provider metadata
- **WHEN** `list_metadata()` is called
- **THEN** the registry returns a list of metadata entries sorted by `name` ascending

### Requirement: Seed Registry With Stub Connector
The system SHALL include at least one stub connector registered to validate wiring.

#### Scenario: Stub connector registration
- **WHEN** the system initializes the registry
- **THEN** at least one provider (e.g., `example`) exists with non-empty metadata and a no-op connector implementation

