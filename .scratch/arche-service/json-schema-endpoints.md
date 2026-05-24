---
title: JSON Schema introspection endpoints
status: completed
---

## Parent

Arche API Service PRD (see `.scratch/arche-service/prd.md`)

## What to build

Implement three JSON Schema endpoints that describe the request/response structures for the Admin UI's API client generator:

- `GET /api/schema/blueprints` — JSON Schema document for blueprint create/update payload and response
- `GET /api/schema/affixes` — JSON Schema for affix create/update payload and response
- `GET /api/schema/generate` — JSON Schema for generate request/response

Schemas are derived from the Rust types in `arche-types` using the `schemars` crate. They are generated at compile time (build script) or lazily on first request and cached thereafter.

The schemas are static — they do not vary by client. They describe the structural contract, not instance data (e.g., not listing actual attribute values, just the valid shapes).

The Admin UI's API client generation (`npm run generate-api-client`) consumes these endpoints.

## Acceptance criteria

- [ ] `GET /api/schema/blueprints` returns a valid JSON Schema document
- [ ] `GET /api/schema/affixes` returns valid JSON Schema
- [ ] `GET /api/schema/generate` returns valid JSON Schema
- [ ] Schemas are derived from Rust types (not hand-written)
- [ ] Schemas are stable (same response on every call, no client dependence)
- [ ] Response Content-Type is `application/schema+json` or `application/json`
- [ ] Integration test: fetch schema, validate it's well-formed JSON Schema

## Blocked by

- arche-types/domain-structs-enums.md
- arche-types/request-response-types.md
- arche-service/service-scaffolding.md
