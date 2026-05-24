---
title: Audit log — full-page view
status: completed
---

## Parent

Arche Admin UI PRD (see `.scratch/admin-ui/prd.md`)

## What to build

Implement the Audit Log full-page view:

Route: `/audit-log`

1. **Filter bar**: 
   - Client dropdown (super admin only)
   - Resource type dropdown (all, blueprint, affix, global_meta_attribute, client, api_key)
   - Action dropdown (all, created, updated, deleted, force_deleted, adjusted)
   - Date range picker (from/to)
   - Apply / Reset buttons

2. **Table columns**: Timestamp, Actor Name, Client, Resource Type, Action (badge), Resource Name/ID

3. **Row expansion**: click a row to expand and show the before/after JSON diff. Uses `react-diff-viewer-continued` for side-by-side, color-coded display.

4. **Pagination**: cursor-based or offset-based

Timestamps displayed in browser's local timezone with a UTC tooltip on hover.

## Acceptance criteria

- [ ] Table renders with audit log entries
- [ ] All filters work independently and combined
- [ ] Date range picker works
- [ ] Row expansion shows side-by-side JSON diff with color coding
- [ ] Timestamps show local time with UTC tooltip
- [ ] Pagination works
- [ ] Loading skeleton during fetch
- [ ] Empty state when no entries match
- [ ] Responsive: diff view collapses to inline on mobile

## Blocked by

- admin-ui/api-client-generation.md
- admin-ui/app-shell-layout.md
- arche-service/audit-log-query-endpoint.md
