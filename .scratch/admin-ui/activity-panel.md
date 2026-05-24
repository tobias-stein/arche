---
title: Activity panel
status: completed
---

## Parent

Arche Admin UI PRD (see `.scratch/admin-ui/prd.md`)

## What to build

Implement the Activity Panel — a toggleable right-side panel that shows recent audit log entries:

**Behavior:**
- Toggle button in the persistent header
- Default: floating overlay panel that appears on top of the content and hides when clicking outside
- Optional: user can "pin" the panel so it docks to the viewport and stays visible across page navigations

**Content:**
- Shows the most recent audit log entries for the current client
- Each entry: timestamp (relative: "2 min ago"), actor, action badge, resource type, resource name
- Click an entry → navigate to the resource's detail page
- "View all" link at the bottom → navigate to full Audit Log page

**State management:**
- Panel open/closed state in Zustand UI store
- Pin state in Zustand (persisted in localStorage)
- Auto-refresh on navigation events
- Panel width: ~350px default

## Acceptance criteria

- [ ] Activity Panel toggles open/closed from header button
- [ ] Default: floating overlay, hides on outside click
- [ ] Pin/dock: panel stays visible, persists across navigations
- [ ] Shows recent audit entries for current client
- [ ] Relative timestamps ("2 min ago")
- [ ] Click entry navigates to detail page
- [ ] "View all" link navigates to full audit log
- [ ] Panel state managed in Zustand
- [ ] Pin state persists in localStorage

## Blocked by

- admin-ui/app-shell-layout.md
- arche-service/audit-log-query-endpoint.md
