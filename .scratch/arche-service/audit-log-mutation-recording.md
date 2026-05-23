---
title: Audit log — mutation recording
status: ready-for-agent
---

## Parent

Arche API Service PRD (see `.scratch/arche-service/prd.md`)

## What to build

Implement audit log recording for all CRUD mutations. Every create, update, delete, force-delete, and cascade adjustment is recorded with before/after JSON snapshots.

Audit log entry fields:
- `id` (UUID), `timestamp`, `actor_key_id`, `actor_key_name` (denormalized), `client_id`, `resource_type` ("blueprint", "affix", "global_meta_attribute", "client", "api_key"), `resource_id`, `action` (created/updated/deleted/force_deleted/adjusted), `before` (JSONB, NULL for creates), `after` (JSONB, NULL for deletes)

Behavior:
- Written synchronously in the same DB transaction as the mutation (guarantees at-most-once logging)
- Generate operations are NOT logged
- Cascade adjustments (from force-delete) are logged with action `adjusted`
- The `before` snapshot is the resource state BEFORE the mutation; `after` is the state AFTER
- Actor information comes from the `AuthenticatedKey` extractor

## Acceptance criteria

- [ ] Create resource: audit entry with `action=created`, `before=NULL`, `after` = new resource
- [ ] Update resource: `action=updated`, `before` = old state, `after` = new state
- [ ] Delete resource: `action=deleted`, `before` = old state, `after=NULL`
- [ ] Force delete: `action=force_deleted`, with before/after snapshots
- [ ] Cascade adjustment: `action=adjusted`, before/after show the changed fields
- [ ] Generate: no audit entry created
- [ ] Actor key ID and name correctly recorded
- [ ] Integration test: perform mutation, query audit log, verify correct entry

## Blocked by

- arche-types/database-migrations.md
- arche-types/domain-structs-enums.md
- arche-service/auth-api-key-extractor.md
