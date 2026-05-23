---
title: Duplicate / use as template
status: ready-for-agent
---

## Parent

Arche Admin UI PRD (see `.scratch/admin-ui/prd.md`)

## What to build

Implement blueprint duplicate ("Use as template") functionality:

1. **Trigger points**:
   - Row action "Duplicate" in Blueprints List
   - "Duplicate" button in Blueprint Detail toolbar

2. **Behavior**:
   - Calls the API to create a new blueprint with all attributes, affixes, and settings copied from the source
   - The new blueprint's name is the source name appended with " (Copy)"
   - If the copy name already exists, keep appending " (Copy 2)", " (Copy 3)", etc.
   - After creation, opens the new blueprint in the edit modal so the user can immediately modify before saving
   - Alternatively: pre-fill the edit modal with the copied data but don't create until the user saves (avoids orphan copies)

Approach: pre-fill the edit modal (no API call until user saves). This is cleaner UX — ensures only intentional copies.

## Acceptance criteria

- [ ] "Duplicate" button in list row action triggers copy flow
- [ ] "Duplicate" button in detail page toolbar also works
- [ ] Edit modal opens pre-filled with source blueprint's data
- [ ] Name is "(Copy)" suffixed by default (user can change before saving)
- [ ] Creating the copy creates a new blueprint on save
- [ ] Success toast on creation
- [ ] User can cancel without creating an orphan

## Blocked by

- admin-ui/blueprints-list-page.md
- admin-ui/blueprints-create-edit-modal.md
