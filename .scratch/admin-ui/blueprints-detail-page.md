---
title: Blueprints — detail page
status: completed
---

## Parent

Arche Admin UI PRD (see `.scratch/admin-ui/prd.md`)

## What to build

Implement the Blueprint detail page with read-only, shareable URL and tabbed layout:

Route: `/blueprints/:id`

**Tab 1 — General:**
- Name, Archetype, Weight, Description (display only)
- "Edit" button → opens Edit Blueprint modal
- "Duplicate" button → creates copy with "(Copy)" suffix, opens edit modal

**Tab 2 — Attributes:**
- Table of attributes sorted by `attribute_order`
- Each row: Name, Value Type (badge), Preview (formatted value), Source
- Source column: "inline" badge (gray) for inline attribs, "global" badge (blue, clickable → global detail page) for `$ref_id`
- For `$ref_id` attributes: show the resolved values from the global definition in a lighter color as a preview

**Tab 3 — Affixes:**
- Two sections: Prefixes and Suffixes
- Each section: min/max count display, affix pool table with Name (clickable → affix detail), Weight
- Empty state if no affixes configured

**Tab 4 — History:**
- Filtered audit log for this blueprint (resource_type = "blueprint", resource_id = current ID)
- Table: Timestamp, Actor, Action, with expandable row to show before/after diff
- Diff view using `react-diff-viewer-continued` (side-by-side, color-coded)

## Acceptance criteria

- [ ] URL `/blueprints/:id` loads the correct blueprint
- [ ] General tab renders all blueprint fields
- [ ] Attributes tab: ordered table with source badges (inline vs global links)
- [ ] Affixes tab: prefixes/suffixes sections with pool table
- [ ] History tab: filtered audit log with expandable diff rows
- [ ] Edit button opens Edit Blueprint modal (pre-filled)
- [ ] Duplicate button creates copy and opens edit modal
- [ ] Loading skeleton during data fetch
- [ ] 404 state if blueprint not found
- [ ] Shareable URL: copy-paste works, loads same view

## Blocked by

- admin-ui/api-client-generation.md
- admin-ui/app-shell-layout.md
- admin-ui/blueprints-list-page.md
