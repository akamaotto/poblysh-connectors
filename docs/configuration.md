# Configuration System Documentation

This document provides a comprehensive guide to configuration system used by Connectors API service.

## Table of Contents

- [Overview](#overview)
- [Configuration Architecture](#configuration-architecture)
- [Configuration Sources and Precedence](#configuration-sources-and-precedence)
- [Configuration Profiles](#configuration-profiles)
- [Configuration Options Reference](#configuration-options-reference)
- [Configuration File Formats](#configuration-file-formats)
- [Environment Variables](#environment-variables)
- [Command-Line Arguments](#command-line-arguments)
- [Configuration Validation](#configuration-validation)
- [Secret Redaction](#secret-redaction)
- [Setting Up Configuration for Different Environments](#setting-up-configuration-for-different-environments)
- [Security Best Practices](#security-best-practices)

## Overview

The Connectors API uses a flexible, hierarchical configuration system that allows you to:

- Configure application using files, environment variables, or command-line arguments
- Support different environments (local, test, dev, prod) with profile-specific configurations
- Override specific settings without modifying entire configuration
- Securely handle sensitive information with automatic secret redaction
- Validate configuration values before application starts

## Configuration Architecture

The configuration system is built around several key components:

1. **Configuration Structures**: Rust structs that define the configuration schema
2. **Configuration Loader**: Loads and merges configuration from multiple sources
3. **Configuration Validator**: Validates configuration values
4. **Secret Redactor**: Redacts sensitive information for safe logging

The main configuration structure is [`AppConfig`](../src/config/app_config.rs:165), which contains all configuration sections.

## Configuration Sources and Precedence

Configuration is loaded from multiple sources and merged in the following order (highest to lowest precedence):

1. **Command-line arguments**: Directly passed when running the application
2. **Environment variables**: With `POBLYSH_` prefix
3. **Configuration files**: Base `config.{ext}` and profile-specific `config.{profile}.{ext}`
4. **Default values**: Defined in Rust code

This means that command-line arguments will override environment variables, which will override configuration files, and so on.

## Configuration Profiles

The application supports different configuration profiles to adapt to various environments:

- **local**: For local development (default)
- **test**: For automated testing environments
- **dev**: For shared development environments
- **prod**: For production deployments

You can specify the profile in several ways:

1. Set `POBLYSH_PROFILE` environment variable
2. Use `--profile` command-line argument
3. Create a profile-specific configuration file (e.g., `config.prod.toml`)

## Configuration Options Reference

### Server Configuration

Controls HTTP server behavior.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `server.host` | String | `"0.0.0.0"` | Server host address to bind to |
| `server.port` | Integer | `8080` | Server port number (1-65535) |
| `server.workers` | Integer (optional) | `None` | Number of worker threads (default: CPU count) |

### Database Configuration

Controls database connection settings.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `database_url` | String (secret) | `"postgresql://localhost/poblysh"` | Database connection URL |
| `db_max_connections` | Integer | `10` | Maximum number of database connections |
| `db_acquire_timeout_ms` | Integer | `5000` | Connection acquire timeout in milliseconds |

### Logging Configuration

Controls application logging behavior.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `logging.level` | String | `"info"` | Log level: `trace`, `debug`, `info`, `warn`, `error` |
| `logging.format` | String | `"json"` | Log format: `json`, `text` |
| `logging.file` | String (optional) | `None` | Optional log file path |

### Authentication Configuration

Controls JWT authentication settings.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `auth.jwt_secret` | String (secret) | `"default-secret-change-in-production"` | JWT secret key (min 32 chars) |
| `auth.jwt_expiry` | Integer | `3600` | JWT token expiry in seconds (max 30 days) |
| `auth.refresh_token_expiry` | Integer | `86400` | Refresh token expiry in seconds (max 1 year) |

### API Configuration

Controls API behavior and limits.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `api.rate_limit` | Integer (optional) | `100` | Rate limit requests per minute |
| `api.cors_origins` | Array of Strings | `["http://localhost:3000"]` | CORS allowed origins |
| `api.request_timeout` | Integer | `30` | Request timeout in seconds (max 1 hour) |

## Configuration File Formats

The configuration system supports three file formats:

### TOML Format (Recommended)

```toml
# config.toml
profile = "local"

[server]
host = "0.0.0.0"
port = 8080
workers = 4

[database]
url = "postgresql://localhost/poblysh"
max_connections = 10
acquire_timeout_ms = 5000

[logging]
level = "info"
format = "json"
file = "/var/log/connectors.log"

[auth]
jwt_secret = "your-secret-key-here"
jwt_expiry = 3600
refresh_token_expiry = 86400

[api]
rate_limit = 100
cors_origins = ["http://localhost:3000"]
request_timeout = 30
```

### JSON Format

```json
{
  "profile": "local",
  "server": {
    "host": "0.0.0.0",
    "port": 8080,
    "workers": 4
  },
  "database": {
    "url": "postgresql://localhost/poblysh",
    "max_connections": 10,
    "acquire_timeout_ms": 5000
  },
  "logging": {
    "level": "info",
    "format": "json",
    "file": "/var/log/connectors.log"
  },
  "auth": {
    "jwt_secret": "your-secret-key-here",
    "jwt_expiry": 3600,
    "refresh_token_expiry": 86400
  },
  "api": {
    "rate_limit": 100,
    "cors_origins": ["http://localhost:3000"],
    "request_timeout": 30
  }
}
```

### YAML Format

```yaml
# config.yaml
profile: local

server:
  host: "0.0.0.0"
  port: 8080
  workers: 4

database:
  url: "postgresql://localhost/poblysh"
  max_connections: 10
  acquire_timeout_ms: 5000

logging:
  level: "info"
  format: "json"
  file: "/var/log/connectors.log"

auth:
  jwt_secret: "your-secret-key-here"
  jwt_expiry: 3600
  refresh_token_expiry: 86400

api:
  rate_limit: 100
  cors_origins:
    - "http://localhost:3000"
  request_timeout: 30
```

## Environment Variables

You can override any configuration option using environment variables with the `POBLYSH_` prefix. Use double underscores (`__`) to denote nested fields.

### Examples

```bash
# Server configuration
export POBLYSH_SERVER__HOST=127.0.0.1
export POBLYSH_SERVER__PORT=3000
export POBLYSH_SERVER__WORKERS=8

# Database configuration
export POBLYSH_DATABASE_URL=postgresql://user:pass@localhost/mydb
export POBLYSH_DB_MAX_CONNECTIONS=20
export POBLYSH_DB_ACQUIRE_TIMEOUT_MS=2000

# Logging configuration
export POBLYSH_LOGGING__LEVEL=debug
export POBLYSH_LOGGING__FORMAT=text
export POBLYSH_LOGGING__FILE=/tmp/connectors.log

# Authentication configuration
export POBLYSH_AUTH__JWT_SECRET=your-production-secret
export POBLYSH_AUTH__JWT_EXPIRY=7200

# API configuration
export POBLYSH_API__RATE_LIMIT=200
export POBLYSH_API__REQUEST_TIMEOUT=60

# Profile
export POBLYSH_PROFILE=prod

### Jira Connector Environment Variables

The Jira connector reads its OAuth and webhook settings from the following variables (either plain or prefixed with `POBLYSH_`):

- `JIRA_CLIENT_ID` / `POBLYSH_JIRA_CLIENT_ID` (**required outside `local`/`test`**): Atlassian OAuth client identifier.
- `JIRA_CLIENT_SECRET` / `POBLYSH_JIRA_CLIENT_SECRET` (**required outside `local`/`test`**): Atlassian OAuth client secret.
- `JIRA_OAUTH_BASE` / `POBLYSH_JIRA_OAUTH_BASE` (optional): Overrides the OAuth authorize and token base URL. Defaults to `https://auth.atlassian.com`.
- `JIRA_API_BASE` / `POBLYSH_JIRA_API_BASE` (optional): Overrides the Atlassian REST API host. Defaults to `https://api.atlassian.com`.
- `WEBHOOK_JIRA_SECRET` / `POBLYSH_WEBHOOK_JIRA_SECRET` (optional): Shared secret used to protect public Jira webhooks. When unset, public webhook routes skip verification in `local`/`test` profiles; the protected operator route still accepts events. Requests must present the secret via the `Authorization: Bearer <secret>` header.

Note: The loader no longer injects placeholder Jira credentials for any profile. Set `JIRA_CLIENT_ID` and `JIRA_CLIENT_SECRET` explicitly when the Jira connector is enabled.
```

## Command-Line Arguments

You can also override configuration options using command-line arguments:

```bash
# Profile
cargo run -- --profile prod

# Server configuration
cargo run -- --host 127.0.0.1 --port 3000

# Database configuration
cargo run -- --database-url postgresql://user:pass@localhost/mydb
cargo run -- --db-max-connections 20
cargo run -- --db-acquire-timeout-ms 2000

# Logging configuration
cargo run -- --log-level debug

# Authentication configuration
cargo run -- --jwt-secret your-production-secret
```

## Configuration Validation

The configuration system validates all configuration values before the application starts. If any validation errors are found, the application will display all errors and exit.

### Validation Rules

- **Server Configuration**:
  - Port must be between 1 and 65535
  - Host cannot be empty
  - Workers (if specified) cannot be 0

- **Database Configuration**:
  - Database URL must be a valid URL
  - Max connections cannot be 0
  - Min connections cannot be greater than max connections

- **Logging Configuration**:
  - Log level must be one of: `trace`, `debug`, `info`, `warn`, `error`
  - Log format must be one of: `json`, `text`

- **Authentication Configuration**:
  - JWT secret must be at least 32 characters long
  - JWT expiry cannot be 0 or exceed 30 days
  - Refresh token expiry cannot be 0 or exceed 1 year
  - Refresh token expiry must be greater than or equal to JWT expiry

- **API Configuration**:
  - Rate limit (if specified) cannot be 0
  - CORS origins cannot be empty strings
  - Request timeout cannot be 0 or exceed 1 hour

- **Profile**:
  - Must be one of: `local`, `test`, `dev`, `prod`

## Secret Redaction

The configuration system automatically redacts sensitive information when logging configuration values. This prevents accidental exposure of sensitive data in logs.

### Redacted Fields

The following fields are automatically redacted:
- `database.url`
- `auth.jwt_secret`
- Any field containing "password", "secret", "token", "key", "auth", "credential", or "private"

### Example

When logging configuration, sensitive values are replaced with `[REDACTED]`:

```json
{
  "profile": "prod",
  "server": {
    "host": "0.0.0.0",
    "port": 8080
  },
  "database": {
    "url": "[REDACTED]",
    "max_connections": 20,
    "min_connections": 5
  },
  "auth": {
    "jwt_secret": "[REDACTED]",
    "jwt_expiry": 3600,
    "refresh_token_expiry": 86400
  }
}
```

## Setting Up Configuration for Different Environments

### Local Development

For local development, create a `config.local.toml` file:

```toml
# config.local.toml
profile = "local"

[server]
host = "127.0.0.1"
port = 8080

[database]
url = "postgresql://localhost/connectors_dev"
max_connections = 5
min_connections = 1

[logging]
level = "debug"
format = "text"

[auth]
jwt_secret = "local-dev-secret-key-for-testing-only"
jwt_expiry = 3600
refresh_token_expiry = 86400

[api]
cors_origins = ["http://localhost:3000", "http://localhost:8080"]
```

### Testing Environment

For testing, create a `config.test.toml` file:

```toml
# config.test.toml
profile = "test"

[server]
host = "127.0.0.1"
port = 0  # Let OS choose a random port

[database]
url = "postgresql://localhost/connectors_test"
max_connections = 2
min_connections = 1

[logging]
level = "warn"
format = "json"

[auth]
jwt_secret = "test-secret-key-for-testing-only"
jwt_expiry = 60  # Short expiry for tests
refresh_token_expiry = 300

[api]
rate_limit = 1000  # Higher limit for tests
request_timeout = 5
```

### Production Environment

For production, use environment variables or a `config.prod.toml` file:

```toml
# config.prod.toml
profile = "prod"

[server]
host = "0.0.0.0"
port = 8080
workers = 8

[database]
url = "postgresql://user:password@db.example.com:5432/connectors_prod"
max_connections = 20
min_connections = 5

[logging]
level = "info"
format = "json"
file = "/var/log/connectors/app.log"

[auth]
jwt_secret = "production-secret-key-must-be-at-least-32-chars"
jwt_expiry = 3600
refresh_token_expiry = 604800  # 7 days

[api]
rate_limit = 100
cors_origins = ["https://app.example.com"]
request_timeout = 30
```

Or using environment variables:

```bash
export POBLYSH_PROFILE=prod
export POBLYSH_DATABASE__URL=postgresql://user:password@db.example.com:5432/connectors_prod
export POBLYSH_AUTH__JWT_SECRET=production-secret-key-must-be-at-least-32-chars
export POBLYSH_LOGGING__FILE=/var/log/connectors/app.log
```

## Security Best Practices

1. **Use Strong Secrets**: Ensure all secrets (JWT secret, database passwords) are strong and unique.

2. **Environment Variables for Secrets**: Store sensitive values in environment variables rather than configuration files, especially in production.

3. **File Permissions**: Ensure configuration files containing secrets have appropriate file permissions (e.g., `600`).

4. **Secret Management**: Consider using a secret management system (like HashiCorp Vault, AWS Secrets Manager) in production.

5. **Profile-Specific Configurations**: Use profile-specific configurations to ensure production settings don't accidentally use development values.

6. **Regular Rotation**: Regularly rotate secrets, especially JWT secrets.

7. **Audit Configuration**: Regularly audit configuration files and environment variables to ensure no sensitive information is accidentally exposed.

8. **Version Control**: Never commit configuration files containing secrets to version control. Use example files instead.
