---
title: App shell — header + navigation drawer + content area
status: ready-for-agent
---

## Parent

Arche Admin UI PRD (see `.scratch/admin-ui/prd.md`)

## What to build

Implement the persistent app shell layout wrapping all authenticated routes:

1. **Persistent Header** (top bar):
   - Logo / app name (left)
   - Global Search bar (Cmd+K) — placeholder for now
   - Dark/light mode toggle (icon button)
   - Navigation Drawer toggle (hamburger button)
   - Activity Panel toggle button

2. **Navigation Drawer** (left side, toggleable):
   - Client switcher dropdown (populated from `/api/clients` for super admin)
   - Navigation links: Dashboard, Blueprints, Affixes, Global Meta Attributes, Audit Log, Import, Export
   - API key management link (navigates to current client's keys view)
   - Login/Logout
   - Settings (dark mode toggle)

3. **Content area** — React Router `<Outlet />` between the drawer and activity panel

4. **Activity Panel** (right side, toggleable) — placeholder stub

Responsive behavior: on mobile, the drawer overlays content as a full-width slide-out. On desktop, it optionally pushes the content. The activity panel defaults as a floating overlay.

All toggle states (drawer open/closed, panel open/closed) are managed in the UI Zustand store.

## Acceptance criteria

- [ ] Header renders on all authenticated pages
- [ ] Navigation Drawer opens/closes, shows all nav links
- [ ] Client switcher dropdown is populated (super admin only)
- [ ] Activity Panel placeholder renders when toggled
- [ ] Drawer overlays on mobile, pushes on desktop
- [ ] Activity Panel floats by default, hides on outside click
- [ ] All states managed via Zustand (persistent across route changes)
- [ ] Responsive: test at 375px, 768px, 1440px widths

## Blocked by

- admin-ui/scaffolding.md
- admin-ui/api-client-generation.md
- admin-ui/auth-login-page.md
