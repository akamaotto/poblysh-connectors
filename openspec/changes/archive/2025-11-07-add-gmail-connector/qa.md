# QA Review: add-gmail-connector

## Summary
- Total Issues Found: 5
- Critical: 1 | High: 2 | Medium: 1 | Low: 1

## Detailed Reports

### Report 1
In openspec/changes/add-gmail-connector/proposal.md around line 28, status code `204` is specified for ack; this conflicts with the established convention in openspec/changes/add-webhook-ingest-endpoint/proposal.md:9 which mandates `202 Accepted` on success. Replace `responds 204 within 1s` with `responds 202 Accepted within 1s` to align across proposals, and update any other references to ack status accordingly.

### Report 2
In openspec/changes/add-gmail-connector/specs/connectors/spec.md around line 19, the webhook ack requirement says "responds with 2xx within one second" which is ambiguous versus the base webhook spec. Change to explicitly require `202 Accepted` for consistency: "responds with `202 Accepted` within one second while enqueueing sync". Consider adding a note that any 2xx is technically an ack for Pub/Sub but `202` is our standard.

### Report 3
In openspec/changes/add-gmail-connector/design.md around the Flow section, the Pub/Sub envelope is underspecified for idempotency. It should explicitly reference `message.messageId` and `subscription` fields used for dedupe/observability. Add: "Use `message.messageId` as the preferred idempotency key and record `subscription` for diagnostics; fall back to `(connection_id, historyId)` only if messageId is unavailable."

### Report 4
In openspec/changes/add-gmail-connector/tasks.md around section 1, the project currently has `reqwest` only as a dev-dependency (Cargo.toml). Add an explicit task to move `reqwest = { version = "0.12.9", features = ["json", "rustls-tls"] }` to runtime dependencies to support Gmail and JWKS HTTP calls.

### Report 5
In openspec/changes/add-gmail-connector/design.md Configuration section, the issuer defaults are correct but the audience requirement should state the exact expected value format (the configured `audience` set on the Pub/Sub push subscription), and that verification must reject tokens where `aud` does not exactly match. Add that clarification and specify typical patterns (service URL or custom audience string).

## Improvement Tasks

### Task 1: Align webhook ack status to 202
**Priority**: Critical
**Files**: openspec/changes/add-gmail-connector/proposal.md, openspec/changes/add-gmail-connector/specs/connectors/spec.md
**Issue**: Inconsistent ack status code vs base webhook spec
**Action Required**: Change references from 204/2xx to explicit `202 Accepted`; add a note that any 2xx acks Pub/Sub, but 202 is our standard

### Task 2: Specify Pub/Sub envelope for idempotency
**Priority**: High
**Files**: openspec/changes/add-gmail-connector/design.md
**Issue**: Missing explicit reference to `messageId` and `subscription`
**Action Required**: Document fields and prefer `messageId` as dedupe key, recording `subscription`

### Task 3: Move reqwest to runtime dependencies
**Priority**: Medium
**Files**: openspec/changes/add-gmail-connector/tasks.md
**Issue**: Missing task to update Cargo dependencies
**Action Required**: Add checklist item to move `reqwest` to `[dependencies]` with TLS features

### Task 4: Clarify OIDC audience matching
**Priority**: High
**Files**: openspec/changes/add-gmail-connector/design.md
**Issue**: Audience matching requirements not explicit
**Action Required**: State exact match required with configured audience and give common patterns (service URL/custom audience)

### Task 5: Note ack timing target vs limits
**Priority**: Low
**Files**: openspec/changes/add-gmail-connector/proposal.md
**Issue**: Single "within 1s" statement may be overly strict without context
**Action Required**: Clarify target ack time (<1s) and maximum acceptable (<10s) per Pub/Sub retry behavior

## Review Notes
- Good alignment with existing Google connectors and shared webhook verification change; keep naming consistent (`gmail` provider slug).
- Strongly consider adding a short example payload stub under design for the Pub/Sub body to aid implementers.

