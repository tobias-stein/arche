---
title: Service scaffolding
status: completed
---

## Parent

Arche API Service PRD (see `.scratch/arche-service/prd.md`)

## What to build

Set up the `arche-service` crate as an axum HTTP server. Include environment-based configuration (port, database URL, Redis URL, cache poll interval), tokio async runtime, graceful shutdown on SIGTERM/SIGINT, and a health check endpoint (`GET /health` returning `{"status": "ok"}`).

Configuration is read from environment variables with sensible defaults: `ARCHE_PORT` (default 8080), `ARCHE_DATABASE_URL`, `ARCHE_REDIS_URL` (optional), `ARCHE_CACHE_POLL_INTERVAL_MS` (default 4000).

The `main.rs` initializes the axum router, binds to the configured address, and starts serving. A `Config` struct with `dotenvy` support loads config on startup.

## Acceptance criteria

- [ ] `cargo run` starts the server and binds to the configured port
- [ ] `GET /health` returns `200 {"status":"ok"}`
- [ ] Graceful shutdown: SIGTERM/SIGINT drains in-flight requests and exits
- [ ] Config loads from environment variables with defaults where sensible
- [ ] `cargo test` passes (health check integration test)
- [ ] Server logs startup info (port, mode, db URL host)

## Blocked by

- arche-types/workspace-scaffolding.md
- arche-types/domain-structs-enums.md
