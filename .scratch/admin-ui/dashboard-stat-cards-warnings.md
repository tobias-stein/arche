---
title: Dashboard — stat cards + warnings
status: completed
---

## Parent

Arche Admin UI PRD (see `.scratch/admin-ui/prd.md`)

## What to build

Implement the Dashboard page with stat cards and warnings:

1. **Stat cards**: three cards in a row — Total Blueprints, Total Affixes, Warnings Count. Each card shows a number and an icon. Fetched from summary API endpoints or computed from list endpoints.

2. **Warnings list**: a section below the stat cards that lists all detected misconfigurations. Each warning has a title, description, severity indicator, and a link to fix the issue. Warning types:
   - Blueprint min prefix/suffix > 0 but the pool is empty or insufficient
   - Zero-weight blueprint or affix assignment
   - Dangling `$ref_id` (references a non-existent global meta attribute)
   - Invalid attribute payload (range min > max, empty enum, etc.)
   - Invalid distribution config (std_dev = 0, rate = 0)

Warnings are fetched from a dedicated endpoint or computed client-side from the loaded data. Each warning is dismissable (per session) or persists until resolved.

## Acceptance criteria

- [ ] Three stat cards render with correct counts
- [ ] Stat numbers update on page refresh
- [ ] Warnings section lists all detected misconfigurations
- [ ] Each warning has a clear description and link to the relevant resource
- [ ] Zero warnings: empty state with "No warnings" message
- [ ] Loading skeletons for stats and warnings
- [ ] Dashboard is the default post-login redirect

## Blocked by

- admin-ui/api-client-generation.md
- admin-ui/app-shell-layout.md
