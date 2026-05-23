---
title: Permission matrix & endpoint guarding
status: ready-for-agent
---

## Parent

Arche API Service PRD (see `.scratch/arche-service/prd.md`)

## What to build

Define a permission matrix that maps each API endpoint to the required permission level. Implement axum middleware (or a wrapper around `AuthenticatedKey`) that enforces the check.

Permission map:
| Endpoint | Required Permission |
|---|---|
| `GET /api/blueprints` (etc. — read endpoints) | `read` |
| `POST /api/blueprints` (etc. — create endpoints) | `write` |
| `PUT /api/blueprints/:id` (etc. — update endpoints) | `write` |
| `DELETE /api/blueprints/:id` (etc.) | `delete` |
| `POST /api/generate` | `generate` |
| `POST /api/clients/:id/keys` | `admin` |
| `GET /api/audit-log` | super admin only |

Super admin keys bypass all permission checks and can access any endpoint across all clients. Per-client keys are scoped to their client's data only — a key with `client_id` cannot access resources belonging to another client.

Return 403 Problem JSON when a key lacks the required permission.

## Acceptance criteria

- [ ] Super admin key can access all endpoints for any client
- [ ] Per-client key with `read` permission can GET but not POST/PUT/DELETE resources
- [ ] Per-client key with `write` can create and update but not delete
- [ ] Per-client key cannot access another client's resources (403)
- [ ] Non-admin key cannot access audit log (403)
- [ ] Integration tests for each permission level and each endpoint category

## Blocked by

- arche-service/auth-api-key-extractor.md
