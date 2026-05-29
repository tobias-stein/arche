---
title: Throughput Benchmark Suite for Arche Generate Endpoint
status: completed
---

## Problem Statement

The Arche service exposes a `POST /api/generate` endpoint that produces generated "things" from blueprints, affixes, and meta attributes. While the generation logic is well-tested at the unit level, there is no benchmark infrastructure to measure **throughput** (generates/s) under realistic data complexity or to find the service's saturation point. Without this, we cannot:

- Objectively compare throughput between code changes or config tweaks
- Find the optimal concurrency level for a given deployment
- Understand how data shape (pool size, attribute complexity, affix count) impacts performance

We need a benchmark suite that can be run with a single command, targets a local Docker Compose stack, and produces both a terminal summary and a visual HTML report.

## Solution

A Python-based benchmark suite under `bench/` that:

1. Reads a YAML config file defining the dimension sweep (blueprint pool size, affixes per blueprint, attributes per blueprint)
2. Starts the Arche stack via Docker Compose
3. Seeds synthetic test data across all dimension combinations (as separate API clients)
4. Runs an **adaptive ramp-up benchmark**: starts at 1 concurrent connection, adds +1 every 1.5s, measures generates/s in a rolling window, and stops when throughput plateaus or drops
5. Records peak throughput, optimal concurrency, and latency percentiles per scenario
6. Produces a terminal summary table and a self-contained HTML report (Chart.js via CDN) with interactive visualisations

## User Stories

1. As a developer, I want to run `./run-bench.sh` and get a complete benchmark result, so that I do not need to manually configure Docker or the benchmark tool.
2. As a developer, I want the benchmark to target an arbitrary URL via a flag, so that I can benchmark a remote/staging deployment.
3. As a developer, I want to configure the dimension sweep in a simple YAML file, so that I can add/remove levels without modifying code.
4. As a developer, I want to see a terminal summary table when the benchmark finishes, so that I can quickly assess results without opening a file.
5. As a developer, I want an HTML report with interactive charts, so that I can visually explore how throughput varies across dimensions and concurrency.
6. As a developer, I want the benchmark to seed synthetic test data with a mix of all attribute types (single, enum, range, string, boolean) and all distributions (uniform, normal, exponential), so that the workload represents a realistic average.
7. As a developer, I want the benchmark to determine the optimal concurrency level automatically via ramp-up, so that I do not need to guess concurrency levels or sweep them manually.
8. As a developer, I want the latency percentiles (p50, p95, p99) recorded at the peak throughput point, so that I can evaluate not just throughput but also quality of service.
9. As a developer, I want the raw results exported to CSV, so that I can further analyse or plot them with other tools.
10. As a developer, I want the benchmark to be reproducible (deterministic seeded data generation), so that I can compare results across runs.
11. As a developer, I want the stack to be torn down automatically after the benchmark, so that no dangling containers remain.

## Implementation Decisions

### Module structure

All code lives under `bench/`:

```
bench/
  run-bench.sh              # Entry point: manages Docker Compose lifecycle, invokes run.py
  scenarios.yaml            # Config: the dimension sweep matrix (user-editable)
  run.py                    # Main Python script — orchestrates, seeds, benchmarks, reports
  report.html               # Generated output — self-contained HTML report with Chart.js
  results.csv               # Generated output — raw data for further analysis
```

### No library/package — single script

`run.py` is a standalone Python script using only `aiohttp` and `pyyaml` as external dependencies (stdlib for everything else). No project-level `pyproject.toml` or Poetry — dependencies documented in a comment header and installable via `pip install aiohttp pyyaml`.

### Functions within run.py

The script is structured as a series of testable functions, not a library:

- **`load_config(path)`** — reads YAML, cross-products dimensions, returns list of scenario descriptors (each with blueprint_count, affix_count, attribute_count, deterministic seed)
- **`seed_client(base_url, api_key, scenario)`** — creates a client via POST /api/clients, then generates and POSTs blueprints, affixes, and blueprint-affix assignments. Returns client_id. Uses deterministic random (seeded by scenario hash) to pick attribute types and distributions.
- **`run_ramp_up(base_url, api_key, client_id, min_duration=20)`** — core benchmark engine. Manages a pool of concurrent aiohttp tasks. Adds +1 connection every 1.5s. Maintains a rolling 10s window of generates/s. Stops when throughput grows <3% over 15s or drops below 90% of observed peak. Returns peak_throughput, optimal_concurrency, latency_percentiles.
- **`LatencyTracker`** — helper class that records elapsed ms per request, computes p50/p95/p99 on demand using sorted list or a streaming algorithm (T-digest or P-squared for memory efficiency at high request rates).
- **`write_csv(path, rows)`** — writes raw results.
- **`write_html_report(path, data)`** — generates a self-contained HTML document with Chart.js loaded from CDN and the data embedded as JSON. Charts: throughput vs concurrency (per scenario), peak throughput faceted by dimension, latency summary.

### Fixture data generation

- Blueprint names are deterministic: `Blueprint-{n:04d}`
- Affix names: `Affix-{n:04d}-{prefix|suffix}`
- Each attribute is randomly assigned a value type (single, enum, range, string, boolean) and, where applicable, a distribution (uniform, normal, exponential)
- Range bounds, single values, enum options, etc. are all deterministic from a seeded RNG
- The seed for each scenario is derived from its dimension tuple: `hash(("arche-bench", bp_count, affix_count, attr_count))`

### Ramp-up algorithm details

```
concurrency = 1
peak_throughput = 0
window = sliding 10s buffer of (timestamp, request_count)

loop:
  spawn new concurrent task if due (every 1.5s)
  measure current throughput = requests completed in last 10s / 10
  update peak_throughput
  elapsed = time since start of run

  if elapsed < min_duration (20s):
    continue
  if throughput < 0.9 * peak_throughput:
    stop (saturated / degrading)
  if throughput < 1.03 * throughput_15s_ago:
    stop (plateaued)

  concurrency++
  wait for next tick (or event-driven when a task completes)
```

### Latency measurement

- Each request records its end-to-end duration
- At the end of the run (after ramp-up stops), the run continues for an additional **5 seconds at the optimal concurrency** to collect a stable latency sample at peak
- Compute p50, p95, p99 from that stable sample window

### Output

**Terminal summary table** (printed by run.py):

```
Scenario                          Concurrency  Gen/s   p50(ms)  p95(ms)  p99(ms)
bp=10  aff=0  attr=3             12           8534    1.2      2.8      4.1
bp=100 aff=2  attr=10            8            4210    2.1      5.3      8.7
...
```

**report.html**: Single HTML page with tabs per scenario grouping. Charts: line chart of throughput vs concurrency for each run, bar chart of peak throughput grouped by dimension, latency percentile grid.

## Testing Decisions

- **Good test**: tests the function in isolation without a running Docker stack. For example, ramp-up detection logic can be tested with synthetic event timestamps. Config parsing can be tested with a known YAML string.
- **What to test**:
  - `load_config` — correct cross-product generation, error handling for missing keys
  - **Ramp-up plateau detection** — feed synthetic throughput timestamps, verify stop decision at correct boundary (3% threshold, 90% drop threshold)
  - **LatencyTracker** — feed known durations, verify p50/p95/p99 match expected values
  - **Data generation determinism** — same seed produces same output
- **What NOT to test**:
  - End-to-end against a real Arche instance — that is the benchmark itself, not a unit test
  - Docker Compose orchestration — opaque to test, minor risk
  - HTML report rendering — visual output, validate by inspection
- **Prior art**: no existing benchmark tests in the codebase. These would be the first. A simple `pytest` setup inside `bench/tests/` is sufficient.

## Out of Scope

- Benchmarking CRUD endpoints (blueprints, affixes, etc.) — only `POST /api/generate`
- Benchmarking with constraints or non-empty request payloads — always `{}`
- Multi-instance / cluster mode — single Arche instance
- Redis pub/sub path — polling cache only (the default in single-instance mode)
- Long-running soak tests — each run is seconds, not hours
- Integration into CI — the benchmark is human-driven, not automated
- Python package management — `pip install` in the run-bench.sh is sufficient; no `pyproject.toml` or virtualenv
- Plotting library dependencies — the HTML report uses Chart.js from CDN, no matplotlib/seaborn

## Further Notes

- The benchmark should be **non-destructive** — it creates clients and data via the API, and teardown via Docker Compose removes everything
- If `--target-url` is provided, `run-bench.sh` skips Docker Compose lifecycle and seeds into the given instance
- The API super admin key is extracted from the Arche server logs at startup (it's printed to stdout on first boot); the script reads it from the container logs
- There is no need for Python venv management — the script is lightweight enough to run with system Python + pip
