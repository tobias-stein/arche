---
title: Blueprints CRUD — core
status: completed
---

## Parent

Arche API Service PRD (see `.scratch/arche-service/prd.md`)

## What to build

Implement core blueprint CRUD (without affix pool — that's a separate slice):

- `POST /api/blueprints` — Create blueprint with name, archetype, weight, description, attributes (inline or `$ref_id`), and attribute_order. Validate attribute payloads, `$ref_id`/inline mutual exclusivity, and attribute_order completeness.
- `GET /api/blueprints/:id` — Get single blueprint with its attributes
- `PUT /api/blueprints/:id` — Update any field. Re-validate attribute_order.
- `DELETE /api/blueprints/:id` — Rejected by default if the blueprint is referenced (no references yet in core — this applies to future affix pool references).

Attributes are stored as JSONB. `attribute_order` is stored as TEXT[] and validated at write time: all keys present, no extras, no duplicates.

Names must be unique per client.

## Acceptance criteria

- [ ] Create blueprint with inline attributes succeeds
- [ ] Create blueprint with `$ref_id` attributes succeeds
- [ ] Create blueprint with both `$ref_id` and inline on same key returns 400
- [ ] Create with invalid attribute_order (missing key, extra key, duplicate) returns 400
- [ ] Create with duplicate name (same client) returns 409
- [ ] Update blueprint modifies only specified fields
- [ ] Get blueprint returns full object with attributes
- [ ] Integration tests for all scenarios

## Blocked by

- arche-types/database-migrations.md
- arche-types/domain-structs-enums.md
- arche-types/validation-logic.md
- arche-service/auth-api-key-extractor.md
- arche-service/permission-matrix.md
