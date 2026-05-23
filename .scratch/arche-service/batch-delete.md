---
title: Batch delete — blueprints & affixes
status: ready-for-agent
---

## Parent

Arche API Service PRD (see `.scratch/arche-service/prd.md`)

## What to build

Implement batch delete endpoints for blueprints and affixes:

- `POST /api/blueprints/batch/delete` — Request: `{ "ids": ["uuid-1", "uuid-2"] }`. Delete each blueprint in a loop with individual reference checks. Skip blueprints that don't exist or belong to another client. Return `{ "deleted_count": N }`.
- `POST /api/affixes/batch/delete` — Same pattern for affixes.

Each deletion in the batch respects the same delete protection rules as single delete. If a resource is referenced, the entire batch fails with 409 and details of which resource is blocked (atomic batch, not partial). Use a single DB transaction.

Force-delete is not supported in batch mode — users must use single force-delete for individual items.

## Acceptance criteria

- [ ] Batch delete blueprints: works when none are referenced
- [ ] Batch delete blueprints: returns 409 if any is referenced (reports which one)
- [ ] Batch delete affixes: same behavior
- [ ] Non-existent IDs silently skipped (not an error)
- [ ] Delete count returned correctly
- [ ] All-or-nothing in a single transaction
- [ ] Integration tests

## Blocked by

- arche-service/blueprints-delete-force-delete.md
- arche-service/affixes-crud.md
