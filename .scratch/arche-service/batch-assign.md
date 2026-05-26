---
title: Batch assign — affixes to blueprints
status: completed
---

## Parent

Arche API Service PRD (see `.scratch/arche-service/prd.md`)

## What to build

Implement batch affix assignment endpoints:

- `POST /api/blueprints/batch/assign` — Request: `{ "blueprint_ids": ["uuid-1"], "affix_ids": ["uuid-3", "uuid-4"], "weight": 1.0 }`. Adds each affix to each blueprint's pool with the given weight. Duplicate (blueprint_id, affix_id) pairs silently skipped.
- `POST /api/affixes/batch/assign` — Symmetric endpoint, same payload shape but semantically from the affix side.

Both operations are idempotent: running the same batch twice results in the same state (no duplicate rows). All affixes and blueprints are validated to belong to the authenticated client. Non-existent IDs are skipped, and a count of successful assignments is returned.

The assign operation respects the affix type (prefix/suffix) — a suffix affix is added as a suffix in the blueprint's pool, a prefix affix as a prefix.

## Acceptance criteria

- [ ] Assign N affixes to M blueprints: N×M rows created
- [ ] Duplicate (blueprint, affix) skipped silently
- [ ] Non-existent IDs skipped
- [ ] Affix type (prefix/suffix) correctly reflected in blueprint_affixes.location
- [ ] Symmetric endpoint from affix side works identically
- [ ] Weight parameter applied to all created rows
- [ ] All operations in single transaction
- [ ] Integration tests

## Blocked by

- arche-service/blueprints-affix-pool.md
- arche-service/affixes-crud.md
