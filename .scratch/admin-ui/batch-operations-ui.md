---
title: Batch operations UI
status: completed
---

## Parent

Arche Admin UI PRD (see `.scratch/admin-ui/prd.md`)

## What to build

Implement batch operation UI components for blueprints and affixes:

**Batch Delete:**
- Triggered from batch toolbar on Blueprints List or Affixes List
- Confirmation dialog listing the resources to be deleted (name, type)
- Confirm → calls batch delete API → success toast with count

**Batch Assign (affixes to blueprints):**
- Two entry points:
  1. From Blueprints List: select blueprints → click "Batch Assign" → opens affix picker modal (searchable, multi-select affixes) → confirm
  2. From Affixes List: select affixes → click "Assign to Blueprints" → opens blueprint picker modal → confirm
- Success toast: "Assigned N affixes to M blueprints"

**Batch Edit (blueprint attributes):**
- Select multiple blueprints → click "Batch Edit"
- Modal shows a merged attribute view: only attributes that exist on ALL selected blueprints AND have the same value type are shown
- Fields are blank if values differ across blueprints
- User edits attribute values, submits
- Calls batch edit API → success toast with count of updated blueprints

All batch operations use TanStack Query mutations with automatic cache invalidation on success.

## Acceptance criteria

- [ ] Batch delete: selects, confirms, deletes, shows count
- [ ] Batch assign from blueprints: select blueprints → pick affixes → confirm
- [ ] Batch assign from affixes: select affixes → pick blueprints → confirm
- [ ] Batch edit: merged attribute view, only matching types shown
- [ ] Blank fields for differing values across blueprints
- [ ] All operations show loading states during mutation
- [ ] Success/error toasts after completion
- [ ] Table multi-select cleared after batch operation

## Blocked by

- admin-ui/blueprints-list-page.md
- admin-ui/affixes-list-page.md
- arche-service/batch-delete.md
- arche-service/batch-assign.md
- arche-service/batch-edit.md
