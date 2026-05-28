---
title: Arche Admin UI
status: completed
---

## Problem Statement

Game designers and developers using Arche have no visual interface to manage their procedural-generation configuration. They must interact with the REST API directly — crafting raw JSON payloads for blueprints, affixes, global meta attributes, import/export, and generation testing. This is error-prone, slow, and excludes non-technical team members. The CLI tool helps but still requires terminal proficiency.

Without an admin UI, onboarding new games to Arche requires a developer familiar with the API. Designers cannot independently adjust drop tables, tweak attribute distributions, or preview generation results.

## Solution

A single-page React admin application (the **Arche Admin UI**) that provides a full CRUD interface for all Arche resources: blueprints, affixes, global meta attributes, clients, API keys, and audit logs. It also includes a Quick Generate playground for ad-hoc testing, import/export wizards, warnings about misconfigured resources, and a global search bar.

The UI follows a consistent three-view pattern per resource: a filterable list page, a read-only detail page (shareable URL, tabbed), and create/edit overlay modals. A left-side Navigation Drawer houses global actions (client switcher, API key management, login/logout, settings). A right-side Activity Panel shows the audit trail. The app is responsive across mobile, tablet, and desktop, supports dark/light mode, and uses smooth animations throughout.

## User Stories

1. As an admin, I want to log in with my super admin API key, so that I can access the admin UI securely.
2. As an admin, I want my API key to be stored only in memory (not localStorage), so that it is automatically cleared when I close the browser tab.
3. As an admin, I want to see a dashboard with stat cards (total blueprints, affixes, warnings), so that I get a quick overview of my game's configuration health.
4. As an admin, I want to use a Quick Generate playground on the dashboard, so that I can test generation with ad-hoc constraints and see results inline without writing API calls.
5. As an admin, I want to see misconfiguration warnings on the dashboard (e.g., blueprints with min prefix/suffix > 0 but empty pool, zero-weight assignments, dangling `$ref_id`, invalid attribute payloads, invalid distribution configs), so that I can fix issues before they affect generation.
6. As an admin, I want to browse all blueprints in a paginated table with search and archetype filtering, so that I can find specific blueprints quickly.
7. As an admin, I want to multi-select blueprints with shift-click range select and per-row checkboxes, so that I can perform batch operations efficiently.
8. As an admin, I want to create a new blueprint via an overlay modal with tabbed/stepped sections for General info, Attributes (with drag-and-drop reorder), and Affixes pool, so that I can define all aspects of a blueprint in one flow.
9. As an admin, I want to view a blueprint's detail page (read-only, shareable URL) with tabs for General, Attributes, Affixes, and History, so that I can review or share a blueprint's full configuration.
10. As an admin, I want to see each attribute's source (inline badge vs. global `$ref_id` badge with link to the global definition) on the blueprint detail page, so that I understand where each attribute value comes from.
11. As an admin, I want to edit a blueprint via a pre-filled modal opened from either the list or detail page, so that I can update any aspect of a blueprint.
12. As an admin, I want to duplicate a blueprint as a template starting point (with "(Copy)" suffix and opened in edit mode), so that I can rapidly create variants of existing blueprints.
13. As an admin, I want to delete a blueprint with a confirmation dialog that shows what references it, so that I understand the impact before deleting.
14. As an admin, I want a "Force Delete" option that cascades deletions (blueprint + affix references), so that I can clean up even when dependencies exist.
15. As an admin, I want to select multiple blueprints and batch-delete them, so that I can clean up in bulk.
16. As an admin, I want to select multiple blueprints and batch-assign affixes to their pools, so that I can configure affix availability efficiently.
17. As an admin, I want to batch-edit attributes across selected blueprints (only attributes with matching value types), so that I can update common properties like weight or damage ranges in one operation.
18. As an admin, I want to browse all affixes in a paginated table with search and type (prefix/suffix) filtering, so that I can find specific affixes.
19. As an admin, I want to create a new affix via an overlay modal with inline attribute form or "From global library" picker, so that I can define affixes flexibly.
20. As an admin, I want to view an affix's detail page (read-only, shareable URL) with its attribute details, name, type, description, and history tab, so that I can review an affix's full configuration.
21. As an admin, I want to edit an affix via a pre-filled modal from the list or detail page, so that I can update existing affixes.
22. As an admin, I want to delete affixes individually or in batch, with force-delete cascading (removes affix from all blueprint pools and adjusts min/max counts), so that I can manage the affix pool.
23. As an admin, I want to batch-assign selected affixes to multiple blueprints from the affix list, so that I can configure which blueprints can roll which affixes.
24. As an admin, I want to browse all global meta attributes in a paginated table with search and value-type filtering, including a usage count showing how many blueprints and affixes reference each one, so that I can understand the impact of changes.
25. As an admin, I want to create a global meta attribute via an overlay modal with a value-type selector (single/enum/range/string/boolean) and dynamic form fields based on the selected type, so that I can define reusable attribute definitions.
26. As an admin, I want to view a global meta attribute's detail page showing its definition, a "Referenced by" section listing blueprints and affixes that use it, and a history tab, so that I can review its usage.
27. As an admin, I want to edit or delete a global meta attribute, with force-delete cascading to remove all `$ref_id` references across blueprints and affixes, so that I can clean up unused or deprecated definitions.
28. As an admin, I want to manage clients from a list with create and delete operations, so that I can onboard and remove games/projects.
29. As an admin, I want to view a client's detail page showing its name, key management table, and a danger zone for deleting the client, so that I can manage tenant configuration.
30. As an admin, I want to create API keys per client with named permission sets (read, write, delete, generate, admin), so that game servers and tools get scoped access.
31. As an admin, I want to see the raw API key exactly once upon creation, so that I can copy it before it is hashed and stored.
32. As an admin, I want to revoke an API key immediately, so that compromised or unused keys are disabled.
33. As an admin, I want to browse the audit log in a full-page view with filters by client, resource type, action, and date range, so that I can track who changed what.
34. As an admin, I want to expand an audit log row to see a before/after JSON diff (side-by-side, color-coded), so that I can understand exactly what changed.
35. As an admin, I want to export client data as a ZIP download with folder-per-client structure, optionally including API keys, audit log, and inline global references, so that I can back up or migrate data.
36. As an admin, I want to import a ZIP archive with a guided wizard that handles client name collisions, reference resolution, and per-attribute conflict resolution, so that I can restore or transfer data safely.
37. As an admin, I want to switch between clients via a dropdown in the Navigation Drawer, so that I can manage multiple games without logging out.
38. As an admin, I want to toggle dark/light mode from the header or Navigation Drawer, so that I can work comfortably in any environment.
39. As an admin, I want a global search bar (Cmd+K) that searches blueprints and affixes by name with grouped results, so that I can navigate directly to any resource.
40. As an admin, I want to toggle an Activity Panel (right side) that shows the recent audit trail, so that I can monitor changes without navigating away.
41. As an admin, I want the Activity Panel to default as a floating overlay that hides on outside click, with an optional anchor/pin to dock it, so that I can choose my preferred interaction.
42. As an admin, I want the app to be fully responsive (mobile, tablet, desktop) with tables collapsing to stacked cards on narrow screens, so that I can manage configurations from any device.
43. As an admin, I want smooth page transitions, micro-interactions on hover, skeleton loaders during data fetches, and panel/drawer animations, so that the UI feels polished and responsive.
44. As an admin, I want toast notifications for success, error, and force-delete outcomes, so that I get clear feedback on my actions.
45. As an admin, I want delete-protection modals that show which resources reference the item being deleted (with links), so that I can navigate to those references before deciding to force-delete.

## Implementation Decisions

### Module Architecture

The UI is organized into the following modules:

- **API Client** (deep module) — A generated TypeScript client from the JSON Schema endpoints (`GET /api/schema/blueprints`, `GET /api/schema/affixes`, `GET /api/schema/generate`). Wraps all REST endpoints with typed request/response interfaces. Handles `X-API-Key` header injection, error normalization (RFC 9457 Problem JSON), and pagination cursor/offset parsing. Provides React hooks via TanStack Query wrappers.

- **Auth** (deep module) — Zustand store holding the raw API key in memory (never persisted). `useAuth()` hook exposes `login(key)`, `logout()`, `isAuthenticated`, and `isSuperAdmin` (derived from the key format or an initial `/api/clients` probe). On login success, redirects to dashboard. On 401/403 from any API call, auto-logs out.

- **App Shell / Layout** — Persistent shell wrapping all routes. Composed of:
  - Navigation Drawer (left, toggleable): client switcher dropdown, API key management link, login/logout, settings (dark mode toggle). Responsive: overlays on mobile, pushes layout on desktop.
  - Persistent Header: logo, global search bar (Cmd+K), dark/light toggle, drawer toggle buttons.
  - Activity Panel (right, toggleable): floating overlay by default, draggable/dockable.
  - Main content area with React Router `<Outlet />`.
  - Dark/light mode via Tailwind `dark:` class on `<html>`, toggled via Zustand.

- **Dashboard** — Stat cards (total blueprints, affixes, warnings count) fetched from summary endpoints. Quick Generate playground: archetype dropdown populated from distinct blueprint archetypes, dynamic constraint builder based on available meta attributes, inline result display.

- **Blueprints** — List page (table with search/filter/pagination, row actions, batch toolbar), Detail page (tabs: General, Attributes, Affixes, History), Create/Edit modal (tabbed: General, Attributes with drag reorder, Affixes with pool editor).

- **Affixes** — List, Detail, Create/Edit modal, following the same pattern as Blueprints.

- **Global Meta Attributes** — List, Detail (with "Referenced by" section), Create/Edit modal.

- **Clients / Keys** — List, Detail (key management table, danger zone), Create Key modal (permission checkboxes, one-time raw key display).

- **Audit Log** — Paginated table with filter bar (client, resource type, action, date range). Row expansion with side-by-side JSON diff using `react-diff-viewer-continued`.

- **Import / Export** — Export: client selector with per-client toggles, global inline-refs toggle, triggers browser download. Import: drag-and-drop ZIP picker, preview step, conflict resolution modal per resource/attribute.

- **Global Search** — Cmd+K command palette (powered by `cmdk`). Searches blueprints and affixes by name client-side (or via API search param). Results grouped by type with keyboard navigation.

- **Shared Components** — A set of reusable UI primitives:
  - `DeleteResourceDialog` — modal showing resource name, referencing resources with links, "Cancel" and "Force Delete" buttons.
  - `AttributeBadge` — small "inline" or "global" badge with optional link.
  - `AttributeRow` — display of a single attribute value with type-specific formatting.
  - `PaginatedTable` — reusable table wrapper with offset-based pagination, sortable columns, multi-select (shift-click range), batch toolbar slot.
  - `DiffView` — wrapper around `react-diff-viewer-continued` for before/after JSON display.
  - `ConfirmDialog` — generic confirmation modal used throughout.

### Data Flow

- All server-state fetches go through TanStack Query using the API Client module.
- Mutations (create, update, delete, batch ops) use TanStack Query mutations with automatic cache invalidation on success.
- When a mutation succeeds, the relevant query key is invalidated, triggering a refetch.
- Client state (auth, dark mode, current client selection, drawer/panel toggles) lives in Zustand stores.
- The Activity Panel polls or refetches on navigation events from the audit log endpoint, filtered to the current client.

### Routing (React Router, hash-based)

```
#/login
#/dashboard
#/blueprints
#/blueprints/:id
#/affixes
#/affixes/:id
#/global-meta-attributes
#/global-meta-attributes/:id
#/clients
#/clients/:id
#/audit-log
#/export
#/import
```

### Form Handling and Validation

- All forms use React Hook Form + Zod schemas.
- Zod schemas are shared between create and edit flows (edit schemas make all fields optional).
- Validation follows the API's rules: `attribute_order` completeness, `$ref_id` + inline field mutual exclusivity, value-type-specific payload requirements (enum must have `values`, range must have `min <= max`, distribution rates must be > 0, etc.).
- Drag-and-drop reorder for `attribute_order` uses `@dnd-kit/core`.

### API Client Generation

The API client is generated from the three JSON Schema endpoints. Generation runs as a build step (`npm run generate-api-client`). The generated client is checked into the repo for reproducible builds.

### Error Handling

- API errors (RFC 9457) are caught by TanStack Query's global error handler.
- On 401/403, the auth store logs out and redirects to `/login`.
- On 409 (delete-referenced-resource), the `DeleteResourceDialog` shows the conflict details and offers the "Force Delete" option.
- On 409 (import-conflict), the import flow transitions to the resolution phase.
- Toast notifications display error summaries from the Problem JSON `detail` field.
- Network errors (fetch failure) show a generic "Connection error" toast.

## Testing Decisions

### Testing Philosophy

Tests should verify external behavior only — what the user sees and does — never internal implementation details of a module. A good test describes a user action and asserts a visible outcome.

For the React admin UI, this means testing at the integration level: render a page, simulate user interactions (clicks, form fills, navigation), assert that the correct API calls were made (mock network), and verify the UI updates accordingly. Unit-test only modules that contain non-trivial logic independent of the DOM (e.g., validation schemas, auth utilities, pagination helpers).

### Modules to Test

| Module | Test Type | What to Test |
|--------|-----------|--------------|
| **Auth** (Zustand store) | Unit | Login/logout state transitions, key storage, `isAuthenticated` derivation |
| **API Client** | Integration (mock server) | Request formatting, error normalization, pagination parsing |
| **Shared Components** (DeleteResourceDialog, ConfirmDialog, etc.) | Component | Rendering with different props, callback invocation (vitest + testing-library) |
| **Blueprints List** | Integration | Render list, search, filter by archetype, pagination, multi-select, batch toolbar visibility |
| **Blueprint Create/Edit modal** | Integration | Form validation errors, add/remove attribute, drag reorder, add affix from picker, submit flow |
| **Blueprint Detail** | Integration | Tab switching, attribute badges, history tab with diff |
| **Affixes List** | Integration | Render, filter by type, batch assign flow |
| **Dashboard Quick Generate** | Integration | Archetype dropdown population, add constraint, submit, show result |
| **Login** | Integration | Submit key, redirect on success, show error on 401 |
| **Global Search** | Integration | Cmd+K opens palette, type shows results, enter navigates |
| **Import wizard** | Integration | Drag file, preview step, conflict resolution UI |

### Prior Art

No existing tests in this codebase. Test patterns follow standard practices for React + TanStack Query apps:

- Component/integration tests: `vitest` + `@testing-library/react` + `msw` for API mocking
- Store unit tests: `vitest` directly on Zustand stores (no DOM needed)
- Test structure: co-locate test files with the module (`blueprint-list.test.tsx` next to `blueprint-list.tsx`)
- Mock data: factory functions for blueprint, affix, global-meta-attribute objects (one shared `test/factories.ts`)
- Accessibility: prefer `getByRole` queries over test IDs (test the real UX)

## Out of Scope

- **Generation preview history** — saving and browsing recently generated things is deferred.
- **Real-time updates via WebSocket** — collaborative editing or live-change broadcasting across browser tabs is not included.
- **Multi-user auth / SSO / OAuth** — the API-key-only model is kept (see ADR 0001). User accounts are deferred.
- **The Rust API service itself** — this PRD covers only the admin UI. The API service is assumed to be already running or implemented separately.
- **The CLI tool** — the `arche` CLI is a separate deliverable.
- **Responsive design for the Quick Generate playground** — the playground is desktop-first in this iteration; mobile support can be refined later.
- **Internationalization (i18n)** — the UI is English-only for now.
- **Keyboard shortcuts beyond Cmd+K (search) and Escape (close modals)** — future iterations may add more.

## Further Notes

- The UI targets modern browsers (last 2 Chrome, Firefox, Safari, Edge releases). No IE11 support.
- Build tooling: Vite + TypeScript + Tailwind CSS v4. The app is deployed as a static SPA (hash-based routing for zero server config).
- The `cmdk` library powers the global search command palette — consistent with shadcn/ui's own preferences.
- Dark mode uses Tailwind's `class` strategy toggled via Zustand, persisted in localStorage across sessions.
- The API client generation step requires the Arche API to be running at build time (or a cached/mock schema). The build script fetches from `VITE_ARCHE_API_URL` with a fallback to local static schema files during development.
- Audit log timestamps are displayed in the browser's local timezone with a UTC tooltip.
- Import conflict resolution supports three strategies per resource: `keep_old`, `keep_new`, and `per_attribute` with per-key resolution.
