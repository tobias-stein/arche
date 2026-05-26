---
title: Global Meta Attributes list page missing create button and modal
status: completed
---

## Bug Description

The Global Meta Attributes list page (`src/pages/GlobalMetaAttributes.tsx`) has an edit dialog but no "Create" button or create modal. PRD user story #25 says: "As an admin, I want to create a global meta attribute via an overlay modal with a value-type selector (single/enum/range/string/boolean) and dynamic form fields based on the selected type."

## Steps to Reproduce

1. Navigate to `/global-meta-attributes`
2. Look for a "Create" or "New" button — there is none
3. No overlay modal exists for creating new global meta attributes

## Expected Behavior

- A "Create Global Meta Attribute" button should be visible above the table
- Clicking it opens an overlay modal with:
  - Name field
  - Description field (optional)
  - Value type selector (single/enum/range/string/boolean)
  - Dynamic form fields based on selected value type (matching the create endpoint's `AttributePayload` shape)
- On submit, calls `useCreateGlobalMetaAttribute` mutation
- On success, invalidates the list query and shows a success toast

## Acceptance Criteria

- [ ] "Create Global Meta Attribute" button added to the list page header
- [ ] Create modal with value-type selector and dynamic fields
- [ ] Form validates required fields before submit
- [ ] API call via `useCreateGlobalMetaAttribute` mutation
- [ ] Cache invalidation and toast on success

## Related

- PRD user story #25
- See existing create patterns in `BlueprintFormModal.tsx` and `AffixCreateEditDialog.tsx`
