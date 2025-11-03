# Connectors API

A Rust-based API service for managing connectors, built with Axum and featuring OpenAPI documentation.

## Prerequisites

- Rust toolchain (latest stable version recommended)
- PostgreSQL database (for local development)

## Local Run

To run the project locally:

```bash
# Set up database (required for first run)
export POBLYSH_DATABASE_URL="postgresql://username:password@localhost/database_name"

# Run the service
cargo run
```

The server will start on the address specified by the `POBLYSH_API_BIND_ADDR` environment variable (default: `0.0.0.0:8080`).

### Database Setup

For local development, you'll need a PostgreSQL database. The service will automatically run migrations for `local` and `test` profiles.

#### Using Docker (recommended)

```bash
# Start PostgreSQL container
docker run --name postgres-dev \
  -e POSTGRES_DB=poblysh \
  -e POSTGRES_USER=postgres \
  -e POSTGRES_PASSWORD=postgres \
  -p 5432:5432 \
  -d postgres:15

# Set environment variable
export POBLYSH_DATABASE_URL="postgresql://postgres:postgres@localhost:5432/poblysh"
```

## Configuration System

The Connectors API loads configuration from layered `.env` files and environment variables. Precedence (lowest to highest):

1. `.env`
2. `.env.local`
3. `.env.<profile>` (e.g., `.env.test`)
4. `.env.<profile>.local`
5. Process environment variables (`POBLYSH_*`)

Later sources override earlier ones so you can keep defaults in `.env` and personal overrides in `.env.local`.

### Configuration Profiles

Profiles allow you to swap environment presets. Supported values today are `local` (default) and `test`; future profiles such as `dev` and `prod` can be added as needed. Set the profile with the `POBLYSH_PROFILE` environment variable before launching the service.

### Environment Variables

Configuration keys use the `POBLYSH_` prefix. The MVP fields are:
- `POBLYSH_PROFILE` – active profile (`local` by default)
- `POBLYSH_API_BIND_ADDR` – socket address to bind (`0.0.0.0:8080` by default)
- `POBLYSH_LOG_LEVEL` – log verbosity (`info` by default)
- `POBLYSH_DATABASE_URL` – PostgreSQL connection string (required)
- `POBLYSH_DB_MAX_CONNECTIONS` – maximum database connections (default: 10)
- `POBLYSH_DB_ACQUIRE_TIMEOUT_MS` – connection acquire timeout in milliseconds (default: 5000)
- `POBLYSH_CRYPTO_KEY` – base64-encoded 32 byte key used to encrypt access/refresh tokens (required)

Example:
```bash
POBLYSH_PROFILE=test \
POBLYSH_API_BIND_ADDR=127.0.0.1:3000 \
POBLYSH_LOG_LEVEL=debug \
POBLYSH_DATABASE_URL="postgresql://postgres:postgres@localhost:5432/poblysh" \
cargo run
```

Configuration is logged at startup using a redacted JSON representation (no secrets in the current schema).

## Database Migrations

The service uses SeaORM migrations to manage database schema changes.

### Automatic Migrations

For `local` and `test` profiles, migrations run automatically on startup.

### Manual Migrations

For other profiles, use the CLI commands:

```bash
# Apply all pending migrations
cargo run -- migrate up

# Rollback the last migration
cargo run -- migrate down

# Check migration status
cargo run -- migrate status
```

### Token Encryption Backfill

If you enable token encryption on an existing environment, run the helper binary to re-encrypt any legacy plaintext rows:

```bash
cargo run --bin reencrypt_plaintext_tokens
```

## Environment Variables

- `POBLYSH_PROFILE`: Configuration profile to use (default: `local`)
- `POBLYSH_API_BIND_ADDR`: Address and port for the HTTP server (default: `0.0.0.0:8080`)
- `POBLYSH_LOG_LEVEL`: Log verbosity (`trace`, `debug`, `info`, `warn`, `error`; default: `info`)
- `POBLYSH_DATABASE_URL`: PostgreSQL connection string (required)
- `POBLYSH_DB_MAX_CONNECTIONS`: Maximum database connections (default: 10)
- `POBLYSH_DB_ACQUIRE_TIMEOUT_MS`: Connection acquire timeout in milliseconds (default: 5000)
- `POBLYSH_CRYPTO_KEY`: Base64 string that decodes to 32 bytes; required to encrypt/decrypt stored tokens. Generate with `openssl rand -base64 32`.

Examples:
```bash
# Set profile
POBLYSH_PROFILE=prod cargo run

# Set API bind address
POBLYSH_API_BIND_ADDR=127.0.0.1:3000 cargo run

# Set database URL
POBLYSH_DATABASE_URL="postgresql://user:pass@host:5432/db" cargo run

# Set all
POBLYSH_PROFILE=prod \
POBLYSH_API_BIND_ADDR=127.0.0.1:3000 \
POBLYSH_DATABASE_URL="postgresql://user:pass@host:5432/db" \
cargo run
```

## Available Endpoints

- `/` - Root endpoint that returns basic service information
- `/docs` - Swagger UI for interactive API documentation
- `/openapi.json` - OpenAPI specification in JSON format
