# Google Drive Normalization

## Rationale
Drive webhook normalization depends on headers forwarded by the platform.
The harness needs synthetic payloads that include those headers and a shared
helper before fixtures can assert the mapping for `file_created`, `file_updated`,
`file_deleted`, and `file_moved`.

## Plan
Create a normalization helper that inspects `x-goog-resource-state`, add fixtures
for each state, and then enforce coverage in the harness.
