---
title: Cache — cluster-mode Redis pub/sub invalidation
status: completed
---

## Parent

Arche API Service PRD (see `.scratch/arche-service/prd.md`)

## What to build

Implement cluster-mode cache invalidation via Redis pub/sub.

Add Redis support:
- On startup, connect to Redis (configurable via `ARCHE_REDIS_URL`).
- If Redis is unavailable, log a warning and fall back to polling mode.
- Subscribe to a Redis pub/sub channel (e.g., `arche:cache-invalidate`).

On any CRUD write that affects generation data (blueprints, affixes, global meta attributes, blueprint_affixes), the handling instance publishes an invalidation message:
```json
{ "type": "invalidate", "client_id": "uuid" }
```

All instances (including the publisher) receive the message and reload the affected client's data.

Redis is NEVER used for hot-path data lookups — only for cross-instance coordination. The existing in-memory cache serves all generation requests.

## Acceptance criteria

- [ ] Redis connection established at startup when `ARCHE_REDIS_URL` is set
- [ ] Redis unavailable → fallback to polling, warning logged
- [ ] CRUD write publishes invalidation event to Redis pub/sub
- [ ] All instances receive event and reload affected client
- [ ] Hot-path generation does NOT touch Redis
- [ ] Invalidation event format: `{ "type": "invalidate", "client_id": "uuid" }`
- [ ] Integration test: start two instances sharing Redis, write to one, verify the other picks up changes

## Blocked by

- arche-service/generation-engine-memory-load.md
- arche-service/cache-single-mode-polling.md
