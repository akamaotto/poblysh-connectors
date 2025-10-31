## 1. Implementation
- [ ] 1.1 Add deps: `dotenvy`, `envy`, and `thiserror` (or reuse `anyhow`).
- [ ] 1.2 Create `src/config.rs` with `AppConfig` and loader.
- [ ] 1.3 Support `POBLYSH_PROFILE` with layered file precedence:
      `.env`, `.env.local`, `.env.<profile>`, `.env.<profile>.local` (last wins).
- [ ] 1.4 Map env to struct via `envy` with prefix `POBLYSH_`.
- [ ] 1.5 Validate: parse `api_bind_addr` as `SocketAddr`; report aggregate errors.
- [ ] 1.6 Redact secrets on debug print (e.g., keys, passwords, tokens).
- [ ] 1.7 Wire `main`/`server` to use `AppConfig` (keep existing defaults).
- [ ] 1.8 Unit tests: precedence, defaults, validation (invalid bind addr), profile selection.

## 2. Validation
- [ ] 2.1 `cargo test -q` passes for config module.
- [ ] 2.2 `openspec validate add-config-and-env-loading --strict` shows no errors.

## 3. Notes / Non-goals
- No production secret store integration (local env files only).
- Do not require `POBLYSH_CRYPTO_KEY` yet; will become required in the crypto change.

