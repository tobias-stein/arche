---
title: Cache — single-instance polling invalidation
status: completed
---

## Parent

Arche API Service PRD (see `.scratch/arche-service/prd.md`)

## What to build

Implement cache invalidation via periodic polling for single-instance mode (no Redis).

A background tokio task polls `SELECT max(updated_at) FROM ...` across all relevant tables every N seconds (configurable via `ARCHE_CACHE_POLL_INTERVAL_MS`, default 4000). If any `updated_at` value has changed since the last poll, reload the affected client's data from the database.

The poll task:
- Tracks the last-seen max `updated_at` per client
- On detecting a change, triggers a reload of that client's cache data
- Reload is done by re-fetching all blueprints, affixes, and global meta attributes for that client from the DB
- The reload happens asynchronously (non-blocking to the generation hot path)
- During reload, old data continues to serve requests (RwLock read path is not blocked)

Add a `Cache::reload_client(client_id)` method that re-fetches and replaces data for a single client.

## Acceptance criteria

- [ ] Background polling task starts on service boot
- [ ] Poll interval respects `ARCHE_CACHE_POLL_INTERVAL_MS` config
- [ ] Detects `updated_at` changes within poll interval
- [ ] Reloads only the changed client's data (not all clients)
- [ ] Old data serves requests during reload (no read lock contention)
- [ ] Integration test: change a blueprint via API, wait for poll interval, verify generate picks up changes
- [ ] Unit tests for poll logic with mock DB

## Blocked by

- arche-service/generation-engine-memory-load.md
