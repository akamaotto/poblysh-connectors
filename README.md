# Connectors API

A Rust-based API service for managing connectors, built with Axum and featuring OpenAPI documentation.

# Business Use Case

This project emerged from the Poblysh PRD framework (`plan/prd.md`) and the storytelling thesis documented in `plan/signals.md`. The goal is to ingest activity from SaaS tools (GitHub, Jira, Google, Zoho, etc.), normalize that activity into reusable Signals, encrypt tokens for each tenant, and surface those Signals to downstream Story Hunter workflows. In short, the Connectors API turns fragmented operational events into publishable stories, keeping PR/comm teams proactive without adding manual monitoring work. The README below focuses on the technical UX (local tooling, OpenAPI, Docker), but consult `plan/prd.md` and `plan/signals.md` for the product motivations, success metrics, and signal → grounded signal → Idea pipeline that the service implements.

## Prerequisites

- Rust toolchain (latest stable version recommended)

## Local Development

### Quick Start

For local development, the project includes `Makefile` and `Justfile` with consistent targets. The recommended workflow uses SQLite by default and requires no external database setup.

```bash
# Using make (recommended)
make env && make db-sqlite && make migrate && make run

# Or using just
just env && just db-sqlite && just migrate && just run
```

### Available Commands

Both `make` and `just` support the same set of targets:

| Command | Description |
|---------|-------------|
| `make help` / `just` | Show all available commands |
| `make setup` / `just setup` | Check core prerequisites and suggest optional tools |
| `make env` / `just env` | Create/update `.env.local` with SQLite defaults |
| `make db-sqlite` / `just db-sqlite` | Ensure local SQLite database file exists |
| `make db-pg-check` / `just db-pg-check` | Optional Postgres connectivity check |
| `make migrate` / `just migrate` | Run database migrations (idempotent) |
| `make run` / `just run` | Start the API |
| `make watch` / `just watch` | Start dev loop with hot reload (requires cargo-watch) |
| `make test` / `just test` | Run tests |
| `make lint` / `just lint` | Run clippy with -D warnings |
| `make fmt` / `just fmt` | Format code |
| `make openapi` / `just openapi` | Export OpenAPI spec to `openapi.json` |
| `make smoke` / `just smoke` | Run E2E smoke tests against the real binary |

### Environment Configuration

The `make env` / `just env` command creates `.env.local` with sensible defaults:

- `POBLYSH_PROFILE=local`
- `POBLYSH_DATABASE_URL=sqlite://dev.db`
- `POBLYSH_OPERATOR_TOKEN=local-dev-token`
- `POBLYSH_CRYPTO_KEY=<generated-32-byte-base64-key>`

### Optional: PostgreSQL Setup

While SQLite is the default for local development, you can optionally use PostgreSQL:

```bash
# Start PostgreSQL container
docker run --name postgres-dev \
  -e POSTGRES_DB=poblysh \
  -e POSTGRES_USER=postgres \
  -e POSTGRES_PASSWORD=postgres \
  -p 5432:5432 \
  -d postgres:15

# Override the database URL in .env.local
echo "POBLYSH_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/poblysh" >> .env.local

# Test Postgres connectivity (optional)
make db-pg-check
```

### Tooling Notes

- **cargo-watch**: Install with `cargo install cargo-watch` for the `watch` command
- **curl**: Required for `openapi` command
- **jq**: Recommended for validating JSON output in `openapi`
- **just**: Optional task runner (https://just.systems)

The scripts handle missing optional tools gracefully with clear installation guidance.

### E2E Smoke Tests

The project includes comprehensive end-to-end smoke tests that validate the real binary startup, database connectivity, and core HTTP endpoints.

**Prerequisites:**
- `POBLYSH_DATABASE_URL` set (PostgreSQL or SQLite)
- `POBLYSH_OPERATOR_TOKEN` set for protected endpoint testing

**Running Smoke Tests:**

```bash
# Using make
make smoke

# Using just
just smoke

# Or with cargo directly
POBLYSH_DATABASE_URL=sqlite://dev.db \
POBLYSH_OPERATOR_TOKEN=local-dev-token \
cargo test --test e2e_smoke_tests -- --test-threads=1
```

**What the smoke tests validate:**
- Binary startup and configuration loading
- Database connectivity and migrations
- Readiness endpoint (`/readyz`) with timeout handling
- Core public endpoints: `/`, `/healthz`, `/readyz`, `/openapi.json`, `/providers`
- Protected endpoint (`/protected/ping`) with authentication and tenant headers
- Graceful shutdown and cleanup

**Database Options:**
- **PostgreSQL (recommended):** `postgresql://postgres:postgres@localhost:5432/poblysh`
- **SQLite (development):** `sqlite://dev.db`

**Docker PostgreSQL for smoke tests:**
```bash
docker run --name postgres-dev \
  -e POSTGRES_DB=poblysh \
  -e POSTGRES_USER=postgres \
  -e POSTGRES_PASSWORD=postgres \
  -p 5432:5432 \
  -d postgres:15

# Then run smoke tests
export POBLYSH_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/poblysh
export POBLYSH_OPERATOR_TOKEN=local-dev-token
make smoke
```

The smoke tests use `assert_cmd` for reliable binary path resolution and `portpicker` for deterministic port selection, providing fast and reliable validation of the complete service stack.

## Docker Build & Swagger UI Testing

To validate the dockerized workflow and manually exercise the Swagger UI surface, follow these steps every time you ship the `add-dockerization-post-mvp` change or a follow-up.

### 1. Build the Docker image

```bash
docker build --pull -t connectors:local .
```

The repository’s `Dockerfile` uses a multi-stage build (Rust builder + slim runtime). The `--pull` flag ensures the base images are refreshed before building.

### 2. Run the container locally

```bash
docker run --rm -p 8080:8080 \
  -e POBLYSH_PROFILE=local \
  -e POBLYSH_DATABASE_URL=sqlite://dev.db \
  -e POBLYSH_OPERATOR_TOKEN=local-dev-token \
  -e POBLYSH_CRYPTO_KEY="$(openssl rand -base64 32)" \
  -v "$PWD/dev.db:/app/dev.db" \
  connectors:local
```

Adjust `POBLYSH_DATABASE_URL` to point at a Postgres instance when you need to test migrations (`postgresql://pbl:secret@localhost:5432/connectors` for local containers). The bind mount keeps the SQLite file in sync with your host so you can inspect migrations after the run.

Watch the logs for a message confirming `/readyz` is passing before moving on to the Swagger UI.

### 3. Manual Swagger UI checklist

Once the container is running on `http://localhost:8080`, open `http://localhost:8080/docs` and run the following manual checks in Swagger UI:

- Use the **Authorize** button to paste `Bearer <your operator token>` so protected endpoints accept your requests.
- Execute `GET /healthz`, `/readyz`, `/providers`, `/` and `/openapi.json` via **Try it out**; verify each returns 200 and meaningful payloads.
- Use `POST /protected/ping` with a generated tenant UUID (e.g., `uuidgen`) and confirm a successful response that proves auth and tenant headers work.
- Submit a sample webhook via `POST /webhooks/github/{tenant_id}` with a fixture payload (see `tests/fixtures/normalization/github`) and ensure Swagger reports success while the container logs show signal normalization activity.
- Download `/openapi.json` from the UI and run `curl -fsS http://localhost:8080/openapi.json | jq .info.title` to confirm the spec matches expectations.
- Keep an eye on the container logs (`docker logs <container>`) for startup/migration errors or schema validation defects while you interact with the UI.

When you’re finished, stop the container with `Ctrl+C` and mention the steps you ran in PR descriptions so reviewers know the Docker and Swagger workflow has been validated.

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
- `POBLYSH_CRYPTO_KEY` – base64-encoded 32 byte key used to encrypt access/refresh tokens (required). See [Crypto Key Rotation Guide](docs/runbooks/local-crypto-rotation.md) for rotation procedures.

### Mail Spam Filtering

The service includes a centralized spam filtering system for mail connectors. Configure spam filtering with these environment variables:

- `POBLYSH_MAIL_SPAM_THRESHOLD` – Spam confidence threshold (0.0-1.0, default: 0.8). Messages scoring >= threshold are blocked
- `POBLYSH_MAIL_SPAM_ALLOWLIST` – Comma-separated trusted emails/domains that are never marked as spam
- `POBLYSH_MAIL_SPAM_DENYLIST` – Comma-separated blocked emails/domains that are always marked as spam

**Spam Filtering Examples:**

```bash
# Default configuration (moderate filtering)
POBLYSH_MAIL_SPAM_THRESHOLD=0.8

# Aggressive filtering - block more potential spam
POBLYSH_MAIL_SPAM_THRESHOLD=0.5

# Lenient filtering - allow more messages through
POBLYSH_MAIL_SPAM_THRESHOLD=0.9

# Trust specific domains and emails
POBLYSH_MAIL_SPAM_ALLOWLIST="@trustedcompany.com,user@important.com"

# Block known spam domains
POBLYSH_MAIL_SPAM_DENYLIST="@spammydomain.com,blocked@badactor.net"

# Combined configuration
POBLYSH_MAIL_SPAM_THRESHOLD=0.6 \
POBLYSH_MAIL_SPAM_ALLOWLIST="@mycompany.com" \
POBLYSH_MAIL_SPAM_DENYLIST="@spam.com"
```

**Spam Detection Logic:**

- **Provider Labels**: Respects Gmail labels (SPAM, TRASH, PROMOTIONS, etc.)
- **Subject Analysis**: Detects urgency words, financial offers, phishing attempts
- **Attachment Heuristics**: Identifies suspicious file types (.exe, .bat, .scr, etc.)
- **Allowlist Override**: Allowlisted senders always bypass spam filtering
- **Denylist Override**: Denylisted senders are always marked as spam

**Telemetry:** Spam decisions are logged with structured telemetry including provider, message ID, spam score, and decision reason.

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
- `POBLYSH_CRYPTO_KEY`: Base64 string that decodes to 32 bytes; required to encrypt/decrypt stored tokens. Generate with `openssl rand -base64 32`. See [Crypto Key Rotation Guide](docs/runbooks/local-crypto-rotation.md) for rotation procedures.

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
