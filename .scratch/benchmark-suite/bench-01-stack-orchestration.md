---
title: "Benchmark: Stack orchestration"
status: ready-for-agent
---

## Parent

PRD: `.scratch/benchmark-suite/prd.md`

## What to build

A `bench/run-bench.sh` shell script that:

1. Checks that Docker and Docker Compose are available
2. Runs `docker compose up -d` to start Postgres + arche-service
3. Polls `curl -s -o /dev/null -w "%{http_code}" http://localhost:8080/api/blueprints` until it returns a non-5xx status (with a timeout)
4. Extracts the super admin API key from the arche-service container logs (it's printed to stdout on first startup)
5. Prints the API key to stdout for downstream use
6. On Ctrl+C or script exit (trap), runs `docker compose down`

## Acceptance criteria

- [ ] `./bench/run-bench.sh` starts the stack, waits for health, prints the API key, and stays running (or hands off to a child script)
- [ ] Ctrl+C triggers cleanup (`docker compose down`)
- [ ] Supports `--skip-setup` flag to skip Docker Compose lifecycle (for reuse by other scripts)
- [ ] Supports `--target-url` flag to skip Docker Compose entirely and just print a placeholder API key

## Blocked by

None — can start immediately.
