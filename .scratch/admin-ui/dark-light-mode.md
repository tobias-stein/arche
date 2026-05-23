---
title: Dark/light mode toggle
status: ready-for-agent
---

## Parent

Arche Admin UI PRD (see `.scratch/admin-ui/prd.md`)

## What to build

Implement dark/light mode toggle across the app:

1. **Theme Zustand store**: `mode: 'light' | 'dark'`, `toggle()`, `setMode()`. Persisted in localStorage under key `arche-theme`.

2. **Toggle UI**: an icon button (sun/moon) in the persistent header. Also available inside the Navigation Drawer under Settings.

3. **Application**: Tailwind's `class` strategy — add/remove `dark` class on `<html>` element. All shadcn/ui components automatically respond to the class.

4. **Initial load**: read from localStorage on mount. If no saved preference, check `prefers-color-scheme` media query. Default to light.

The toggle is smooth and the transition is animated via CSS `transition` on colors.

## Acceptance criteria

- [ ] Toggle button in header switches between light and dark
- [ ] Toggle also available in Navigation Drawer settings
- [ ] Mode persisted across page reloads (localStorage)
- [ ] Respects system `prefers-color-scheme` on first visit
- [ ] All shadcn/ui components render correctly in both modes
- [ ] No flash of wrong theme on page load
- [ ] Dark mode class correctly applied to `<html>`

## Blocked by

- admin-ui/app-shell-layout.md
