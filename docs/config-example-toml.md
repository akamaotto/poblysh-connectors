# Updated config.example.toml

This file contains the updated content for `config.example.toml` with all available configuration options and detailed comments.

```toml
# Example configuration file for the Connectors API service
# This file demonstrates the configuration structure and default values
# Copy this file to config.toml and modify as needed

# Profile name (local, test, dev, prod)
# Determines which configuration profile to use
# Can be overridden with POBLYSH_PROFILE environment variable
profile = "local"

[server]
# Server host address to bind to
# Use "0.0.0.0" to bind to all interfaces (default)
# Use "127.0.0.1" to bind only to localhost
host = "0.0.0.0"

# Server port number (1-65535)
# Port 0 is not allowed
port = 8080

# Number of worker threads (optional)
# If not specified, defaults to the number of CPU cores
# Must be greater than 0 if specified
# workers = 4

[database]
# Database connection URL (sensitive)
# Should be kept secret and not committed to version control
# Format: postgresql://[user[:password]@][host][:port][/dbname][?param1=value1&...]
# Can be overridden with POBLYSH_DATABASE__URL environment variable
url = "postgresql://localhost/connectors"

# Maximum number of database connections in the connection pool
# Must be greater than 0
# Should be tuned based on your database capacity and application load
max_connections = 10

# Minimum number of database connections to maintain in the pool
# Must be less than or equal to max_connections
# Helps maintain a baseline of connections for better performance
min_connections = 1

[logging]
# Log level (trace, debug, info, warn, error)
# Controls the verbosity of log messages
# trace: Most verbose, includes all log messages
# debug: Debug information for development
# info: General information about application operation (default)
# warn: Warning messages for potentially problematic situations
# error: Error messages only
level = "info"

# Log format (json, text)
# Controls how log messages are formatted
# json: Structured JSON logs, ideal for log aggregation systems (default)
# text: Human-readable text logs, ideal for development
format = "json"

# Optional log file path
# If specified, logs will be written to this file
# If not specified, logs will be written to stdout/stderr
# Ensure the application has write permissions to the directory
# file = "/var/log/connectors.log"

[auth]
# JWT secret key (sensitive)
# Must be at least 32 characters long
# Should be a strong, random string in production
# Can be overridden with POBLYSH_AUTH__JWT_SECRET environment variable
# IMPORTANT: Change this in production!
jwt_secret = "default-secret-change-in-production"

# JWT token expiry in seconds
# Controls how long JWT access tokens are valid
# Must be greater than 0 and less than or equal to 30 days (2592000 seconds)
# Default: 3600 seconds (1 hour)
jwt_expiry = 3600

# Refresh token expiry in seconds
# Controls how long refresh tokens are valid
# Must be greater than 0, greater than or equal to jwt_expiry, and less than or equal to 1 year (31536000 seconds)
# Default: 86400 seconds (24 hours)
refresh_token_expiry = 86400

[api]
# Rate limit requests per minute (optional)
# Controls how many requests a client can make per minute
# If not specified, no rate limiting is applied
# Must be greater than 0 if specified
rate_limit = 100

# CORS allowed origins
# List of origins that are allowed to make cross-origin requests
# Use "*" to allow all origins (not recommended for production)
# Include the protocol (http:// or https://)
cors_origins = ["http://localhost:3000"]

# Request timeout in seconds
# Controls how long the server waits for a request to complete
# Must be greater than 0 and less than or equal to 1 hour (3600 seconds)
# Default: 30 seconds
request_timeout = 30
```

## Profile-Specific Configuration Examples

### Local Development (config.local.toml)

```toml
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

### Testing Environment (config.test.toml)

```toml
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

### Production Environment (config.prod.toml)

```toml
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

## Environment Variable Overrides

All configuration options can be overridden using environment variables with the `POBLYSH_` prefix. Use double underscores (`__`) to denote nested fields.

Examples:
```bash
# Server configuration
export POBLYSH_SERVER__HOST=127.0.0.1
export POBLYSH_SERVER__PORT=3000

# Database configuration
export POBLYSH_DATABASE__URL=postgresql://user:pass@localhost/mydb
export POBLYSH_DATABASE__MAX_CONNECTIONS=20

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
```

## Command-Line Arguments

Configuration options can also be overridden using command-line arguments:

```bash
# Profile
cargo run -- --profile prod

# Server configuration
cargo run -- --host 127.0.0.1 --port 3000

# Database configuration
cargo run -- --database-url postgresql://user:pass@localhost/mydb

# Logging configuration
cargo run -- --log-level debug

# Authentication configuration
cargo run -- --jwt-secret your-production-secret