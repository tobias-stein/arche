---
title: Blueprints — delete & force delete
status: ready-for-agent
---

## Parent

Arche API Service PRD (see `.scratch/arche-service/prd.md`)

## What to build

Implement blueprint deletion with reference protection:

- Regular delete (`DELETE /api/blueprints/:id`): Succeeds if the blueprint has no references from `blueprint_affixes` (no other resources depend on it). Otherwise returns 409 with details of what references it.
- Force delete (`DELETE /api/blueprints/:id?force=true`): Cascade deletes all `blueprint_affixes` rows referencing this blueprint. DB-level `ON DELETE CASCADE` from `blueprints` → `blueprint_affixes` handles it; the force flag controls the application-level reference check bypass.

Return 404 if blueprint doesn't exist or belongs to a different client.

## Acceptance criteria

- [ ] Delete blueprint with no affix pool references succeeds
- [ ] Delete blueprint that has affixes in pool returns 409 with reference details
- [ ] Force delete succeeds even with affixes in pool, cascade deletes associations
- [ ] Delete non-existent blueprint returns 404
- [ ] Delete blueprint belonging to another client returns 404
- [ ] Integration tests

## Blocked by

- arche-service/blueprints-crud-core.md
- arche-service/blueprints-affix-pool.md
