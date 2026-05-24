---
title: Auth middleware — API key extractor
status: completed
---

## Parent

Arche API Service PRD (see `.scratch/arche-service/prd.md`)

## What to build

Implement a custom axum extractor `AuthenticatedKey` that reads the `X-API-Key` header, looks up the key by bcrypt hash from the `api_keys` table, and resolves:
- The key's `id` and `name`
- The `client_id` (if per-client key) or `None` (if super admin)
- The permission set
- Whether it's a super admin key
- Whether it has expired

On failure (missing header, unknown key, expired), return RFC 9457 Problem JSON with status 401.

The extractor is the primary authentication mechanism for all endpoints. It should be efficient (single DB query per request, or cached key lookups).

## Acceptance criteria

- [ ] `AuthenticatedKey` extractor works on any axum handler
- [ ] Valid super admin key authenticates successfully
- [ ] Valid per-client key authenticates with correct client_id and permissions
- [ ] Expired key returns 401
- [ ] Revoked key (deleted from DB) returns 401
- [ ] Missing `X-API-Key` header returns 401
- [ ] Malformed key returns 401
- [ ] All error responses are RFC 9457 Problem JSON
- [ ] Integration tests for all auth scenarios

## Blocked by

- arche-types/database-migrations.md
- arche-types/domain-structs-enums.md
- arche-service/super-admin-bootstrap.md
