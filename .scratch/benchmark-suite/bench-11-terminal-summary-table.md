---
title: "Benchmark: Terminal summary table"
status: ready-for-agent
---

## Parent

PRD: `.scratch/benchmark-suite/prd.md`

## What to build

Add a terminal summary table printed at the end of the sweep. The table is rendered to stdout with aligned columns:

```
Scenario                         Peak gen/s  Concurrency  p50(ms)  p95(ms)  p99(ms)
bp=10    aff=0  attr=3           8534        12           1.2      2.8      4.1
bp=10    aff=2  attr=3           7210        10           1.4      3.2      5.0
...
bp=1000  aff=4  attr=25          2100        6            4.1      11.2     18.3
```

Also include a footer row showing the overall peak throughput across all scenarios and the scenario that achieved it.

## Acceptance criteria

- [ ] Table is printed to stdout at the end of `--sweep`
- [ ] Columns are aligned and readable
- [ ] Footer row shows overall peak and best scenario
- [ ] Table handles missing latency data gracefully (shows "—" or "N/A")
- [ ] Column widths auto-adjust to content

## Blocked by

- `bench-06-multi-scenario-sweep` (needs the sweep data)
- `bench-10-latency-tracker` (needs latency data columns)
