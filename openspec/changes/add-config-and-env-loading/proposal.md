## Why
Local development and future environments need a consistent, typed configuration system. Today the server reads a single env var (`POBLYSH_API_BIND_ADDR`) ad‑hoc. We should standardize on `POBLYSH_*` environment variables, support local profile overlays via `.env` files, and validate required settings early with clear errors.

## What Changes
- Introduce a centralized config loader with `POBLYSH_*` env prefix.
- Add local profiles via `POBLYSH_PROFILE` with layered `.env` precedence.
- Validate typed settings on startup; fail fast with actionable messages.
- Redact secrets when logging config.
- Replace ad‑hoc reads with typed `AppConfig` usage (non‑breaking defaults preserved).
- Add unit tests covering precedence, defaults, and validation errors.

## Impact
- Affected specs: `config`
- Affected code: `src/config.rs` (new), `src/main.rs`, `src/server.rs`
- Dependencies: add `dotenvy` (load .env), `envy` (env→struct), optional `thiserror`/`anyhow` for errors

