---
title: Global search (Cmd+K)
status: completed
---

## Parent

Arche Admin UI PRD (see `.scratch/admin-ui/prd.md`)

## What to build

Implement global search via Cmd+K command palette:

1. **Trigger**: Cmd+K (Mac) or Ctrl+K (Windows/Linux) from anywhere in the app. Also an icon button in the header.

2. **Command palette** (using `cmdk` library): modal overlay with a search input

3. **Search behavior**: searches blueprints and affixes by name. Two approaches (choose one or both):
   - Client-side: fetch all names on load, filter locally
   - API-driven: call list endpoints with `search` param

4. **Results**: grouped by type ("Blueprints" section, "Affixes" section). Each result shows: name, type icon/badge, archetype (for blueprints) or prefix/suffix (for affixes).

5. **Navigation**: keyboard navigation (↑↓ arrows), Enter to navigate to the selected item's detail page. Escape to close.

6. **Empty state**: "No results found" when search yields nothing.

The palette should feel fast — debounce input before searching. Cache results in memory for the session.

## Acceptance criteria

- [x] Cmd+K opens the search palette from any page
- [x] Header search icon also opens palette
- [x] Typing shows grouped results (Blueprints, Affixes)
- [x] Each result shows name and type info
- [x] Keyboard navigation (arrows + enter) navigates to detail page
- [x] Escape closes palette
- [x] Empty state when no results
- [x] Debounced search (don't fire on every keystroke)
- [x] Results cached for session

## Blocked by

- admin-ui/api-client-generation.md
- admin-ui/app-shell-layout.md
