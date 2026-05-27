---
title: Add /api/me endpoint returning authenticated key info
status: ready-for-agent
---

## What to build

The frontend needs a way to determine what API key is authenticated, its scope (super admin vs. client-scoped), and which client it belongs to. Currently, the login flow calls `GET /api/clients` which requires super admin permission — client-scoped keys (even with `Admin` permission) cannot log in.

Add a new public endpoint `GET /api/me` that returns the authenticated key's information:

```json
{
  "id": "uuid",
  "name": "key-name",
  "clientId": "uuid-or-null",
  "permissions": ["read", "write", ...],
  "isSuper": false
}
```

- Route: `GET /api/me`
- Auth: Requires valid `X-API-Key` header (like all API routes)
- Permission: Passes through the permission middleware but requires no specific permission — any valid key can access it
- Returns the `AuthenticatedKey` struct serialized as JSON

Update the permission middleware to allow `/api/me` through without requiring a specific permission (similar to how schema endpoints are public, but still requiring a valid key).

## Acceptance criteria

- [ ] `GET /api/me` returns the authenticated key's info for any valid API key
- [ ] `GET /api/me` returns 401 for missing or invalid API key
- [ ] Super admin keys show `clientId: null`, `isSuper: true`
- [ ] Client-scoped keys show their `clientId` and `permissions`, `isSuper: false`
- [ ] Unit tests for the new endpoint

## Blocked by

None — can start immediately.
