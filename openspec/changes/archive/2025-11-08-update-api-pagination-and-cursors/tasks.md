## 1. Implementation
- [x] 1.1 Add shared pagination contract to docs (API core) and update endpoint docs to reference it
- [x] 1.2 Introduce cursor codec util: Standard Base64(JSON{ ver, keys }) using `base64` + `serde_json`
- [x] 1.3 Standardize stable ordering per endpoint: add explicit secondary `id` tiebreakers in queries
- [x] 1.4 Update repository list methods to accept `(limit, cursor)` and return `(items, next_cursor)`
- [x] 1.5 API responses: always include `next_cursor` (null when no more pages)
- [x] 1.6 OpenAPI: generic `Paginated<T>` schema + parameter docs for `limit` and `cursor`
- [x] 1.7 Tests: tie-breaker stability, cursor round-trip, first/next-page coverage, bounds validation
- [x] 1.8 Add `base64 = "0.22"` dependency in `Cargo.toml`

## 2. Rollout & Compatibility
- [x] 2.1 Verify all existing list endpoints for consistent ordering and return shape
- [x] 2.2 Verify SeaORM queries use deterministic tie-breakers (ASC/DSC + `id`)
- [x] 2.3 Update Swagger examples to show `next_cursor: null` for last page
- [x] 2.4 Audit in-progress OpenSpec changes (e.g., jobs endpoint) and align wording to "always include next_cursor (nullable)"

### 2.5 Endpoint Compliance Checklist
All list endpoints MUST reference and comply with the `api-core` pagination requirements:

**Current endpoints verified:**
- [x] `GET /signals` - Event-like ordering: `occurred_at DESC, id DESC`
- [x] `GET /jobs` - Event-like ordering: `scheduled_at DESC, id DESC`
- [x] `GET /connections` - Catalog-like ordering: `created_at ASC, id ASC`
- [x] `GET /providers` - Catalog-like ordering: `name ASC, id ASC` (static list)

**Future endpoints must:**
- Reference `api-core` pagination requirements in their spec
- Define explicit stable ordering with `id` tiebreaker
- Always include `next_cursor` in responses (null when no more pages)
- Support `limit` (default 50, max 100) and optional `cursor` parameters
- Validate cursor format and reject malformed/invalid cursors with 400 errors

## 3. Notes / Non-goals
- Tokens are opaque for clients, not cryptographically tamper-proof (v1)
- Keep cursor payload minimal: only the sort keys required for resuming
