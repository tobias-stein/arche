---
title: Consistency — delete protection & force delete cascade
status: ready-for-agent
---

## Parent

Arche API Service PRD (see `.scratch/arche-service/prd.md`)

## What to build

Implement unified delete protection and force-delete cascade for all resource types.

**Delete protection (all resources):**
- Default `DELETE` returns 409 if the resource is referenced by any other resource
- The 409 response includes a list of referencing resources (type + id + name) so the frontend can guide the user
- Reference checking:
  - Blueprint: referenced by `blueprint_affixes`
  - Affix: referenced by `blueprint_affixes` (blueprint pool entries)
  - Global meta attribute: referenced by blueprints and affixes via `$ref_id`

**Force delete via `?force=true`:**
- Affix: remove all `blueprint_affixes` rows; auto-adjust `min_prefixes`/`max_prefixes`/`min_suffixes`/`max_suffixes` on affected blueprints. First downsize max, then min. Log `adjusted` for each modified blueprint.
- Global meta attribute: scan all blueprints and affixes for `$ref_id` pointing to this attribute; remove the entire attribute entry (not just the `$ref_id`). Log `adjusted` for each modified resource.
- Blueprint: already covered — cascade deletes `blueprint_affixes` via DB `ON DELETE CASCADE`.

All force-delete actions are atomic (single DB transaction with all side effects).

## Acceptance criteria

- [ ] Delete affix (no references): succeeds
- [ ] Delete affix (referenced by N blueprints): returns 409 with reference details
- [ ] Force delete affix: removes from all pools, adjusts min/max, logs `adjusted`
- [ ] Force delete global: removes all `$ref_id` references from blueprints and affixes, logs `adjusted`
- [ ] Auto-adjustment: first reduces max, then min; never increases (always downsizes)
- [ ] All side effects in single atomic transaction
- [ ] Integration tests for all force-delete scenarios

## Blocked by

- arche-service/blueprints-delete-force-delete.md
- arche-service/affixes-crud.md
- arche-service/global-meta-attributes-crud.md
