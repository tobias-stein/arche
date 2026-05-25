---
title: Dashboard — Quick Generate playground
status: completed
---

## Parent

Arche Admin UI PRD (see `.scratch/admin-ui/prd.md`)

## What to build

Implement the Quick Generate playground section on the Dashboard:

1. **Archetype dropdown**: populated from distinct blueprint archetypes in the current client. User selects an archetype or "Any".

2. **Constraint builder**: dynamic form that lets users add attribute constraints. Each constraint row has:
   - Attribute key dropdown (populated from available attributes across all blueprints)
   - Operator selector (gte, lte, in, contains, eq — appropriate to the value type)
   - Value input (type-appropriate: number, text, multi-select for enum)
   - Remove button
   - "Add constraint" button to add more rows

3. **Affix controls**: min/max prefix/suffix inputs, require/block affix multiselects (searchable)

4. **Seed input**: optional text field for u64 seed

5. **Generate button**: calls `POST /api/generate` with the configured constraints

6. **Result display**: inline preview showing the generated thing's name, blueprint attributes (in order), and affix attributes. Rendered as a formatted JSON block or a structured card.

Loading state during generation. Error state if no blueprints match. "Clear" button to reset the form.

## Acceptance criteria

- [ ] Archetype dropdown populated from API data
- [ ] Add/remove constraint rows dynamically
- [ ] Operator & value inputs adapt to attribute value type
- [ ] Seed input (optional)
- [ ] Generate button calls API and shows result inline
- [ ] "No matching blueprints" error shown gracefully
- [ ] Result shows name, blueprint attributes, affix attributes
- [ ] Clear button resets the form
- [ ] Responsive (desktop-first for this iteration)

## Blocked by

- admin-ui/api-client-generation.md
- admin-ui/app-shell-layout.md
- arche-service/generate-endpoint.md
