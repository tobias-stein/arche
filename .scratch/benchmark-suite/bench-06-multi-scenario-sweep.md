---
title: "Benchmark: Multi-scenario sweep"
status: completed
---

## Parent

PRD: `.scratch/benchmark-suite/prd.md`

## What to build

Extend `bench/run.py` with a `sweep` subcommand (e.g. `--sweep`) that:

1. Reads the YAML config and computes all 27 scenario descriptors
2. For each scenario:
   - Creates a named client (e.g. `bench-bp10-aff0-attr3`)
   - Seeds the data matching that scenario
   - Creates a scoped API key
   - Runs the fixed-concurrency benchmark loop (`--bench` logic)
   - Records results
3. Produces `bench/results.csv` with columns: `blueprint_count, affix_count, attribute_count, concurrency, duration_s, total_requests, throughput`
4. Prints progress to stdout during the sweep (e.g. `[1/27] bp=10 aff=0 attr=3 ... 1423 gen/s`)
5. Cleans up all test clients after the sweep completes

## Acceptance criteria

- [ ] `python bench/run.py --sweep` runs all 27 scenarios sequentially
- [ ] Each scenario uses a separate client with appropriate data
- [ ] `bench/results.csv` is produced with one row per scenario
- [ ] Progress is printed to stdout
- [ ] All test clients are cleaned up at the end

## Blocked by

- `bench-04-fixed-concurrency-bench` (reuses the bench loop)
- `bench-05-yaml-config` (reuses the config loader)
