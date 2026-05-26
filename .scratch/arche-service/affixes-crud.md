---
title: Affixes CRUD
status: completed
---

## Parent

Arche API Service PRD (see `.scratch/arche-service/prd.md`)

## What to build

Implement full CRUD for affixes:

- `GET /api/affixes` — List with cursor pagination, filter by type (prefix/suffix), search by name
- `POST /api/affixes` — Create affix with name, type (prefix/suffix), description, and exactly one meta attribute (either inline definition or `$ref_id` to global). Validate inline attribute payload.
- `GET /api/affixes/:id` — Get single affix
- `PUT /api/affixes/:id` — Update affix fields and attribute
- `DELETE /api/affixes/:id` — Rejected by default if any blueprint references this affix in its pool. Returns 409 with reference list (which blueprints, how many).
- Force delete via `DELETE /api/affixes/:id?force=true` — Cascade: remove affix from all blueprint pools via `blueprint_affixes` deletions. Auto-adjust min/max counts on affected blueprints. Log each adjustment with action `adjusted`.

Names must be unique per client. Attribute validation uses shared `arche-types` logic.

## Acceptance criteria

- [ ] Full CRUD endpoints work as specified
- [ ] Create with inline attribute validates payload rules
- [ ] Create with `$ref_id` references an existing global meta attribute
- [ ] Create with both inline and `$ref_id` returns 400
- [ ] Regular delete blocked when referenced (returns 409 with blueprint list)
- [ ] Force delete removes from all blueprint pools, adjusts min/max counts
- [ ] Auto-adjustment logs `adjusted` audit entries
- [ ] Integration tests for all scenarios

## Blocked by

- arche-types/database-migrations.md
- arche-types/domain-structs-enums.md
- arche-types/validation-logic.md
- arche-service/auth-api-key-extractor.md
- arche-service/permission-matrix.md
