---
title: "Benchmark: Ramp-up engine"
status: completed
---

## Parent

PRD: `.scratch/benchmark-suite/prd.md`

## What to build

Replace the fixed-concurrency benchmark loop with a dynamic ramp-up engine:

1. Start with 1 concurrent task
2. Every 1.5 seconds, add 1 additional concurrent task
3. Maintain a rolling window of throughput measurements (requests completed per second, updated every 1s)
4. Track the peak throughput observed so far
5. Expose as a `--ramp-up` flag in the bench subcommand

The ramp-up function should be testable in isolation: accept a callback that simulates request completion timestamps, return the final concurrency level and peak throughput.

## Acceptance criteria

- [ ] `--ramp-up` flag replaces `--concurrency` in the bench subcommand
- [ ] Concurrency starts at 1 and grows by 1 every 1.5s
- [ ] A rolling 10s throughput window is maintained
- [ ] Peak throughput is tracked and updated as concurrency increases
- [ ] The ramp-up function is extractable and testable with synthetic timestamps

## Blocked by

- `bench-04-fixed-concurrency-bench` (replaces its loop logic)
