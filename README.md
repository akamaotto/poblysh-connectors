# Connectors API

A Rust-based API service for managing connectors, built with Axum and featuring OpenAPI documentation.

## Prerequisites

- Rust toolchain (latest stable version recommended)

## Local Run

To run the project locally:

```bash
cargo run
```

The server will start on the address specified by the `POBLYSH_API_BIND_ADDR` environment variable (default: `0.0.0.0:8080`).

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

Example:
```bash
POBLYSH_PROFILE=test \
POBLYSH_API_BIND_ADDR=127.0.0.1:3000 \
POBLYSH_LOG_LEVEL=debug \
cargo run
```

Configuration is logged at startup using a redacted JSON representation (no secrets in the current schema).

## Environment Variables

- `POBLYSH_PROFILE`: Configuration profile to use (default: `local`)
- `POBLYSH_API_BIND_ADDR`: Address and port for the HTTP server (default: `0.0.0.0:8080`)
- `POBLYSH_LOG_LEVEL`: Log verbosity (`trace`, `debug`, `info`, `warn`, `error`; default: `info`)

Examples:
```bash
# Set profile
POBLYSH_PROFILE=prod cargo run

# Set API bind address
POBLYSH_API_BIND_ADDR=127.0.0.1:3000 cargo run

# Set both
POBLYSH_PROFILE=prod POBLYSH_API_BIND_ADDR=127.0.0.1:3000 cargo run
```

## Available Endpoints

- `/` - Root endpoint that returns basic service information
- `/docs` - Swagger UI for interactive API documentation
- `/openapi.json` - OpenAPI specification in JSON format
