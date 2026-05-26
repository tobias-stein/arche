---
title: Blueprints — create/edit modal
status: completed
---

## Parent

Arche Admin UI PRD (see `.scratch/admin-ui/prd.md`)

## What to build

Implement the Blueprint create/edit overlay modal with tabbed/stepped form:

1. **General tab**: Name (required), Archetype (required), Weight (required, > 0), Description (optional). React Hook Form + Zod validation.

2. **Attributes tab**: Table of attributes with drag-and-drop reorder (via `@dnd-kit/core`, persisted as `attribute_order`). Each row shows: attribute name, value type badge, preview, source badge (inline vs global). 
   - "Add Attribute" button → two options: "Inline attribute" form (value type selector with dynamic fields) or "From global library" picker (searchable modal)
   - "Delete" button per row with confirmation (can't delete if it's the last attribute)
   - Inline attribute form: type selector (single/enum/range/string/boolean) → dynamic form fields based on type
   - For range: distribution type selector (uniform/normal/exponential) with appropriate fields

3. **Affixes tab**: Min/max prefix/suffix count inputs. Affix pool table with drag reorder and editable weight column. "Add affix" button → searchable affix picker (filtered by prefix/suffix type).

Zod schemas shared between create and edit (edit makes all fields optional). Validation mirrors the API's rules.

Create modal: title "Create Blueprint", empty form. Edit modal: pre-filled with existing data, title "Edit Blueprint".

## Acceptance criteria

- [ ] Modal opens from Blueprints List (Create button) and Detail (Edit button)
- [ ] General tab: all fields with validation
- [ ] Attributes tab: table with drag-and-drop reorder
- [ ] Add inline attribute: dynamic form per value type
- [ ] Add global attribute: searchable picker modal
- [ ] Affixes tab: min/max inputs, pool table with editable weights
- [ ] Add affix to pool: searchable picker filtered by prefix/suffix
- [ ] Zod validation shows inline errors on fields
- [ ] Submit calls correct API endpoint, shows success/error toast
- [ ] Edit modal pre-filled with existing data
- [ ] Loading state during submit

## Blocked by

- admin-ui/api-client-generation.md
- admin-ui/app-shell-layout.md
