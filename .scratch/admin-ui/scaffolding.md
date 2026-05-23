---
title: Admin UI scaffolding
status: ready-for-agent
---

## Parent

Arche Admin UI PRD (see `.scratch/admin-ui/prd.md`)

## What to build

Scaffold a new Vite + React + TypeScript project with Tailwind CSS v4 and shadcn/ui components. Set up:

- Vite config with React plugin and TypeScript
- Tailwind CSS v4 with shadcn/ui preset and dark mode (`class` strategy)
- `tsconfig.json` with strict mode and path aliases (`@/`)
- Hash-based React Router (for zero server-config deployment)
- shadcn/ui base components: Button, Input, Card, Table, Dialog, Modal, Badge, Toast, DropdownMenu, Tabs, Skeleton, Command (cmdk)
- `components.json` for shadcn/ui configuration
- Placeholder routes: `/login`, `/dashboard`, `/blueprints`, `/affixes`, `/global-meta-attributes`, `/clients`, `/audit-log`, `/export`, `/import`
- Empty Zustand stores for auth, theme, and UI state (drawer/panel toggles)
- `src/lib/utils.ts` with `cn()` helper (standard shadcn/ui pattern)

The app compiles and renders a minimal app shell with route placeholders.

## Acceptance criteria

- [ ] `npm run dev` starts the dev server, shows a basic page
- [ ] All shadcn/ui base components are installed and importable
- [ ] Hash-based router with all placeholder routes defined
- [ ] Tailwind dark mode class strategy configured
- [ ] Zustand stores scaffolded (auth, theme, ui)
- [ ] `npm run build` produces a static SPA with zero errors
- [ ] TypeScript strict mode enabled

## Blocked by

None - can start immediately
