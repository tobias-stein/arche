---
title: Affixes — create/edit modal & detail page
status: completed
---

## Parent

Arche Admin UI PRD (see `.scratch/admin-ui/prd.md`)

## What to build

Implement the Affix create/edit modal and detail page:

**Create/Edit Modal:**
- Name (required), Type (prefix/suffix toggle, required), Description (optional)
- Attribute section: either "Inline attribute" (value type selector with dynamic form) or "From global library" (searchable picker)
- Same attribute dynamic forms as blueprint attributes (range with distribution, enum with values, etc.)
- Validation: exactly one attribute must be defined, can't have both inline and global ref

**Detail Page** (`/affixes/:id`):
- Read-only, shareable URL
- Name, Type badge, Description
- Attribute section: inline badge + value preview, or global badge + link to global detail
- History tab: filtered audit log for this affix with expandable diff
- "Edit" button → opens Edit Affix modal

## Acceptance criteria

- [ ] Create modal: name, type toggle, inline attribute form
- [ ] Create modal: "From global library" picker works
- [ ] Edit modal pre-filled with existing data
- [ ] Detail page renders all affix fields
- [ ] Attribute source badge (inline vs global) with link
- [ ] History tab with filtered audit log and diff
- [ ] Edit button opens pre-filled modal
- [ ] 404 state for non-existent affix
- [ ] Loading skeleton during data fetch

## Blocked by

- admin-ui/api-client-generation.md
- admin-ui/app-shell-layout.md
- admin-ui/affixes-list-page.md
