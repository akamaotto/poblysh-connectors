<!-- OPENSPEC:START -->
# OpenSpec Instructions

These instructions are for AI assistants working in this project.

Always open `@/openspec/AGENTS.md` when the request:
- Mentions planning or proposals (words like proposal, spec, change, plan)
- Introduces new capabilities, breaking changes, architecture shifts, or big performance/security work
- Sounds ambiguous and you need the authoritative spec before coding

Use `@/openspec/AGENTS.md` to learn:
- How to create and apply change proposals
- Spec format and conventions
- Project structure and guidelines
- How the `examples/nextjs-demo` mock UX project fits into the overall, spec-driven workflow

Keep this managed block so 'openspec update' can refresh the instructions.

<!-- OPENSPEC:END -->

## TypeScript Guidelines

When working with TypeScript in this repository (including `examples/nextjs-demo` and any other TS/TSX code):

- Do not use the `any` type. Always prefer precise, strongly typed alternatives such as:
  - Specific primitives (`string`, `number`, `boolean`)
  - Domain-specific interfaces, types, or enums
  - Generics with concrete constraints
  - Discriminated unions for variant data
- Do not use the `undefined` type as an explicit annotation in public or internal APIs.
  - For optional values, use optional properties (`foo?: T`) or `T | null` as appropriate.
  - Model absence explicitly in the type system (e.g. union types, dedicated state objects) instead of relying on `undefined`.
- If you believe `any` or `undefined` is necessary, reconsider the design and refactor toward a safer, more expressive type before proceeding.

These constraints are intended to maintain a strictly typed, predictable codebase and should be followed consistently.

## Docker QA Guidance

- When evaluating the dockerization effort (e.g., `add-dockerization-post-mvp`), follow the new guide in `README.md` under **Docker Build & Swagger UI Testing** before approving any change.
- The guide covers `docker build`, running the container with `POBLYSH_*` environment variables, and manually exercising the Swagger UI (`/docs`) plus `/openapi.json`; refer back to it whenever manual QA is requested.
