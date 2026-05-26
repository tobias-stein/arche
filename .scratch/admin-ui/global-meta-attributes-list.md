---
title: Global meta attributes — list page
status: completed
---

## Parent

Arche Admin UI PRD (see `.scratch/admin-ui/prd.md`)

## What to build

Implement the Global Meta Attributes list page:

1. **Table columns**: Name, Value Type (badge), Preview (first few values or range), Usage Count (# of blueprints + affixes referencing it)

2. **Filter bar**: value type dropdown (All / Single / Enum / Range / String / Boolean), text search by name

3. **Pagination**: offset-based

4. **Row actions**: Edit (opens modal), Delete (with confirmation dialog showing usage count)

5. **Usage count**: fetched from API or computed from relationship data — shows number of blueprints and affixes that reference this global attribute via `$ref_id`

Follows same patterns as other list pages (multi-select not needed here — no batch operations for globals in the PRD).

## Acceptance criteria

- [x] Table renders with all global meta attributes
- [x] Value type filter works (dropdown)
- [x] Search by name works
- [x] Usage count column shows correct references (blueprints + affixes)
- [x] Row actions: Edit, Delete
- [x] Delete shows confirmation with usage count details
- [x] Pagination works
- [x] Loading skeletons, empty state

## Blocked by

- admin-ui/api-client-generation.md
- admin-ui/app-shell-layout.md
