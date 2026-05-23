---
title: Delete protection modals + toast notifications
status: ready-for-agent
---

## Parent

Arche Admin UI PRD (see `.scratch/admin-ui/prd.md`)

## What to build

Implement reusable `DeleteResourceDialog` component and toast notification system:

**DeleteResourceDialog:**
- Shows the resource name and type being deleted
- Lists all resources that reference it (if any) with links to their detail pages
- Two buttons: "Cancel" and "Force Delete" (red/danger styled, only shown when references exist)
- "Force Delete" shows additional warning text: "This action cannot be undone. [N] resources will be modified."
- Used by: blueprints, affixes, global meta attributes, clients

**Toast system:**
- shadcn/ui toast component (or `sonner`)
- Success toasts: green, "Blueprint updated", "Affix deleted"
- Error toasts: red, show Problem JSON `detail` field, "Failed to delete affix — it is referenced by 2 blueprints"
- Force delete toasts: amber/warning, "Affix deleted. Removed from 2 blueprints."
- Auto-dismiss after 5 seconds
- Stackable (multiple toasts visible at once)

## Acceptance criteria

- [ ] DeleteResourceDialog shows resource info and referencing resources with links
- [ ] "Force Delete" button only visible when references exist
- [ ] Force delete warning text shown
- [ ] Cancel closes dialog
- [ ] Success toast on successful operations
- [ ] Error toast on failed operations (with API error detail)
- [ ] Force-delete toast shows summary of cascade effects
- [ ] Toasts auto-dismiss after 5s
- [ ] Multiple toasts stack

## Blocked by

- admin-ui/scaffolding.md
- admin-ui/app-shell-layout.md
