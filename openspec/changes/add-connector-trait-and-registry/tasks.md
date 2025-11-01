## 1. Implementation
- [ ] 1.1 Define `Connector` trait with async methods (authorize, exchange_token, refresh_token, sync, handle_webhook).
- [ ] 1.2 Add `ProviderMetadata { name, auth_type, scopes, webhooks }` and `AuthType` enum.
- [ ] 1.3 Implement `registry` module with `get(name)`, `list_metadata()`; ensure stable name sort.
- [ ] 1.4 Add a stub connector (e.g., `example`) and register it with metadata.
- [ ] 1.5 Unit tests: resolve known/unknown, list ordering, metadata completeness.

## 2. Validation
- [ ] 2.1 `cargo test` passes for new modules.
- [ ] 2.2 Ensure no API changes introduced in this change (wiring only).

## 3. Notes / Non-goals
- No real provider HTTP calls; stubs only.
- API exposure (`/providers`) is covered by a separate change.

