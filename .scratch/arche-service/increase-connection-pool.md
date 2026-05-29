---
title: Increase DB connection pool size from 5 to ~50
status: ready-for-agent
---

## What to build

The PgPool is configured with `max_connections(5)` in `main.rs:140`. With only 5 connections shared across auth, cache polling, CRUD operations, and generate requests, the pool saturates immediately under concurrent load — requests queue behind each other even when the CPU is idle.

**Design:**

- Increase `max_connections` from 5 to a configurable value defaulting to 50.
- Add an `ARCHE_DB_POOL_SIZE` environment variable to `Config`.
- Consider the formula: `pool_size = num_cpus * 2 + 10` as a reasonable default, capped at the PostgreSQL `max_connections` limit.
- Ensure the pool sizing is documented in the config/env vars.

**Acceptance criteria:**

- [ ] Connection pool size is configurable via env var
- [ ] Default is 50 (or calculated from CPU count)
- [ ] Benchmark at conc=10 and conc=100 shows improved throughput (less queueing)
- [ ] Existing tests still pass

## Blocked by

None — can start immediately
