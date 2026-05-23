---
title: Audit log — query endpoint
status: ready-for-agent
---

## Parent

Arche API Service PRD (see `.scratch/arche-service/prd.md`)

## What to build

Implement `GET /api/audit-log` for querying audit log entries. Super admin only.

Query parameters:
- `cursor` — cursor-based pagination (by audit entry UUID)
- `limit` — page size (default 50, max 200)
- `client_id` — filter by client
- `resource_type` — filter by resource type string
- `action` — filter by audit action (created/updated/deleted/force_deleted/adjusted)
- `actor_key_id` — filter by actor
- Optionally: `from` / `to` timestamps for date range filtering

Response: `{ "data": [...], "next_cursor": "uuid" }`

Entries are ordered by `timestamp DESC` (most recent first). The cursor uses the entry's UUID for efficient offset pagination.

The endpoint is used by both the Admin UI's Audit Log page and the History tabs on detail pages.

## Acceptance criteria

- [ ] All filter parameters work independently and in combination
- [ ] Cursor pagination works correctly (no duplicates, no gaps)
- [ ] Default sort is timestamp DESC
- [ ] Super admin can see all entries; per-client key gets 403
- [ ] `from`/`to` timestamp filtering works
- [ ] Integration tests with populated audit log

## Blocked by

- arche-service/audit-log-mutation-recording.md
- arche-service/pagination-utilities.md
