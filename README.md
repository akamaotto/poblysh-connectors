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

## Environment Variables

- `POBLYSH_API_BIND_ADDR`: The address and port to bind the server to (default: `0.0.0.0:8080`)

Example:
```bash
POBLYSH_API_BIND_ADDR=127.0.0.1:3000 cargo run
```

## Available Endpoints

- `/` - Root endpoint that returns basic service information
- `/docs` - Swagger UI for interactive API documentation
- `/openapi.json` - OpenAPI specification in JSON format