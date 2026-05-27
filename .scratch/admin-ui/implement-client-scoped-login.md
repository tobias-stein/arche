---
title: Implement client-scoped key login in frontend
status: completed
---

## What to build

The current login flow (`stores/auth.ts`) calls `GET /api/clients` to verify the API key, which requires super admin permission. Client-scoped keys (even with `Admin` permission) get a 403 and cannot log in. The store also unconditionally sets `isSuperAdmin: true`.

Refactor the login flow to use the new `GET /api/me` endpoint:

1. Call `/api/me` with the provided API key
2. If the response has `isSuper: true`:
   - Set `isAuthenticated: true`, `isSuperAdmin: true`
   - Set `selectedClientId: null` (all clients)
3. If the response has `isSuper: false` and a `clientId`:
   - Set `isAuthenticated: true`, `isSuperAdmin: false`
   - Set `selectedClientId` to the key's `clientId` (auto-select the client)
   - Fetch the client name from the UI state or from `/api/clients/{clientId}` to display it
4. If the response lacks both `isSuper` and `clientId`, reject with an error

Update the `AuthState` interface to store the authenticated key info (`keyId`, `keyName`, `clientId`, `permissions`).

Remove the `listClients()` call from the login flow.

## Acceptance criteria

- [ ] Login works for super admin keys (unchanged UX)
- [ ] Login works for client-scoped keys — verified via `/api/me`
- [ ] Client-scoped login auto-selects the key's client (sets `selectedClientId`)
- [ ] Auth store correctly tracks `isSuperAdmin` for both key types
- [ ] 401/403 responses from `/api/me` show "Invalid API key" error
- [ ] Existing tests updated and pass

## Blocked by

- [Add /api/me endpoint](../../arche-service/add-api-me-endpoint.md)
