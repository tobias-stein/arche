# ADR 0003: In-Memory Cache with Optional Redis Pub/Sub

**Date:** 2026-05-22
**Status:** Accepted

## Context

Arche targets both small games (100 req/s) and large scale (1M req/s). The /generate endpoint is read-heavy and CPU-bound (attribute rolling, weighted selection). Database access on every request would be a bottleneck.

## Decision

Each arche instance loads **all definitions** (blueprints, affixes, global meta attributes) into memory at startup. Two deployment modes:

1. **Single-instance:** No Redis. Cache invalidation via periodic polling of `updated_at` timestamps or direct PostgreSQL `NOTIFY`/`LISTEN`.

2. **Cluster mode:** On any CRUD write, the handling instance publishes an invalidation message to Redis pub/sub. All instances reload the affected client's data.

Redis is **never** used for hot-path data lookups — only for cross-instance coordination.

## Consequences

**Positive:**
- Zero database latency on /generate (all data in process memory)
- Same codebase scales from 100 req/s to 1M req/s with a config flag
- Redis stays lightweight — pub/sub only, no data storage, no memory pressure

**Negative:**
- Cache staleness window: in single mode, polling means stale data briefly
- Memory usage scales with total definition count, not active clients — a client with 10M blueprints consumes memory on all instances
- Startup time increases with data volume (must load everything before serving)
- Cache invalidation is all-or-nothing per client — a single affix change reloads all blueprints for that client

## Alternatives Considered

- **PostgreSQL on every request** — simplest but cannot reach 1M req/s
- **Redis as read cache** — adds a round-trip and Redis memory pressure. Shaves off DB time but adds network time. In-process memory is strictly faster.
- **Database read replicas** — distributes load but doesn't eliminate latency. Overkill for this use case.
- **No caching** — viable for 100 req/s with tuned PostgreSQL. The architecture supports this via the single-instance polling approach.
