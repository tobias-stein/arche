---
title: Auth — login page + Zustand store
status: ready-for-agent
---

## Parent

Arche Admin UI PRD (see `.scratch/admin-ui/prd.md`)

## What to build

Implement the login flow:

1. **Login page** (`/login` route): single input field for the super admin API key. Submit button. On success, redirect to `/dashboard`. On 401, show error message. Clean, centered layout.

2. **Auth Zustand store**: holds the raw API key in memory only (never persisted to localStorage/IndexedDB). Exposes:
   - `apiKey: string | null`
   - `isAuthenticated: boolean` (derived from `apiKey !== null`)
   - `isSuperAdmin: boolean` (derived by probing `/api/clients` on login)
   - `login(key: string): Promise<void>` — sets the API key, probes auth state
   - `logout(): void` — clears the key, redirects to `/login`

3. **Auth guard**: a React Router layout wrapper that redirects unauthenticated users to `/login`. Protects all routes except login.

4. **Auto-logout**: TanStack Query's global error handler: on 401/403 from any API call, call `logout()` and redirect to `/login`.

## Acceptance criteria

- [x] Login form renders, accepts API key, calls login store method
- [x] Valid key: redirects to dashboard
- [x] Invalid key: shows error message, stays on login page
- [x] Key is stored in memory only (check: not in localStorage)
- [x] After login, protected routes render (not redirected)
- [x] After logout, protected routes redirect to login
- [x] 401/403 from any API call triggers auto-logout
- [x] Unit tests for auth store state transitions
- [x] Integration test: login flow with mocked API

## Blocked by

- admin-ui/api-client-generation.md
- admin-ui/app-shell-layout.md
