---
title: Import — resolve & commit
status: completed
---

## Parent

Arche API Service PRD (see `.scratch/arche-service/prd.md`)

## What to build

Implement `POST /api/import/resolve` for applying conflict resolutions to a previously initiated import.

Request:
```json
{
  "import_token": "token-from-409-response",
  "resolutions": {
    "uuid-of-blueprint": {
      "strategy": "per_attribute",
      "attributes": {
        "damage": "keep_new",
        "weight": "keep_old"
      }
    },
    "uuid-of-affix": {
      "strategy": "keep_new"
    }
  }
}
```

Resolution strategies:
- `keep_old` — keep the existing resource's version entirely (skip import for this resource)
- `keep_new` — overwrite with the imported version entirely
- `per_attribute` — per-key resolution within the resource

The import token ties back to the original import's parsed data (stored temporarily, e.g., in memory with a TTL or in a staging table). After resolution, execute the full import in a single DB transaction with one cache invalidation at the end.

If the import token is expired or invalid, return 404.

## Acceptance criteria

- [ ] `keep_old` strategy: existing resource preserved
- [ ] `keep_new` strategy: imported version overwrites
- [ ] `per_attribute` with per-key resolution: correct keys kept/replaced
- [ ] Invalid import token: 404
- [ ] Expired import token: 410 or 404
- [ ] Single DB transaction with one cache invalidation
- [ ] Integration test: export → modify → import (conflict) → resolve → verify correct merge

## Blocked by

- arche-service/import-parse-conflict-detection.md
