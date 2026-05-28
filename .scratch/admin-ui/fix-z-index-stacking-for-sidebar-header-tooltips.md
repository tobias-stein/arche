---
title: "Fix z-index stacking for sidebar, header, and tooltips"
status: ready-for-agent
---

## What to build

Fix the z-index hierarchy so sidebar tooltips render above the sticky header.

The sidebar (desktop fixed) is at `z-10`, the sticky header is at `z-50`. Sidebar tooltips (shown in collapsed icon mode) are rendered inside the sidebar's DOM subtree at `z-50` — before the header in DOM order. Since both are `z-50`, the header wins via DOM stacking and occludes the tooltip.

## Acceptance criteria

- [ ] Sidebar desktop fixed element has `z-40` (below header/modals, above content/activity panel)
- [ ] `TooltipContent` has `z-[60]` (tooltips always on top of everything)
- [ ] Hovering a sidebar nav icon in collapsed mode shows the tooltip label fully visible above the header
- [ ] Hovering the sidebar footer identity icon in collapsed mode shows the tooltip fully visible

## Blocked by

None — can start immediately

## Files to change

- `admin-ui/src/components/ui/sidebar.tsx` — Sidebar component: change `z-10` to `z-40` on the fixed element (line 246)
- `admin-ui/src/components/ui/tooltip.tsx` — TooltipContent: change `z-50` to `z-[60]` (line 20)
