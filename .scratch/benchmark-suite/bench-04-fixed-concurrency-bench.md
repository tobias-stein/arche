---
title: "Benchmark: Fixed-concurrency benchmark loop"
status: completed
---

## Parent

PRD: `.scratch/benchmark-suite/prd.md`

## What to build

Extend `bench/run.py` with a `bench` subcommand (e.g. `--bench`) that:

1. Accepts `--concurrency` (default 1) and `--duration` (default 10) arguments
2. Uses the seeded client's API key and client_id
3. Fires `POST /api/generate {}` requests at the given concurrency level for the given duration
4. Tracks total requests completed, elapsed wall time
5. Prints: total requests, generates/s, and a simple summary to stdout
6. Cleans up the test client and API key after the run

## Acceptance criteria

- [ ] `python bench/run.py --bench --concurrency 1 --duration 10` runs and prints generates/s
- [ ] `--concurrency 5` spawns 5 concurrent tasks
- [ ] Generates/s is computed correctly (completed / wall time)
- [ ] Test client + key are deleted after the run (or the script supports `--no-cleanup`)

## Blocked by

- `bench-03-data-seeding` (needs a seeded client)
