---
title: Global meta attributes — create/edit modal & detail page
status: ready-for-agent
---

## Parent

Arche Admin UI PRD (see `.scratch/admin-ui/prd.md`)

## What to build

Implement the Global Meta Attribute create/edit modal and detail page:

**Create/Edit Modal:**
- Name (required), Description (optional)
- Value type selector: Single / Enum / Range / String / Boolean → dynamic form fields based on type:
  - Enum: textarea or tag input for values
  - Range: min/max number inputs, distribution selector (uniform/normal with mean/std_dev, exponential with rate)
  - Single: default value input
  - String: default value input (optional)
  - Boolean: no extra fields

**Detail Page** (`/global-meta-attributes/:id`):
- Read-only, shareable URL
- Name, Description, Value type badge with type-specific payload display
- "Referenced by" section: two tables — blueprints that reference it (Name → link), affixes that reference it (Name → link)
- History tab: filtered audit log with diff
- "Edit" button → opens Edit modal

## Acceptance criteria

- [ ] Create modal: name, description, value type selector with dynamic form
- [ ] Range: min/max, distribution type selector with conditional fields
- [ ] Enum: tag input for entering values
- [ ] Edit modal pre-filled with existing data
- [ ] Detail page: all fields rendered
- [ ] "Referenced by" section: tables with links to referencing blueprints and affixes
- [ ] History tab: filtered audit log with diff
- [ ] Edit button opens pre-filled modal
- [ ] 404 state, loading skeleton

## Blocked by

- admin-ui/api-client-generation.md
- admin-ui/app-shell-layout.md
- admin-ui/global-meta-attributes-list.md
