---
title: Global meta attributes CRUD
status: completed
---

## Parent

Arche API Service PRD (see `.scratch/arche-service/prd.md`)

## What to build

Implement full CRUD for global meta attributes:

- `GET /api/global-meta-attributes` — List with cursor pagination, filter by value_type, search by name
- `POST /api/global-meta-attributes` — Create with name, description, value_type, and type-specific payload. Validate payload against value type rules.
- `GET /api/global-meta-attributes/:id` — Get single attribute
- `PUT /api/global-meta-attributes/:id` — Update. Changes propagate automatically at generation time (resolution happens at generation, not write).
- `DELETE /api/global-meta-attributes/:id` — Rejected by default if any blueprint or affix references this attribute via `$ref_id`. Returns 409 with reference details.
- Force delete via `DELETE /api/global-meta-attributes/:id?force=true` — Cascading: remove all `$ref_id` references from blueprints and affixes. Log `adjusted` for each modified resource.

Names must be unique per client. Validation uses the shared `arche-types` validation logic.

## Acceptance criteria

- [ ] CRUD endpoints work as specified
- [ ] Create validates payload per value type (range min ≤ max, enum values non-empty, etc.)
- [ ] Create with duplicate name (same client) returns 409
- [ ] Regular delete blocked when referenced (returns 409 with reference list)
- [ ] Force delete removes all `$ref_id` references, logs adjustments
- [ ] Pagination and filtering on list endpoint work
- [ ] Integration tests for all scenarios

## Blocked by

- arche-types/database-migrations.md
- arche-types/domain-structs-enums.md
- arche-types/validation-logic.md
- arche-service/auth-api-key-extractor.md
- arche-service/permission-matrix.md
