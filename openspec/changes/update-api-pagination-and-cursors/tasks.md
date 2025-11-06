## 1. Implementation
- [ ] 1.1 Add shared pagination contract to docs (API core) and update endpoint docs to reference it
- [ ] 1.2 Introduce cursor codec util: Base64URL(JSON{ ver, keys }) using `base64` + `serde_json`
- [ ] 1.3 Standardize stable ordering per endpoint: add explicit secondary `id` tiebreakers in queries
- [ ] 1.4 Update repository list methods to accept `(limit, cursor)` and return `(items, next_cursor)`
- [ ] 1.5 API responses: always include `next_cursor` (null when no more pages)
- [ ] 1.6 OpenAPI: generic `Paginated<T>` schema + parameter docs for `limit` and `cursor`
- [ ] 1.7 Tests: tie-breaker stability, cursor round-trip, first/next-page coverage, bounds validation
- [ ] 1.8 Add `base64 = "0.22"` dependency in `Cargo.toml`

## 2. Rollout & Compatibility
- [ ] 2.1 Verify all existing list endpoints for consistent ordering and return shape
- [ ] 2.2 Verify SeaORM queries use deterministic tie-breakers (ASC/DSC + `id`)
- [ ] 2.3 Update Swagger examples to show `next_cursor: null` for last page
- [ ] 2.4 Audit in-progress OpenSpec changes (e.g., jobs endpoint) and align wording to "always include next_cursor (nullable)"

## 3. Notes / Non-goals
- Tokens are opaque for clients, not cryptographically tamper-proof (v1)
- Keep cursor payload minimal: only the sort keys required for resuming
