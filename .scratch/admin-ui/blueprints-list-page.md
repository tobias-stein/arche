---
title: Blueprints — list page
status: completed
---

## Parent

Arche Admin UI PRD (see `.scratch/admin-ui/prd.md`)

## What to build

Implement the Blueprints list page with table, search, filter, pagination, and multi-select:

1. **Table columns**: checkbox (multi-select), Name (clickable → detail page), Archetype, Weight, Updated (timestamp)

2. **Multi-select**: per-row checkbox. Shift-click range select (selects all rows between last clicked checkbox and current). Header checkbox selects/deselects all visible rows.

3. **Filter bar**: archetype dropdown (populated from distinct blueprint archetypes in current client), text search by name

4. **Pagination**: offset-based with page numbers (`< Prev 1 2 3 ... Next >`), page size selector (10/25/50/100)

5. **Row actions**: Edit (opens modal), Duplicate, Delete (with confirmation)

6. **Batch toolbar**: appears when ≥1 rows selected. Shows count, Batch Delete, Batch Assign, Batch Edit buttons (disabled unless appropriate)

7. **Loading state**: skeleton rows while data fetches. Empty state with helpful text when no blueprints.

Uses TanStack Query for data fetching with automatic cache invalidation after mutations.

## Acceptance criteria

- [ ] Table renders with blueprint data from API
- [ ] Sort by name/weight/updated (click column headers)
- [ ] Multi-select: checkbox per row, shift-click range select, header checkbox
- [ ] Archetype filter: dropdown filters table
- [ ] Search: text input filters by name
- [ ] Pagination: page numbers, page size selector, prev/next
- [ ] Row actions: Edit, Duplicate, Delete buttons visible
- [ ] Batch toolbar appears when ≥1 rows selected
- [ ] Loading skeletons during fetch
- [ ] Empty state when no blueprints
- [ ] Responsive: table collapses to stacked cards on mobile

## Blocked by

- admin-ui/api-client-generation.md
- admin-ui/app-shell-layout.md
