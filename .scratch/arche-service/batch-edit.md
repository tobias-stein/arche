---
title: Batch edit — blueprint attributes
status: completed
---

## Parent

Arche API Service PRD (see `.scratch/arche-service/prd.md`)

## What to build

Implement batch attribute editing for blueprints:

- `POST /api/blueprints/batch/edit` — Request:
```json
{
  "blueprint_ids": ["uuid-1", "uuid-2", "uuid-3"],
  "attributes": {
    "damage": { "min": 5, "max": 20, "value_type": "range" }
  }
}
```

Rules:
- Only blueprints that already have the attribute key are updated (blueprints without the key are silently skipped)
- Type mismatch (e.g., blueprint has `damage` as `single` but batch edit sends `range` payload) returns 422 for the entire batch
- Only the specified attribute keys are updated — other attributes on the blueprint are untouched
- `attribute_order` is NOT affected by batch edits (new keys are not added, and order is preserved)

All edits run in a single DB transaction. Return `{ "updated_count": N }`.

## Acceptance criteria

- [ ] Update matching attribute on N blueprints: all N updated
- [ ] Blueprint without the attribute key: silently skipped
- [ ] Type mismatch: entire batch fails with 422
- [ ] Unrelated attributes: preserved
- [ ] attribute_order: unchanged after batch edit
- [ ] Single transaction: partial failure rolls back all
- [ ] Integration tests

## Blocked by

- arche-service/blueprints-crud-core.md
- arche-types/validation-logic.md
