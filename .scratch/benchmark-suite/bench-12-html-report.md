---
title: "Benchmark: HTML report with Chart.js"
status: completed
---

## Parent

PRD: `.scratch/benchmark-suite/prd.md`

## What to build

Generate `bench/report.html` — a self-contained HTML report with:

1. Chart.js loaded from CDN (`https://cdn.jsdelivr.net/npm/chart.js`)
2. The sweep data embedded as a JSON array in a `<script>` tag
3. **Charts:**
   - **Throughput vs Concurrency** — one line series per scenario, x=concurrency, y=gen/s. Grouped into sub-charts per blueprint pool size for readability.
   - **Peak Throughput by Scenario** — bar chart, one bar per scenario, sorted descending, coloured by attribute complexity.
   - **Latency Percentiles** — grouped bar chart showing p50/p95/p99 for each scenario.
4. A **scenario filter / dimension picker** (simple checkboxes or dropdowns) to select which dimension levels to show
5. The report file is standalone — open it in a browser, no build step needed

## Acceptance criteria

- [ ] `bench/report.html` is generated after `--sweep` completes
- [ ] Charts render correctly when opened in a browser
- [ ] Data is embedded as JSON (not fetched from an external source)
- [ ] Scenario filter controls work
- [ ] Axes are labelled with units

## Blocked by

- `bench-06-multi-scenario-sweep` (needs the sweep data)
- `bench-10-latency-tracker` (needs latency data for the percentiles chart)
