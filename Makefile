# Poblysh Connectors - Local Development Makefile
#
# This Makefile implements the local run scripts defined by the
# `add-local-run-scripts` dev-tooling OpenSpec change.
#
# Goals:
# - Zero-docker local development.
# - SQLite-first happy path.
# - No new runtime dependencies.
# - Deterministic, idempotent behavior.
# - Friendly, explicit handling of optional tools.
#
# Primary targets (mirrored by Justfile):
#   help, setup, env, db-sqlite, db-pg-check, migrate,
#   run, watch, test, lint, fmt, openapi
#
# Notes:
# - Assumes execution from repo root.
# - Uses only ubiquitous POSIX-compatible features to stay cross-platform.
# - Does not mutate runtime code or specs.


# ==============================================================================
# Configuration
# ==============================================================================

SHELL := /bin/sh

# Default values used when scaffolding .env.local
ENV_FILE        := .env.local
LOCAL_PROFILE   := local
LOCAL_DB_URL    := sqlite://dev.db
LOCAL_DB_FILE   := dev.db
LOCAL_OP_TOKEN  := local-dev-token

# Default API bind address, only documented here; actual default is determined
# by the Rust config loader.
DEFAULT_BIND_ADDR := 0.0.0.0:8080

# OpenAPI endpoint path (must match server configuration)
OPENAPI_URL ?= http://localhost:8080/openapi.json

# Detect OS utilities lazily inside targets for portability.


# ==============================================================================
# Helpers
# ==============================================================================

.PHONY: help
help: ## Show available local development commands
	@echo "Poblysh Connectors - Local Development"
	@echo
	@echo "Primary targets:"
	@printf "  %-15s %s\n" "help"        "Show this help message"
	@printf "  %-15s %s\n" "setup"       "Check core tooling; hint optional tools"
	@printf "  %-15s %s\n" "env"         "Create/update $(ENV_FILE) with sane local defaults"
	@printf "  %-15s %s\n" "db-sqlite"   "Ensure local SQLite database file ($(LOCAL_DB_FILE)) exists"
	@printf "  %-15s %s\n" "db-pg-check" "Optionally check Postgres connectivity if configured"
	@printf "  %-15s %s\n" "migrate"     "Run database migrations (idempotent)"
	@printf "  %-15s %s\n" "run"         "Run the API with local configuration"
	@printf "  %-15s %s\n" "watch"       "Run the API in watch mode if cargo-watch is available"
	@printf "  %-15s %s\n" "test"        "Run tests"
	@printf "  %-15s %s\n" "lint"        "Run clippy with -D warnings"
	@printf "  %-15s %s\n" "fmt"         "Run cargo fmt"
	@printf "  %-15s %s\n" "openapi"     "Export OpenAPI spec to openapi.json"


# ==============================================================================
# Tooling & Environment
# ==============================================================================

.PHONY: setup
setup: ## Check for required and optional tools
	@echo "==> Checking core tooling..."
	@if command -v rustup >/dev/null 2>&1; then \
	  echo "  [ok] rustup found"; \
	else \
	  echo "  [!!] rustup not found - install from https://rustup.rs/"; \
	fi
	@if command -v cargo >/dev/null 2>&1; then \
	  echo "  [ok] cargo found"; \
	else \
	  echo "  [!!] cargo not found (should be installed via rustup)"; \
	fi
	@echo
	@echo "==> Optional tools (recommended, not required)..."
	@if command -v just >/dev/null 2>&1; then \
	  echo "  [ok] just found"; \
	else \
	  echo "  [..] just not found (optional) - see https://just.systems/man/en/"; \
	fi
	@if command -v cargo-watch >/dev/null 2>&1; then \
	  echo "  [ok] cargo-watch found"; \
	else \
	  echo "  [..] cargo-watch not found (optional) - install: cargo install cargo-watch"; \
	fi
	@if command -v curl >/dev/null 2>&1; then \
	  echo "  [ok] curl found"; \
	else \
	  echo "  [..] curl not found - required for 'make openapi' (install via your package manager)"; \
	fi
	@if command -v jq >/dev/null 2>&1; then \
	  echo "  [ok] jq found"; \
	else \
	  echo "  [..] jq not found (optional) - recommended for validating openapi.json"; \
	fi
	@if command -v psql >/dev/null 2>&1 || command -v pg_isready >/dev/null 2>&1; then \
	  echo "  [ok] Postgres client tools detected (psql/pg_isready)"; \
	else \
	  echo "  [..] Postgres client tools not found (only needed for db-pg-check when using Postgres)"; \
	fi
	@echo
	@echo "Setup check complete."


.PHONY: env
env: ## Create or update .env.local with local defaults (idempotent)
	@echo "==> Ensuring $(ENV_FILE) exists with local defaults"
	@if [ ! -f "$(ENV_FILE)" ]; then \
	  echo "Creating $(ENV_FILE)"; \
	  touch "$(ENV_FILE)"; \
	fi

	# Ensure POBLYSH_PROFILE=local
	@if ! grep -Eq '^POBLYSH_PROFILE=' "$(ENV_FILE)"; then \
	  echo "POBLYSH_PROFILE=$(LOCAL_PROFILE)" >> "$(ENV_FILE)"; \
	  echo "  [set] POBLYSH_PROFILE=$(LOCAL_PROFILE)"; \
	fi

	# Ensure POBLYSH_DATABASE_URL=sqlite://dev.db
	@if ! grep -Eq '^POBLYSH_DATABASE_URL=' "$(ENV_FILE)"; then \
	  echo "POBLYSH_DATABASE_URL=$(LOCAL_DB_URL)" >> "$(ENV_FILE)"; \
	  echo "  [set] POBLYSH_DATABASE_URL=$(LOCAL_DB_URL)"; \
	fi

	# Ensure POBLYSH_OPERATOR_TOKEN is set
	@if ! grep -Eq '^POBLYSH_OPERATOR_TOKEN=' "$(ENV_FILE)"; then \
	  echo "POBLYSH_OPERATOR_TOKEN=$(LOCAL_OP_TOKEN)" >> "$(ENV_FILE)"; \
	  echo "  [set] POBLYSH_OPERATOR_TOKEN=$(LOCAL_OP_TOKEN)"; \
	fi

	# Ensure POBLYSH_CRYPTO_KEY is a 32-byte base64 value (generate if missing)
	@if ! grep -Eq '^POBLYSH_CRYPTO_KEY=' "$(ENV_FILE)"; then \
	  echo "  [gen] Generating POBLYSH_CRYPTO_KEY (32-byte base64)"; \
	  if command -v openssl >/dev/null 2>&1; then \
	    KEY="$$(openssl rand -base64 32 | tr -d '\n')"; \
	    DECODED_LEN="$$(printf "%s" "$$KEY" | base64 -d 2>/dev/null | wc -c | tr -d ' ')"; \
	    if [ "$$DECODED_LEN" = "32" ]; then \
	      echo "POBLYSH_CRYPTO_KEY=$$KEY" >> "$(ENV_FILE)"; \
	      echo "  [set] POBLYSH_CRYPTO_KEY (generated via openssl)"; \
	    else \
	      echo "  [!!] Generated key length ($$DECODED_LEN) != 32 bytes; please set POBLYSH_CRYPTO_KEY manually."; \
	    fi; \
	  else \
	    echo "  [!!] openssl not found; please add a base64-encoded 32-byte POBLYSH_CRYPTO_KEY to $(ENV_FILE)."; \
	    echo "# TODO: set POBLYSH_CRYPTO_KEY=<base64-encoded-32-bytes>" >> "$(ENV_FILE)"; \
	  fi; \
	fi

	@echo "Environment scaffolding complete. Review $(ENV_FILE) as needed."


# ==============================================================================
# Database
# ==============================================================================

.PHONY: db-sqlite
db-sqlite: ## Ensure local SQLite database file exists for sqlite://dev.db
	@echo "==> Ensuring local SQLite database file $(LOCAL_DB_FILE)"
	@if [ ! -f "$(LOCAL_DB_FILE)" ]; then \
	  echo "Creating $(LOCAL_DB_FILE) (empty file; migrations will populate schema)"; \
	  : > "$(LOCAL_DB_FILE)"; \
	else \
	  echo "  [ok] $(LOCAL_DB_FILE) already exists"; \
	fi
	@echo "db-sqlite complete."


.PHONY: db-pg-check
db-pg-check: ## Optionally verify Postgres connectivity if POBLYSH_DATABASE_URL is postgres://
	@echo "==> Checking Postgres configuration (optional)"
	@if [ -f "$(ENV_FILE)" ]; then \
	  . "$(ENV_FILE)"; \
	fi; \
	URL="$${POBLYSH_DATABASE_URL:-}"; \
	if [ -z "$$URL" ] || printf "%s" "$$URL" | grep -Evq '^postgres(ql)?://'; then \
	  echo "POBLYSH_DATABASE_URL is not postgres:// - skipping Postgres check (no-op)."; \
	  exit 0; \
	fi; \
	echo "Detected Postgres URL in POBLYSH_DATABASE_URL=$$URL"; \
	if command -v pg_isready >/dev/null 2>&1; then \
	  echo "Using pg_isready to test connectivity..."; \
	  if pg_isready -d "$$URL"; then \
	    echo "  [ok] Postgres is reachable."; \
	    exit 0; \
	  else \
	    echo "  [!!] Postgres appears unreachable. Inspect POBLYSH_DATABASE_URL and local DB."; \
	    exit 1; \
	  fi; \
	elif command -v psql >/dev/null 2>&1; then \
	  echo "Using psql to attempt a simple connection..."; \
	  if PGPASSWORD="" psql "$$URL" -c '\q' >/dev/null 2>&1; then \
	    echo "  [ok] Postgres connection succeeded."; \
	    exit 0; \
	  else \
	    echo "  [!!] Failed to connect to Postgres using psql. Check URL or service status."; \
	    exit 1; \
	  fi; \
	else \
	  echo "  [..] No pg_isready or psql found. Postgres check is skipped."; \
	  echo "       Install Postgres client tools if you need db-pg-check for Postgres workflows."; \
	  exit 0; \
	fi


# ==============================================================================
# Migrations & Runtime
# ==============================================================================

.PHONY: migrate
migrate: ## Run database migrations (idempotent)
	@echo "==> Running migrations via cargo"
	@if [ -f "$(ENV_FILE)" ]; then \
	  . "$(ENV_FILE)"; \
	fi; \
	cargo run --bin connectors -- migrate up
	@echo "Migrations complete."


.PHONY: run
run: ## Run the API using current configuration
	@echo "==> Starting API (run)"
	@if [ -f "$(ENV_FILE)" ]; then \
	  . "$(ENV_FILE)"; \
	fi; \
	cargo run --bin connectors


.PHONY: watch
watch: ## Run the API in watch mode if cargo-watch is available; otherwise guide and fallback
	@echo "==> Starting API in watch mode (or fallback)"
	@if command -v cargo-watch >/dev/null 2>&1; then \
	  echo "cargo-watch detected; running watch loop..."; \
	  cargo watch -x 'run'; \
	else \
	  echo "cargo-watch not found."; \
	  echo "To install: cargo install cargo-watch"; \
	  echo "Falling back to a single 'cargo run' invocation."; \
	  if [ -f "$(ENV_FILE)" ]; then \
	    . "$(ENV_FILE)"; \
	  fi; \
	  cargo run --bin connectors; \
	fi


# ==============================================================================
# Quality: Tests, Lint, Format
# ==============================================================================

.PHONY: test
test: ## Run tests
	@echo "==> Running tests"
	cargo test


.PHONY: lint
lint: ## Run clippy with -D warnings
	@echo "==> Running clippy (lint)"
	cargo clippy --all-targets --all-features -- -D warnings


.PHONY: fmt
fmt: ## Run cargo fmt
	@echo "==> Running cargo fmt"
	cargo fmt --all


# ==============================================================================
# OpenAPI Export
# ==============================================================================

.PHONY: openapi
openapi: ## Export OpenAPI spec to openapi.json
	@echo "==> Exporting OpenAPI spec to openapi.json"
	@if ! command -v curl >/dev/null 2>&1; then \
	  echo "  [!!] curl is required for openapi export. Please install curl and retry."; \
	  exit 1; \
	fi; \
	if [ -f "$(ENV_FILE)" ]; then \
	  . "$(ENV_FILE)"; \
	fi; \
	echo "Attempting to fetch OpenAPI from: $(OPENAPI_URL)"; \
	if curl -fsS "$(OPENAPI_URL)" -o openapi.json; then \
	  echo "  [ok] Wrote openapi.json"; \
	  if command -v jq >/dev/null 2>&1; then \
	    if jq . >/dev/null 2>&1 < openapi.json; then \
	      echo "  [ok] openapi.json is valid JSON"; \
	    else \
	      echo "  [!!] openapi.json is not valid JSON according to jq."; \
	      exit 1; \
	    fi; \
	  else \
	    echo "  [..] jq not found; skipping JSON validation. Install jq for extra verification."; \
	  fi; \
	else \
	  echo "  [!!] Failed to fetch OpenAPI from $(OPENAPI_URL)."; \
	  echo "       Ensure the server is running (e.g., 'make run') and reachable, then retry."; \
	  exit 1; \
	fi
	@echo "OpenAPI export complete."

.PHONY: smoke
smoke: ## Run E2E smoke tests against the real connectors binary
	@echo "==> Running E2E smoke tests"
	@if [ -z "$${POBLYSH_DATABASE_URL:-}" ]; then \
	  echo "  [!!] POBLYSH_DATABASE_URL is required for smoke tests (Postgres or SQLite)."; \
	  echo "       Examples:"; \
	  echo "         POBLYSH_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/poblysh"; \
	  echo "         POBLYSH_DATABASE_URL=sqlite://dev.db"; \
	  exit 1; \
	fi; \
	if [ -z "$${POBLYSH_OPERATOR_TOKEN:-}" ]; then \
	  echo "  [!!] POBLYSH_OPERATOR_TOKEN is required to exercise protected endpoints."; \
	  echo "       Example:"; \
	  echo "         POBLYSH_OPERATOR_TOKEN=local-dev-token"; \
	  exit 1; \
	fi; \
	echo "Using POBLYSH_DATABASE_URL=$${POBLYSH_DATABASE_URL}"; \
	echo "Using POBLYSH_OPERATOR_TOKEN (redacted)"; \
	POBLYSH_PROFILE="$${POBLYSH_PROFILE:-test}" \
	cargo test --test e2e_smoke_tests -- --test-threads=1

.PHONY: smoke-skip-protected
smoke-skip-protected: ## Run E2E smoke tests but skip protected endpoint checks
	@echo "==> Running E2E smoke tests (skipping protected endpoints)"
	@if [ -z "$${POBLYSH_DATABASE_URL:-}" ]; then \
	  echo "  [!!] POBLYSH_DATABASE_URL is required for smoke tests."; \
	  exit 1; \
	fi; \
	echo "Using POBLYSH_DATABASE_URL=$${POBLYSH_DATABASE_URL}"; \
	POBLYSH_PROFILE="$${POBLYSH_PROFILE:-test}" \
	POBLYSH_SMOKE_SKIP_PROTECTED=1 \
	cargo test --test e2e_smoke_tests -- --test-threads=1
