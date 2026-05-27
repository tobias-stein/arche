---
title: Fix generation from client-specific domain
status: completed
---

## What to build

The `POST /api/generate` endpoint resolves the target client exclusively from the API key's `client_id` field. Super admin keys have `client_id: null`, so they always get a 403 "API key is not scoped to a client". This prevents the Quick Generate playground on the Dashboard from working for super admin users.

Two changes:

**Backend:** Add an optional `clientId` field to `GenerateRequest` in `crates/arche-types/src/generate.rs`. Update the `generate_handler` to:
- If the API key has a `client_id` (client-scoped key), use it (existing behavior)
- If the API key is a super admin key (`client_id: None`), require `clientId` in the request body — return 400 if missing
- The `clientId` from the request body passes through `require_client_access` for non-super keys (normal validation)

**Frontend:** In the Dashboard Quick Generate playground (`pages/Dashboard.tsx`), pass the current `selectedClientId` from the UI store as `clientId` in the `GenerateRequest`. When `selectedClientId` is null (all clients), disable the generate button or show a message to select a client.

## Acceptance criteria

- [ ] `GenerateRequest` has an optional `clientId` field
- [ ] Super admin keys can generate by providing `clientId` in the request body
- [ ] Client-scoped keys continue to work as before (their own client_id is used)
- [ ] Invalid/missing `clientId` for super admin returns a clear 400 error
- [ ] Dashboard Quick Generate passes the selected client ID in the request
- [ ] Generate button is disabled when no client is selected (for super admin users)

## Blocked by

None — can start immediately. Uses the existing `selectedClientId` from the UI store; does not depend on the `/api/me` endpoint.
