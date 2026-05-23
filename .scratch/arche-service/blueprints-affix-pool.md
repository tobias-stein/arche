---
title: Blueprints — affix pool management
status: ready-for-agent
---

## Parent

Arche API Service PRD (see `.scratch/arche-service/prd.md`)

## What to build

Extend the blueprint create/update to include affix pool configuration and manage the `blueprint_affixes` join table.

The affix pool is defined inline on the blueprint payload:
```json
{
  "affixes": {
    "min_prefixes": 0,
    "max_prefixes": 1,
    "min_suffixes": 0,
    "max_suffixes": 2,
    "prefixes": [
      { "affix_id": "uuid", "weight": 1.0 }
    ],
    "suffixes": [
      { "affix_id": "uuid", "weight": 0.5 }
    ]
  }
}
```

On create, insert `blueprint_affixes` rows for each affix in the pool. On update, diff the pool: remove rows for removed affixes, insert rows for new ones, update weights for existing ones.

Min/max counts are validated: min ≥ 0, max ≥ min.

## Acceptance criteria

- [ ] Create blueprint with affix pool creates `blueprint_affixes` rows
- [ ] Update blueprint affix pool adds/removes/updates rows correctly
- [ ] Invalid affix_id in pool returns 404
- [ ] Invalid min/max counts return 400
- [ ] Affix from wrong client returns 404 (cannot reference affix from another client)
- [ ] Duplicate affix in pool returns 409
- [ ] Integration tests

## Blocked by

- arche-service/blueprints-crud-core.md
- arche-service/affixes-crud.md
