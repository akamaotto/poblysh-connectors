# Justfile for Poblysh Connectors
#
# Implements local developer run scripts as specified in:
# - openspec/changes/add-local-run-scripts/proposal.md
# - openspec/changes/add-local-run-scripts/design.md
# - openspec/changes/add-local-run-scripts/specs/dev-tooling/spec.md
#
# Primary goals:
# - Zero-docker local development
# - Fast, deterministic setup and run flows
# - No new runtime dependencies
# - Graceful handling of optional tools
#
# Usage:
# - `just`                  # show help
# - `just env`              # scaffold .env.local
# - `just db-sqlite`        # ensure dev.db exists
# - `just migrate`          # run migrations
# - `just run`              # start API
# - `just watch`            # dev loop (if cargo-watch present)
# - `just test`             # run tests
# - `just lint`             # run clippy
# - `just fmt`              # run fmt
# - `just openapi`          # export OpenAPI if server running

set shell := ["sh", "-cu"]

# Shared variables
default_port := "8080"
openapi_url := "http://localhost:8080/openapi.json"
env_file := ".env.local"
sqlite_db := "dev.db"

# Default: show help
default:
    @just help

help:
    @printf "Poblysh Connectors - Local Dev Tasks\n\n"
    @printf "Primary targets:\n"
    @printf "  help         Show this help overview\n"
    @printf "  setup        Check core prerequisites and suggest optional tools\n"
    @printf "  env          Scaffold/update .env.local for local profile\n"
    @printf "  db-sqlite    Ensure local SQLite database file exists (for sqlite://dev.db)\n"
    @printf "  db-pg-check  Optional Postgres connectivity check (non-blocking for SQLite flow)\n"
    @printf "  migrate      Run database migrations (idempotent)\n"
    @printf "  run          Start the API using cargo run\n"
    @printf "  watch        Start dev loop (uses cargo-watch when available, else falls back)\n"
    @printf "  test         Run tests\n"
    @printf "  lint         Run clippy with -D warnings\n"
    @printf "  fmt          Run rustfmt for all crates\n"
    @printf "  openapi      Export OpenAPI spec to openapi.json\n"
    @printf "\nNotes:\n"
    @printf "  - Designed to work without Docker by default.\n"
    @printf "  - Uses SQLite (sqlite://dev.db) for local profile; Postgres is opt-in.\n"
    @printf "  - Optional tools (just, cargo-watch, curl, jq, psql, pg_isready) are handled gracefully.\n"

# -------------------------------------------------------------------
# setup: verify core prerequisites and suggest optional tools
# -------------------------------------------------------------------
setup:
    @echo "Running setup checks..."
    if ! command -v rustup >/dev/null 2>&1; then
      echo "ERROR: rustup is not installed. See https://rustup.rs/"; exit 1;
    fi
    if ! command -v cargo >/dev/null 2>&1; then
      echo "ERROR: cargo is not available in PATH."; exit 1;
    fi
    echo "Core Rust tooling detected."

    if command -v just >/dev/null 2>&1; then
      echo "just: available."
    else
      echo "just: not found (optional). Install from https://github.com/casey/just or your package manager."
    fi

    if command -v cargo-watch >/dev/null 2>&1; then
      echo "cargo-watch: available."
    else
      echo "cargo-watch: not found (optional). Install with: cargo install cargo-watch"
    fi

    if command -v curl >/dev/null 2>&1; then
      echo "curl: available."
    else
      echo "curl: not found (optional, required for 'just openapi')."
    fi

    if command -v jq >/dev/null 2>&1; then
      echo "jq: available."
    else
      echo "jq: not found (optional, used to validate openapi.json)."
    fi

    if command -v psql >/dev/null 2>&1 || command -v pg_isready >/dev/null 2>&1; then
      echo "Postgres client tools: available."
    else
      echo "Postgres client tools: not found (only needed for 'just db-pg-check' when using Postgres)."
    fi

    echo "Setup checks complete."

# -------------------------------------------------------------------
# env: scaffold .env.local for local profile
# -------------------------------------------------------------------
env:
    @echo "Scaffolding ${env_file} for local profile..."
    if [ ! -f "${env_file}" ]; then
      touch "${env_file}"
    fi

    # Ensure POBLYSH_PROFILE=local
    if ! grep -q '^POBLYSH_PROFILE=' "${env_file}" 2>/dev/null; then
      echo 'POBLYSH_PROFILE=local' >> "${env_file}"
      echo "Set POBLYSH_PROFILE=local"
    fi

    # Ensure POBLYSH_DATABASE_URL=sqlite://dev.db
    if ! grep -q '^POBLYSH_DATABASE_URL=' "${env_file}" 2>/dev/null; then
      echo 'POBLYSH_DATABASE_URL=sqlite://dev.db' >> "${env_file}"
      echo "Set POBLYSH_DATABASE_URL=sqlite://dev.db"
    fi

    # Ensure POBLYSH_OPERATOR_TOKEN
    if ! grep -q '^POBLYSH_OPERATOR_TOKEN=' "${env_file}" 2>/dev/null; then
      echo 'POBLYSH_OPERATOR_TOKEN=local-dev-token' >> "${env_file}"
      echo "Set POBLYSH_OPERATOR_TOKEN=local-dev-token"
    fi

    # Ensure POBLYSH_CRYPTO_KEY (32 bytes base64)
    if ! grep -q '^POBLYSH_CRYPTO_KEY=' "${env_file}" 2>/dev/null; then
      echo "Generating POBLYSH_CRYPTO_KEY (32-byte base64)..."
      if command -v openssl >/dev/null 2>&1; then
        KEY="$(openssl rand -base64 32 | tr -d '\n')"
        # Best-effort length check: decode and count
        if printf "%s" "${KEY}" | base64 -d >/dev/null 2>&1; then
          echo "POBLYSH_CRYPTO_KEY=${KEY}" >> "${env_file}"
          echo "Set POBLYSH_CRYPTO_KEY (generated via openssl)."
        else
          echo "WARNING: Generated key failed decode check; not writing. Please set POBLYSH_CRYPTO_KEY manually to a base64-encoded 32-byte value."
        fi
      else
        echo "WARNING: openssl not found. Please generate a 32-byte base64 key and add:"
        echo "  POBLYSH_CRYPTO_KEY=<base64-encoded-32-byte-key>"
        echo "# POBLYSH_CRYPTO_KEY=<base64-encoded-32-byte-key>" >> "${env_file}"
      fi
    fi

    echo "${env_file} is prepared for local profile."

# -------------------------------------------------------------------
# db-sqlite: ensure sqlite://dev.db file exists
# -------------------------------------------------------------------
db-sqlite:
    @echo "Ensuring SQLite database file ${sqlite_db} exists..."
    if [ ! -f "${sqlite_db}" ]; then
      # Creating an empty file is sufficient; migrations will shape it.
      : > "${sqlite_db}"
      echo "Created ${sqlite_db}"
    else
      echo "${sqlite_db} already exists."
    fi
    echo "SQLite DSN sqlite://dev.db is ready for use."

# -------------------------------------------------------------------
# db-pg-check: optional Postgres connectivity check
# -------------------------------------------------------------------
db-pg-check:
    @echo "Running optional Postgres connectivity check..."
    DB_URL="${POBLYSH_DATABASE_URL:-}"
    if [ -z "${DB_URL}" ] || ! printf "%s" "${DB_URL}" | grep -Eq '^postgres(ql)?://'; then
      echo "POBLYSH_DATABASE_URL is not a Postgres URL; skipping Postgres check (no-op)."
      exit 0
    fi

    # Prefer pg_isready if available
    if command -v pg_isready >/dev/null 2>&1; then
      echo "Using pg_isready to check Postgres..."
      if pg_isready -d "${DB_URL}"; then
        echo "Postgres is reachable."
        exit 0
      else
        echo "Postgres appears unreachable for ${DB_URL}."
        # Non-zero is acceptable since this path is an explicit opt-in diagnostic.
        exit 1
      fi
    fi

    # Fallback: simple psql connect if available
    if command -v psql >/dev/null 2>&1; then
      echo "Using psql to attempt a simple connection..."
      if PGPASSWORD="${PGPASSWORD:-}" psql "${DB_URL}" -c '\q' >/dev/null 2>&1; then
        echo "Postgres is reachable."
        exit 0
      else
        echo "Postgres connection failed for ${DB_URL}."
        exit 1
      fi
    fi

    echo "Postgres client tools (pg_isready/psql) not found."
    echo "Skipping connectivity check. Install Postgres client tools to use this target."
    # Non-blocking for minimal flow
    exit 0

# -------------------------------------------------------------------
# migrate: run database migrations
# -------------------------------------------------------------------
migrate:
    @echo "Running migrations via cargo run --bin connectors -- migrate up ..."
    # Assumes main binary supports `migrate up` as in project README/CLAUDE.md
    cargo run --bin connectors -- migrate up

# -------------------------------------------------------------------
# run: start the API
# -------------------------------------------------------------------
run:
    @echo "Starting Poblysh Connectors API..."
    # Rely on .env.local if present; user typically runs `just env` first.
    cargo run --bin connectors

# -------------------------------------------------------------------
# watch: dev loop with graceful fallback
# -------------------------------------------------------------------
watch:
    @if command -v cargo-watch >/dev/null 2>&1; then
      echo "Starting dev loop with cargo-watch..."
      cargo watch -x 'run'
    else
      echo "cargo-watch is not installed."
      echo "Install with: cargo install cargo-watch"
      echo "Falling back to a single 'cargo run'. Restart manually on changes."
      cargo run --bin connectors
    fi

# -------------------------------------------------------------------
# test: run tests
# -------------------------------------------------------------------
test:
    @echo "Running tests..."
    cargo test

# -------------------------------------------------------------------
# lint: run clippy with -D warnings
# -------------------------------------------------------------------
lint:
    @echo "Running clippy (lint)..."
    cargo clippy --all-targets --all-features -- -D warnings

# -------------------------------------------------------------------
# fmt: format code
# -------------------------------------------------------------------
fmt:
    @echo "Running rustfmt..."
    cargo fmt --all

# -------------------------------------------------------------------
# openapi: export OpenAPI to openapi.json
# -------------------------------------------------------------------
openapi:
    @echo "Exporting OpenAPI specification..."
    if ! command -v curl >/dev/null 2>&1; then
      echo "ERROR: curl is required to export OpenAPI. Please install curl."
      exit 1
    fi

    # Use running server if available.
    # We do a quick probe before writing.
    HTTP_STATUS="$(curl -s -o /dev/null -w '%{http_code}' "${openapi_url}" || true)"

    if [ "${HTTP_STATUS}" != "200" ]; then
      echo "OpenAPI endpoint not reachable at ${openapi_url} (status: ${HTTP_STATUS})."
      echo "Ensure the API is running locally, e.g.:"
      echo "  just env && just db-sqlite && just migrate && just run"
      exit 1
    fi

    curl -s "${openapi_url}" > openapi.json

    if [ ! -s openapi.json ]; then
      echo "ERROR: openapi.json was not written or is empty."
      exit 1
    fi

    if command -v jq >/dev/null 2>&1; then
      if jq . > /dev/null 2>&1 < openapi.json; then
        echo "openapi.json written and validated successfully."
      else
        echo "WARNING: openapi.json is not valid JSON according to jq."
        exit 1
      fi
    else
      echo "openapi.json written. jq not found; JSON validation skipped."
      echo "Install jq for validation: https://stedolan.github.io/jq/"
    fi

smoke:
    @echo "Running E2E smoke tests against the real connectors binary..."
    if [ -z "${POBLYSH_DATABASE_URL:-}" ]; then
      echo "ERROR: POBLYSH_DATABASE_URL is required for smoke tests (Postgres or SQLite)."
      echo "Examples:"
      echo "  POBLYSH_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/poblysh"
      echo "  POBLYSH_DATABASE_URL=sqlite://dev.db"
      exit 1
    fi
    if [ -z "${POBLYSH_OPERATOR_TOKEN:-}" ]; then
      echo "ERROR: POBLYSH_OPERATOR_TOKEN is required to exercise protected endpoints."
      echo "Example:"
      echo "  POBLYSH_OPERATOR_TOKEN=local-dev-token"
      exit 1
    fi
    echo "Using POBLYSH_DATABASE_URL=${POBLYSH_DATABASE_URL}"
    echo "Using POBLYSH_OPERATOR_TOKEN (redacted)"
    POBLYSH_PROFILE="${POBLYSH_PROFILE:-test}" \
    cargo test --test e2e_smoke_tests -- --test-threads=1

smoke-skip-protected:
    @echo "Running E2E smoke tests but skipping protected endpoint checks..."
    if [ -z "${POBLYSH_DATABASE_URL:-}" ]; then
      echo "ERROR: POBLYSH_DATABASE_URL is required for smoke tests."
      exit 1
    fi
    echo "Using POBLYSH_DATABASE_URL=${POBLYSH_DATABASE_URL}"
    POBLYSH_PROFILE="${POBLYSH_PROFILE:-test}" \
    POBLYSH_SMOKE_SKIP_PROTECTED=1 \
    cargo test --test e2e_smoke_tests -- --test-threads=1
