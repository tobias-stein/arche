---
title: Affixes — list page
status: ready-for-agent
---

## Parent

Arche Admin UI PRD (see `.scratch/admin-ui/prd.md`)

## What to build

Implement the Affixes list page:

1. **Table columns**: checkbox (multi-select), Name, Type (prefix/suffix badge), Attribute Name, Value Type (badge), Updated

2. **Filter bar**: type dropdown (All / Prefix / Suffix), text search by name

3. **Pagination**: offset-based, same pattern as blueprints list

4. **Row actions**: Edit (opens modal), Delete (with confirmation)

5. **Batch toolbar**: Batch Delete, Batch Assign to Blueprints (opens blueprint picker)

Follows the same patterns as the Blueprints list page (table, multi-select, pagination, skeletons, empty state, responsive card layout).

## Acceptance criteria

- [ ] Table renders with affix data from API
- [ ] Filter by type (prefix/suffix) works
- [ ] Search by name works
- [ ] Pagination works
- [ ] Multi-select with shift-click range
- [ ] Row actions: Edit, Delete
- [ ] Batch toolbar: Batch Delete, Batch Assign to Blueprints
- [ ] Loading skeletons, empty state
- [ ] Responsive card layout on mobile

## Blocked by

- admin-ui/api-client-generation.md
- admin-ui/app-shell-layout.md
