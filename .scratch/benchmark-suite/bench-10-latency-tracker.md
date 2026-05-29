---
title: "Benchmark: Latency tracker + peak sample"
status: completed
---

## Parent

PRD: `.scratch/benchmark-suite/prd.md`

## What to build

Add a `LatencyTracker` to record per-request durations and compute percentiles:

1. Each completed request records its end-to-end duration in milliseconds
2. When the ramp-up detects plateau/degradation and stops, the run continues for an **additional 5 seconds at the optimal concurrency** to collect a stable latency sample
3. Compute p50, p95, p99 from the stable sample window
4. If the run is too short for a meaningful sample (< 50 requests in the stable window), print a warning and skip percentiles

The `LatencyTracker` should be testable in isolation: feed known durations, verify percentiles.

## Acceptance criteria

- [ ] Per-request timings are recorded in milliseconds
- [ ] After plateau detection, a 5s stable sample is collected at optimal concurrency
- [ ] p50, p95, p99 are computed from the stable sample
- [ ] Percentiles are included in the printed output and CSV
- [ ] A warning is printed if the stable sample has fewer than 50 requests
- [ ] Pytest tests verify percentile correctness with known inputs

## Blocked by

- `bench-09-plateau-detection` (needs the plateau detection to know when to sample)
