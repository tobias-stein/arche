---
title: Missing Global Meta Attributes detail page
status: completed
---

## Bug Description

The Global Meta Attributes list page has no route or page component for a detail view. The PRD specifies route `#/global-meta-attributes/:id` for a read-only detail page with tabs (definition, "Referenced by" list, history).

**In `App.tsx`, only `/global-meta-attributes` is routed — no `:id` sub-route exists.**

The `useGlobalMetaAttribute` hook is implemented in the generated API client but is never used anywhere.

## Steps to Reproduce

1. Navigate to `/global-meta-attributes`
2. Attempt to click on any row to view its detail — rows are not clickable and have no links
3. There is no way to navigate to a GMA detail page

## Expected Behavior

- Route `#/global-meta-attributes/:id` should render a detail page
- The detail page should show attribute definition, a "Referenced by" section listing blueprints/affixes that reference it, and a history tab with audit log entries
- Rows in the GMA list page should link to the detail page (like blueprints and affixes do)

## Acceptance Criteria

- [ ] Route added to `App.tsx`: `<Route path="/global-meta-attributes/:id" element={...} />`
- [ ] Detail page component with tabbed layout (Definition, Referenced By, History)
- [ ] Referenced-by section fetches blueprints and affixes that use this GMA via `$ref_id`
- [ ] History tab shows audit log entries filtered to this GMA
- [ ] List page rows are linked to detail page
- [ ] Edit button on detail page opens the edit dialog

## Related

- PRD user story #25-27
- See `docs/design/ui-ux.md` for detail view patterns
