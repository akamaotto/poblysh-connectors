## ADDED Requirements

### Requirement: Logging Format Configuration
The system SHALL support a `POBLYSH_LOG_FORMAT` variable to control log output format.

Details (MVP):
- Accepted values: `json` (default) and `pretty`
- Unknown values MUST fall back to `json`

#### Scenario: Default format is JSON
- WHEN `POBLYSH_LOG_FORMAT` is unset
- THEN logs are emitted in JSON format

#### Scenario: Pretty format selected
- GIVEN `POBLYSH_LOG_FORMAT=pretty`
- WHEN the service starts
- THEN logs are emitted in a human-readable text format

