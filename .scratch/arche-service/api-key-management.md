---
title: API key management
status: ready-for-agent
---

## Parent

Arche API Service PRD (see `.scratch/arche-service/prd.md`)

## What to build

Implement API key CRUD endpoints scoped to a client:

- `POST /api/clients/:id/keys` — Create a key. Request: `{ "name": "game-server-key", "permissions": ["read", "generate"] }`. Generate a random key (prefixed `arche_k_`), bcrypt-hash it for storage, return the raw key exactly once. Response includes id, name, permissions, and raw key string.
- `GET /api/clients/:id/keys` — List keys for a client. Returns key metadata only (id, name, permissions, created_at, expires_at) — never returns the raw key or hash.
- `DELETE /api/clients/:id/keys/:key_id` — Revoke a key. Soft delete or hard delete (immediate invalidation). No "undo" — the key stops working immediately.

Key creation requires `admin` permission. Key listing requires `read` or `admin`. Key revocation requires `admin`.

Permissions are validated: only valid permission strings (`read`, `write`, `delete`, `generate`, `admin`) are accepted.

## Acceptance criteria

- [ ] Create key returns raw key once, stores bcrypt hash
- [ ] List keys returns metadata only (no raw key)
- [ ] Revoke key — key stops authenticating immediately
- [ ] Revoke non-existent key returns 404
- [ ] Permission validation: invalid permission string returns 400
- [ ] Key auto-generated format starts with `arche_k_`
- [ ] Integration tests for all operations

## Blocked by

- arche-types/database-migrations.md
- arche-types/domain-structs-enums.md
- arche-service/auth-api-key-extractor.md
- arche-service/permission-matrix.md
