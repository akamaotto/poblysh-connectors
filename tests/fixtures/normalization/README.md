# Normalization Test Fixtures

This directory contains golden test fixtures for Signal.kind normalization. These fixtures ensure that provider events are correctly mapped to the canonical taxonomy and prevent schema drift over time.

## Directory Structure

```
tests/fixtures/normalization/
├── README.md                    # This file
├── example/                     # Example provider fixtures
│   ├── issue_created.json
│   ├── pr_merged.json
│   └── message_posted.json
├── jira/                        # Jira webhook fixtures
├── zoho-cliq/                   # Zoho Cliq webhook fixtures
├── github/                      # SKIP.md until fixtures land
├── gmail/
├── google-calendar/
├── google-drive/
├── zoho-mail/
└── ...                          # Other providers
```

## Fixture Format

Each fixture is a JSON file with the following structure:

```json
{
  "provider": "provider_name",
  "name": "short_case_name",
  "input": { /* provider payload */ },
  "expected": { "kind": "<normalized_kind>" }
}
```

### Required Fields

- **provider** (string): The provider identifier (e.g., "github", "jira", "slack")
- **name** (string): Human-readable case name, used for test output
- **input** (object): Raw provider payload that will be normalized
- **expected.kind** (string): Expected normalized Signal.kind value

### Naming Conventions

- Use `snake_case` for file names
- File names should match the expected `Signal.kind` value
- Keep names concise but descriptive (e.g., `issue_created.json`, `pr_merged.json`)

## Fixture Authoring Rules

### 1. Minimal Payloads
Include only the fields necessary to drive the normalization mapping:
- ✅ Include fields that determine the event type
- ✅ Include a few invariant fields for context
- ❌ Omit volatile identifiers (IDs, timestamps) unless required
- ❌ Omit unrelated metadata

**Example:**
```json
{
  "input": {
    "action": "created",
    "title": "Add authentication support"
  }
}
```

### 2. Stable Data
- Use generic/placeholder values instead of real identifiers
- Avoid real user names, emails, or sensitive data
- Use consistent example values across fixtures

### 3. Provider-Agnostic Mapping
Fixtures should test the mapping to canonical kinds, not provider-specific events:
- ✅ Test that GitHub `issues.opened` → `issue_created`
- ✅ Test that Jira `issue_created` → `issue_created`
- ❌ Don't test provider-specific event names directly

## Coverage Requirements

Each integrated provider MUST include at least one fixture per `Signal.kind` it can emit.
The current roster enforced by the harness is:

```
example, github, gmail, google-calendar, google-drive, jira, zoho-cliq, zoho-mail
```

Providers without fixtures MUST include `SKIP.md` explaining why coverage is
temporarily skipped.

### Provider Coverage

For each provider directory:
1. **Complete Coverage**: Include fixtures for all emitted kinds
2. **Temporary Skip**: If not yet implemented, create a `SKIP.md` file
3. **Missing Coverage**: Tests will fail if no fixtures and no `SKIP.md`

### SKIP.md Format

If a provider cannot be normalized yet, create `SKIP.md` with:
```
# Provider Name Normalization

## Rationale
Brief explanation of why normalization is not yet implemented.

## Plan
[optional] Timeline or requirements for implementation.
```

## Canonical Signal.kind Registry

The following kinds are the only allowed values (must match exactly):

### Issue Kinds
- `issue_created`
- `issue_updated`
- `issue_closed`
- `issue_reopened`
- `issue_resolved`
- `issue_comment`

### Pull Request Kinds
- `pr_opened`
- `pr_closed`
- `pr_merged`
- `pr_reopened`
- `pr_updated`
- `pr_review`

### Code Kinds
- `code_pushed`
- `release_published`

### Message Kinds
- `message_posted`
- `message_updated`
- `message_deleted`
- `reaction_added`

### File Kinds
- `file_created`
- `file_updated`
- `file_deleted`
- `file_moved`

### Calendar Kinds
- `calendar_event_created`
- `calendar_event_updated`
- `calendar_event_deleted`

### Email Kinds
- `email_received`
- `email_sent`
- `email_updated`
- `email_deleted`

## Adding New Kinds

To add a new `Signal.kind`:

1. **Update the normalization spec** with the new kind
2. **Add it to the canonical registry** in the test code
3. **Create at least one fixture** demonstrating the mapping
4. **Submit as part of a proposal** following the OpenSpec workflow

## Running Tests

```bash
# Run all normalization tests
cargo test normalization

# Run specific test
cargo test test_normalization_golden_fixtures

# Run coverage check
cargo test test_fixture_coverage_enforcement
```

## Test Failure Examples

### Missing Required Field
```
Fixture tests/fixtures/normalization/example/broken.json: expected.kind field is required
```

### Non-Canonical Kind
```
Found Signal.kind values not in canonical registry:
  - invalid_kind
To add new kinds, update the normalization spec and registry.
```

### Missing Coverage
```
Provider github has no normalization fixtures and no SKIP.md file.
Add fixtures under tests/fixtures/normalization/github/ or create SKIP.md with rationale.
```

## Examples

### GitHub Issue Created
`tests/fixtures/normalization/github/issue_created.json`
```json
{
  "provider": "github",
  "name": "issue_created",
  "input": {
    "action": "opened",
    "issue": {
      "title": "Add user authentication",
      "number": 42
    }
  },
  "expected": {
    "kind": "issue_created"
  }
}
```

### Slack Message Posted
`tests/fixtures/normalization/slack/message_posted.json`
```json
{
  "provider": "slack",
  "name": "message_posted",
  "input": {
    "type": "message",
    "channel": "C1234567890",
    "text": "Hello team!",
    "user": "U1234567890"
  },
  "expected": {
    "kind": "message_posted"
  }
}
```
