---
title: Replace NavigationDrawer with shadcn sidebar
status: completed
---

## What to build

Replace the current custom `NavigationDrawer` component with the official shadcn/ui sidebar (`collapsible="icon"` mode) that matches the reference at https://ui.shadcn.com/docs/components/base/sidebar.

**Structure:**

- **SidebarHeader** — Client switcher (super admin: dropdown listing all clients; scoped keys: read-only client name badge). Previously in `HeaderClientSelector` in the header — **remove** the duplicate from the header.
- **SidebarContent** — Nav links via `SidebarMenu`/`SidebarMenuButton` with `isActive` for active route highlighting:
  - Dashboard, Blueprints, Affixes, Global Meta Attributes, Audit Log, Import, Export, API Keys (in that order)
  - Import/Export hidden for non-super-admin
  - API Keys at the bottom of the nav list
  - On `collapsible="icon"` collapse, only icons are visible
- **SidebarFooter** — Current API key identity: show `keyName` from auth store (or "Super Admin" for super admin keys). Clicking opens a DropdownMenu with a "Logout" option.

**Header changes:**
- Replace hamburger `Menu` button with shadcn's `<SidebarTrigger />` (renders `PanelLeft` icon)
- Remove `HeaderClientSelector` from the header (moved to sidebar header)
- Dark/light mode toggle stays in the header only (not moved to sidebar)

**Side effects:**
- Remove `drawerOpen`, `toggleDrawer`, `closeDrawer` from the Zustand UI store (shadcn sidebar manages its own state)
- Remove `NavigationDrawer.tsx` and `NavigationDrawer.test.tsx`
- Update `AppShell.tsx` to wrap with `<SidebarProvider>` and use `<SidebarTrigger>`
- Update `AppShell.test.tsx` to work with the new sidebar

**Dependencies to install:**
- `npx shadcn@latest add sidebar` — installs the sidebar, tooltip, collapsible, and sheet components
- `@radix-ui/react-tooltip`, `@radix-ui/react-collapsible` (included by sidebar add)

## Acceptance criteria

- [ ] Sidebar renders on all authenticated pages
- [ ] Client switcher dropdown appears in sidebar header (super admin only)
- [ ] Scoped-key users see their client name in the sidebar header
- [ ] All nav links render with correct hrefs and active state
- [ ] Import/Export hidden for non-super-admin
- [ ] Sidebar collapses to icons on desktop (`collapsible="icon"`)
- [ ] Sidebar slides as overlay on mobile
- [ ] SidebarTrigger toggles sidebar open/closed
- [ ] User identity shown in footer with Logout option
- [ ] No client selector duplicate in the header
- [ ] Theme toggle remains in the header only
- [ ] `drawerOpen`/`toggleDrawer`/`closeDrawer` removed from Zustand store
- [ ] Old `NavigationDrawer.tsx` and its test file deleted
- [ ] All existing tests pass (AppShell, store, etc.)

## Blocked by

None - can start immediately
