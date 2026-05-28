---
title: "Fix sidebar collapse to icon width and shift main content"
status: completed
---

## What to build

Two related layout fixes:

**A. Sidebar spacer not shifting content.** The sidebar wrapper (flex child in the layout) has no `flex-shrink-0`, so the flex algorithm can shrink it below the spacer's explicit width. This means the main content wrapper (`flex-1`) can take the full viewport width, and the fixed sidebar at `z-10` overlays the main content. Add `shrink-0` to the sidebar wrapper to prevent this.

**B. Sidebar header/footer overflow when collapsed to icon mode.** The client switcher in `SidebarHeader` renders a `DropdownMenuTrigger` with an icon, a client-name `<span>`, and a `ChevronDown` chevron. When the sidebar collapses to `--sidebar-width-icon` (48px), these text/chevron elements are still rendered and overflow the sidebar boundary. This makes the sidebar appear wider than 48px, and the overflow bleeds into the main content area.

Fix: hide the text `<span>` and `ChevronDown` in the `SidebarHeader` and `SidebarFooter` when the sidebar is in collapsed state using `group-data-[collapsible=icon]:hidden` on each element.

## Acceptance criteria

- [ ] Sidebar wrapper has `shrink-0` class to prevent flex shrink
- [ ] Main content (both `<header>` and `<main>` children of the content wrapper) starts at the sidebar spacer's right edge when sidebar is expanded (256px) and collapsed (48px)
- [ ] Client switcher in `SidebarHeader` shows only the `Building2` icon when sidebar is collapsed; clicking the icon still opens the client dropdown
- [ ] Client name `<span>` and `ChevronDown` are hidden when collapsed
- [ ] Non-super-admin "scoped" badge and client name in sidebar header are hidden when collapsed (icon + tooltip only)
- [ ] Sidebar footer identity `DropdownMenuTrigger` text is hidden when collapsed (already uses `SidebarMenuButton` with `tooltip`, but verify)

## Blocked by

None — can start immediately

## Files to change

- `admin-ui/src/components/ui/sidebar.tsx` — Add `shrink-0` to the sidebar wrapper (line 227: `className="group peer hidden text-sidebar-foreground md:block"`)
- `admin-ui/src/components/AppSidebar.tsx` — Add `group-data-[collapsible=icon]:hidden` to:
  - Client name `<span>` inside the super admin `DropdownMenuTrigger` (line 80)
  - `ChevronDown` inside the super admin `DropdownMenuTrigger` (line 87)
  - Client name `<span>` inside the scoped-key non-super-admin block (line 108)
  - `Badge` ("scoped") inside the scoped-key block (line 109)
  - "Loading..." `<span>` for the loading state (line 114)
