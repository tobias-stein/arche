---
title: "Benchmark: Plateau detection"
status: completed
---

## Parent

PRD: `.scratch/benchmark-suite/prd.md`

## What to build

Add plateau detection to the ramp-up engine. The benchmark run stops when one of two conditions is met:

1. **Plateau:** The trailing 15-second average throughput has grown by less than 3% compared to 15 seconds ago, despite active ramping
2. **Degradation:** Current throughput drops below 90% of the observed peak throughput

A minimum run duration of 20 seconds is enforced — no stop conditions are checked before 20s have elapsed.

The detection logic should be testable in isolation: provide a sequence of throughput measurements at timestamps, verify that the correct stop condition fires at the correct time.

## Acceptance criteria

- [ ] Plateau condition: <3% growth over 15s window triggers stop
- [ ] Degradation condition: throughput < 90% of peak triggers stop
- [ ] Both conditions are ignored for the first 20 seconds
- [ ] Detection functions are testable with synthetic data
- [ ] Pytest tests verify: plateau at correct boundary, degradation at correct boundary, no premature stop before 20s

## Blocked by

- `bench-08-ramp-up-engine` (needs the ramp-up engine)
