---
title: Clients management + API keys
status: completed
---

## Parent

Arche Admin UI PRD (see `.scratch/admin-ui/prd.md`)

## What to build

Implement Clients management pages and API key management:

**Clients List** (`/clients`):
- Table: Name, Created, Key Count
- "Create Client" button → modal with name input
- Row actions: View (→ detail), Delete (with confirmation)

**Client Detail** (`/clients/:id`):
- Name displayed (editable)
- **Keys section**: table of API keys with Name, Permissions (badge list), Created
- "Create Key" button → modal with:
  - Name input
  - Permission checkboxes: read, write, delete, generate, admin
  - On create: shows the raw API key in a highlighted, copyable field (shown exactly once)
  - User must acknowledge they've copied it before closing
- "Revoke Key" button per row → confirmation dialog → key revoked immediately

**Danger Zone**: "Delete Client" section with red styling. Shows warning text about cascading effects. Requires confirmation (type client name to confirm). Blocked if client has active keys (API shows 409).

## Acceptance criteria

- [ ] Client list table renders with all clients
- [ ] Create client: modal, success toast, list refreshes
- [ ] Client detail: name, key table, danger zone
- [ ] Create key: permission checkboxes, displays raw key once
- [ ] Copy key button or select-all for easy copying
- [ ] Revoke key: confirmation → key revoked immediately
- [ ] Delete client: confirmation modal, 409 handling if keys exist
- [ ] All operations: loading states, success/error toasts

## Blocked by

- admin-ui/api-client-generation.md
- admin-ui/app-shell-layout.md
- arche-service/client-crud.md
- arche-service/api-key-management.md
