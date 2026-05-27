---
title: Fix navigation drawer bottom items pushed off-screen
status: ready-for-agent
---

## What to build

In the navigation drawer (`NavigationDrawer.tsx`), the bottom section (API Keys, Logout, Theme toggle) is pinned to the bottom using a flex layout, but when the nav links don't fill the viewport, `flex-1` on the nav flexbox consumes all available space and the bottom section is pushed below the visible area — requiring a scroll to reach.

Fix the layout so the bottom section stays pinned to the visible bottom of the drawer at all times:

- Keep the drawer as a `flex flex-col` container
- Add `mt-auto` to the bottom section (`div.border-t`) so it sticks to the bottom of the flex container
- Keep `overflow-y-auto` on the nav section so links scroll if they exceed the available space
- Remove `flex-1` from the `<nav>` element

## Acceptance criteria

- [ ] API Keys, Logout, and Theme buttons are always visible at the bottom of the open drawer without scrolling
- [ ] When there are many nav items, the nav section scrolls normally
- [ ] No visual regressions on mobile or desktop

## Blocked by

None — can start immediately.
