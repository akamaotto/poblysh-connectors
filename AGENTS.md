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

Keep this managed block so 'openspec update' can refresh the instructions.

<!-- OPENSPEC:END -->

## Docker QA Guidance

- When evaluating the dockerization effort (e.g., `add-dockerization-post-mvp`), follow the new guide in `README.md` under **Docker Build & Swagger UI Testing** before approving any change.
- The guide covers `docker build`, running the container with `POBLYSH_*` environment variables, and manually exercising the Swagger UI (`/docs`) plus `/openapi.json`; refer back to it whenever manual QA is requested.
