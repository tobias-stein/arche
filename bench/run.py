"""
Benchmark suite for Arche generate endpoint.

Supports:
  - API connectivity check: send a single generate request
  - Config loading: read YAML dimension sweep config
  - Data seeding: create client, API key, and blueprints with synthetic data
  - Fixed-concurrency benchmark: fire generate requests at given concurrency
  - Ramp-up benchmark: dynamic concurrency increasing by 1 every 1.5s

Usage:
    python bench/run.py --api-key <key>
    python bench/run.py --api-key <key> --target-url http://other-host:8080
    python bench/run.py --api-key <key> --seed
    python bench/run.py --api-key <key> --bench --concurrency 5 --duration 30
    python bench/run.py --api-key <key> --bench --ramp-up --duration 30
    python bench/run.py --config bench/scenarios.yaml --print-scenarios
"""

import argparse
import csv
import itertools
import json
import os
import re
import sys
import threading
import time
import urllib.request
from urllib.error import HTTPError, URLError

from latency_tracker import LatencyTracker
from ramp_up import run_ramp_up
from synthetic import generate_blueprint_body, generate_global_meta_attributes

SAMPLE_DURATION = 5


def send_generate_request(api_key, target_url="http://localhost:8080"):
    """Send a single POST /api/generate with empty body.

    Returns (status_code, body_string, elapsed_ms).
    Raises HTTPError or URLError on failure.
    """
    url = f"{target_url.rstrip('/')}/api/generate"
    body = b"{}"
    headers = {
        "Content-Type": "application/json",
        "X-API-Key": api_key,
    }
    req = urllib.request.Request(url, data=body, headers=headers, method="POST")

    start = time.perf_counter()
    with urllib.request.urlopen(req) as resp:
        elapsed_ms = (time.perf_counter() - start) * 1000
        status = resp.status
        body_str = resp.read().decode("utf-8")
    return status, body_str, elapsed_ms


def _request_json(method, path, body, api_key, target_url="http://localhost:8080"):
    """Send a JSON request and return the parsed response dict.

    Raises HTTPError or URLError on failure.
    """
    url = f"{target_url.rstrip('/')}{path}"
    data = json.dumps(body).encode("utf-8") if body is not None else None
    headers = {
        "Content-Type": "application/json",
        "X-API-Key": api_key,
    }
    req = urllib.request.Request(url, data=data, headers=headers, method=method)
    with urllib.request.urlopen(req) as resp:
        return json.loads(resp.read().decode("utf-8"))


def _handle_request_error(e, target_url):
    """Handle HTTPError or URLError: print details and exit with code 1."""
    if isinstance(e, HTTPError):
        print(f"Error: HTTP {e.code} {e.reason}", file=sys.stderr)
        try:
            error_body = e.read().decode("utf-8", errors="replace")
            print(f"Response body: {error_body}", file=sys.stderr)
        except Exception:
            pass
    else:
        print(f"Error: Could not reach server at {target_url}", file=sys.stderr)
        print(f"Reason: {e.reason}", file=sys.stderr)
    sys.exit(1)


def create_client(api_key, target_url="http://localhost:8080", name="bench-scratch"):
    """Create a client with the given name and return its id."""
    body = {"name": name}
    resp = _request_json("POST", "/api/clients", body, api_key, target_url)
    return resp["id"]


def create_api_key(client_id, api_key, target_url="http://localhost:8080"):
    """Create a client-scoped API key with generate permission.

    Returns (key_id, key_string).
    """
    body = {"name": "benchmark-key", "permissions": ["generate"]}
    resp = _request_json(
        "POST", f"/api/clients/{client_id}/keys", body, api_key, target_url
    )
    return resp["id"], resp["key"]


def _build_blueprint_body(name, attribute_count=3, scenario=None, blueprint_idx=0):
    """Build the request body for creating a blueprint with synthetic attributes.

    When scenario is None, uses a default scenario for backward compatibility.
    """
    if scenario is None:
        scenario = {"blueprint_count": 10, "affix_count": 0, "attribute_count": attribute_count}
    return generate_blueprint_body(name, attribute_count, scenario, blueprint_idx)


def create_blueprint(client_id, api_key, name, target_url="http://localhost:8080",
                      attribute_count=3, scenario=None, blueprint_idx=0):
    """Create a blueprint with synthetic attributes and return its id."""
    body = _build_blueprint_body(name, attribute_count, scenario, blueprint_idx)
    path = f"/api/blueprints?client_id={client_id}"
    resp = _request_json("POST", path, body, api_key, target_url)
    return resp["id"]


_KEY_MAP = {
    "blueprints": "blueprint_count",
    "affixes": "affix_count",
    "attributes": "attribute_count",
}


def _parse_yaml_dimensions(path):
    """Parse a minimal YAML dimensions file without pyyaml."""
    try:
        with open(path, "r") as f:
            lines = f.readlines()
    except FileNotFoundError:
        raise ValueError(f"Config file not found: {path}")
    except OSError as e:
        raise ValueError(f"Error reading config file {path}: {e}")

    dimensions = {}
    in_dimensions = False

    for i, line in enumerate(lines, 1):
        stripped = line.strip()

        if not stripped or stripped.startswith("#"):
            continue

        if stripped == "dimensions:":
            in_dimensions = True
            continue

        if in_dimensions:
            m = re.match(r"^\s{2}(\w+):\s*\[([^\]]*)\]\s*(?:#.*)?$", line)
            if not m:
                raise ValueError(
                    f"Line {i}: expected indented dimension entry "
                    f"in format '  key: [val1, val2, ...]', got: {stripped!r}"
                )

            key = m.group(1)
            values_str = m.group(2).strip()

            if not values_str:
                raise ValueError(f"Line {i}: dimension '{key}' has empty list")

            values = []
            for v in values_str.split(","):
                v = v.strip()
                if not v:
                    continue
                try:
                    values.append(int(v))
                except ValueError:
                    try:
                        values.append(float(v))
                    except ValueError:
                        values.append(v)

            dimensions[key] = values

    if not in_dimensions:
        raise ValueError("Missing 'dimensions:' key in YAML file")

    if not dimensions:
        raise ValueError("No dimension entries found under 'dimensions:'")

    return dimensions


def _cross_product(dimensions):
    """Compute cross-product of all dimension lists."""
    keys = list(dimensions.keys())
    value_lists = [dimensions[k] for k in keys]

    scenarios = []
    for combo in itertools.product(*value_lists):
        scenario = {}
        for k, v in zip(keys, combo):
            name = _KEY_MAP.get(k, k.rstrip("s") + "_count")
            scenario[name] = v
        scenarios.append(scenario)

    return scenarios


def load_config(path):
    """Load YAML config and return list of scenario descriptors.

    Each scenario descriptor is a dict like:
        {"blueprint_count": 10, "affix_count": 0, "attribute_count": 3}
    """
    dimensions = _parse_yaml_dimensions(path)
    scenarios = _cross_product(dimensions)
    return scenarios


def _format_scenario(s):
    return ", ".join(f"{k}={v}" for k, v in s.items())


def _parse_args(argv=None):
    parser = argparse.ArgumentParser(
        description="Benchmark suite for Arche generate endpoint"
    )
    parser.add_argument(
        "--api-key",
        help="API key for authentication",
    )
    parser.add_argument(
        "--target-url",
        default="http://localhost:8080",
        help="Target Arche server URL (default: http://localhost:8080)",
    )
    parser.add_argument(
        "--config",
        default="bench/scenarios.yaml",
        help="Path to YAML config file (default: bench/scenarios.yaml)",
    )
    parser.add_argument(
        "--print-scenarios",
        action="store_true",
        help="Print all scenario combinations and exit",
    )
    parser.add_argument(
        "--seed",
        action="store_true",
        help="Seed test data (client, API key, blueprints)",
    )
    parser.add_argument(
        "--bench",
        action="store_true",
        help="Run fixed-concurrency or ramp-up benchmark",
    )
    parser.add_argument(
        "--concurrency",
        type=int,
        default=1,
        help="Number of concurrent generate requests (default: 1)",
    )
    parser.add_argument(
        "--ramp-up",
        action="store_true",
        help="Enable ramp-up mode (replaces --concurrency)",
    )
    parser.add_argument(
        "--duration",
        type=int,
        default=10,
        help="Benchmark duration in seconds (default: 10)",
    )
    parser.add_argument(
        "--no-cleanup",
        action="store_true",
        help="Skip cleanup of test client and API key after benchmark",
    )
    parser.add_argument(
        "--sweep",
        action="store_true",
        help="Run multi-scenario sweep from YAML config",
    )
    return parser.parse_args(argv)


def main():
    args = _parse_args()

    if args.api_key:
        if args.sweep:
            _run_sweep(
                args.api_key, args.target_url, args.config,
                concurrency=args.concurrency,
                duration=args.duration,
            )
        elif args.bench:
            _run_bench(
                args.api_key, args.target_url,
                concurrency=args.concurrency,
                duration=args.duration,
                no_cleanup=args.no_cleanup,
                ramp_up=args.ramp_up,
            )
        elif args.seed:
            _run_seed(args.api_key, args.target_url)
        else:
            _run_api_connectivity(args.api_key, args.target_url)
        return

    try:
        scenarios = load_config(args.config)
    except ValueError as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)

    if args.print_scenarios:
        for s in scenarios:
            print(f"  {_format_scenario(s)}")
        print(f"\nTotal: {len(scenarios)} scenario(s)")


def _run_api_connectivity(api_key, target_url):
    """Send a single generate request and print the response."""
    try:
        status, body, elapsed_ms = send_generate_request(api_key, target_url)
    except HTTPError as e:
        _handle_request_error(e, target_url)
    except URLError as e:
        _handle_request_error(e, target_url)

    print(f"Status: {status}")
    print(f"Body: {body}")
    print(f"Elapsed: {elapsed_ms:.2f} ms")


def _seed_bench_data(api_key, target_url):
    """Seed test data and return (client_id, key_id, scoped_key)."""
    client_id = create_client(api_key, target_url)
    key_id, scoped_key = create_api_key(client_id, api_key, target_url)
    for i in range(10):
        create_blueprint(client_id, scoped_key, f"Blueprint-{i:04d}", target_url)
    return client_id, key_id, scoped_key


def _run_seed(api_key, target_url, scenario=None):
    """Seed test data: create client, API key, blueprints, and global meta attributes.

    When scenario is None, uses a default scenario (10 blueprints, 3 attributes each).
    """
    if scenario is None:
        scenario = {"blueprint_count": 10, "affix_count": 0, "attribute_count": 3}
    try:
        client_id = create_client(api_key, target_url)
        scoped_key = create_api_key(client_id, api_key, target_url)

        global_attrs = generate_global_meta_attributes(scenario)
        print(f"Global meta attributes: {len(global_attrs)} generated")

        for i in range(scenario["blueprint_count"]):
            name = f"Blueprint-{i:04d}"
            create_blueprint(
                client_id, scoped_key, name, target_url,
                attribute_count=scenario["attribute_count"],
                scenario=scenario,
                blueprint_idx=i,
            )

        print(f"Client ID: {client_id}")
        print(f"API Key: {scoped_key}")
    except HTTPError as e:
        _handle_request_error(e, target_url)
    except URLError as e:
        _handle_request_error(e, target_url)


def _collect_stable_sample(api_key, target_url, concurrency, sample_duration=None):
    """Run generate requests at fixed concurrency and record latencies.

    Returns a LatencyTracker with per-request durations in milliseconds.
    """
    if sample_duration is None:
        sample_duration = SAMPLE_DURATION
    tracker = LatencyTracker()
    stop_event = threading.Event()

    def _worker():
        while not stop_event.is_set():
            try:
                _, _, elapsed_ms = send_generate_request(api_key, target_url)
                tracker.record(elapsed_ms)
            except (HTTPError, URLError):
                pass

    threads = [
        threading.Thread(target=_worker, daemon=True)
        for _ in range(concurrency)
    ]
    for t in threads:
        t.start()

    stop_event.wait(timeout=sample_duration)
    stop_event.set()
    for t in threads:
        t.join(timeout=2.0)

    return tracker


def _run_bench_loop(api_key, target_url, concurrency=1, duration=10):
    """Run the generate-request loop at fixed concurrency.

    Returns (total_requests, elapsed_s, throughput).
    """
    completed = [0]
    lock = threading.Lock()
    stop_event = threading.Event()

    def _worker():
        while not stop_event.is_set():
            try:
                send_generate_request(api_key, target_url)
                with lock:
                    completed[0] += 1
            except (HTTPError, URLError):
                pass

    threads = []
    for _ in range(concurrency):
        t = threading.Thread(target=_worker, daemon=True)
        t.start()
        threads.append(t)

    start = time.monotonic()
    stop_event.wait(timeout=duration)
    elapsed = time.monotonic() - start

    stop_event.set()
    for t in threads:
        t.join(timeout=2.0)

    total = completed[0]
    rate = total / elapsed if elapsed > 0 else 0.0
    return total, elapsed, rate


def _run_bench(api_key, target_url, concurrency=1, duration=10,
               no_cleanup=False, ramp_up=False):
    """Run a fixed-concurrency or ramp-up benchmark.

    Seeds test data, runs generate requests (fixed concurrency or dynamic
    ramp-up), prints a summary, and cleans up.
    """
    try:
        client_id, key_id, scoped_key = _seed_bench_data(api_key, target_url)
    except HTTPError as e:
        _handle_request_error(e, target_url)
    except URLError as e:
        _handle_request_error(e, target_url)

    completed = [0]
    lock = threading.Lock()
    stop_event = threading.Event()

    def _worker():
        while not stop_event.is_set():
            try:
                send_generate_request(scoped_key, target_url)
                with lock:
                    completed[0] += 1
            except (HTTPError, URLError):
                pass

    if ramp_up:
        ramp_interval = 1.5
        start = time.monotonic()

        def _ramp_worker(c):
            ts = []
            inner_lock = threading.Lock()
            inner_stop = threading.Event()

            def _inner_worker():
                while not inner_stop.is_set():
                    try:
                        send_generate_request(scoped_key, target_url)
                        with lock:
                            completed[0] += 1
                        with inner_lock:
                            ts.append(time.monotonic() - start)
                    except (HTTPError, URLError):
                        pass

            threads = [
                threading.Thread(target=_inner_worker, daemon=True)
                for _ in range(c)
            ]
            for t in threads:
                t.start()
            inner_stop.wait(timeout=ramp_interval)
            inner_stop.set()
            for t in threads:
                t.join(timeout=2.0)

            return ts

        final_conc, peak, stop_reason = run_ramp_up(
            _ramp_worker, duration=duration, ramp_interval=ramp_interval,
        )
        elapsed = time.monotonic() - start
        total = completed[0]

        print(f"total requests: {total}")
        print(f"elapsed: {elapsed:.2f} s")
        print(f"final concurrency: {final_conc}")
        print(f"peak throughput: {peak:.2f} req/s")

        stable_tracker = _collect_stable_sample(
            scoped_key, target_url, concurrency=final_conc,
        )
        if stable_tracker.count < 50:
            print(f"warning: stable sample too small ({stable_tracker.count} requests), skipping percentiles")
        else:
            print(f"stable sample: {stable_tracker.count} requests")
            print(f"p50: {stable_tracker.p50():.2f} ms")
            print(f"p95: {stable_tracker.p95():.2f} ms")
            print(f"p99: {stable_tracker.p99():.2f} ms")
    else:
        total, elapsed, rate = _run_bench_loop(
            scoped_key, target_url, concurrency, duration,
        )
        print(f"total requests: {total}")
        print(f"elapsed: {elapsed:.2f} s")
        print(f"generates/s: {rate:.2f}")

    if no_cleanup:
        return

    try:
        _request_json(
            "DELETE", f"/api/clients/{client_id}/keys/{key_id}",
            None, api_key, target_url,
        )
        _request_json(
            "DELETE", f"/api/clients/{client_id}",
            None, api_key, target_url,
        )
    except HTTPError as e:
        _handle_request_error(e, target_url)
    except URLError as e:
        _handle_request_error(e, target_url)


def _generate_html_report(results, html_path):
    """Generate a self-contained HTML report with Chart.js charts.

    Charts:
      - Throughput vs Concurrency: line series per scenario, grouped by
        blueprint pool size into sub-charts.
      - Peak Throughput by Scenario: bar chart sorted descending, coloured
        by attribute complexity.
      - Latency Percentiles: grouped bar chart with p50/p95/p99 per scenario.
    """
    results_json = json.dumps(results)

    bp_levels = sorted(set(r["blueprint_count"] for r in results))

    subchart_tabs = ""
    subchart_canvases = ""
    for idx, bp in enumerate(bp_levels):
        active = "active" if idx == 0 else ""
        show = "show active" if idx == 0 else ""
        tab_id = f"bp-tab-{bp}"
        canvas_id = f"bp-chart-{bp}"
        subchart_tabs += (
            f'<li class="nav-item">'
            f'<button class="nav-link {active}" id="{tab_id}" '
            f'data-bs-toggle="tab" data-bs-target="#{canvas_id}" '
            f'type="button" role="tab" '
            f'onclick="switchBlueprint()">{bp} blueprints</button>'
            f'</li>\n'
        )
        subchart_canvases += (
            f'<div class="tab-pane fade {show}" id="{canvas_id}" '
            f'role="tabpanel">'
            f'<canvas id="throughputChart-{bp}"></canvas>'
            f'</div>\n'
        )

    html_content = f"""<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Benchmark Report</title>
<script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
<link href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.0/dist/css/bootstrap.min.css" rel="stylesheet">
<style>
    body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 20px; background: #f8f9fa; }}
    h1 {{ color: #333; }}
    .chart-container {{ background: #fff; border-radius: 8px; padding: 20px; margin-bottom: 24px; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }}
    .filters {{ background: #fff; border-radius: 8px; padding: 16px 20px; margin-bottom: 24px; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }}
    .filters h3 {{ margin-top: 0; font-size: 16px; }}
    .filter-group {{ display: inline-block; margin-right: 24px; vertical-align: top; }}
    .filter-group label {{ display: block; font-size: 13px; margin: 2px 0; }}
    .filter-group label input {{ margin-right: 4px; }}
    canvas {{ max-height: 400px; }}
</style>
</head>
<body>

<div class="container">
    <h1>Benchmark Report</h1>

    <div class="filters" id="filters">
        <h3>Scenario Filter</h3>
        <div id="filter-controls"></div>
    </div>

    <div class="chart-container">
        <h2>Throughput vs Concurrency</h2>
        <ul class="nav nav-tabs" role="tablist">
            {subchart_tabs}
        </ul>
        <div class="tab-content">
            {subchart_canvases}
        </div>
    </div>

    <div class="chart-container">
        <h2>Peak Throughput by Scenario</h2>
        <canvas id="peakChart"></canvas>
    </div>

    <div class="chart-container">
        <h2>Latency Percentiles</h2>
        <canvas id="latencyChart"></canvas>
    </div>
</div>

<script>
const BENCH_DATA = {results_json};

const COLORS = [
    'rgba(54, 162, 235, 0.8)', 'rgba(255, 99, 132, 0.8)',
    'rgba(75, 192, 192, 0.8)', 'rgba(255, 159, 64, 0.8)',
    'rgba(153, 102, 255, 0.8)', 'rgba(255, 205, 86, 0.8)',
    'rgba(201, 203, 207, 0.8)', 'rgba(34, 139, 34, 0.8)',
];

function buildFilterControls() {{
    var bpSet = new Set(BENCH_DATA.map(function(r) {{ return r.blueprint_count; }}));
    var affSet = new Set(BENCH_DATA.map(function(r) {{ return r.affix_count; }}));
    var attrSet = new Set(BENCH_DATA.map(function(r) {{ return r.attribute_count; }}));

    var html = '';
    if (bpSet.size > 0) {{
        html += '<div class="filter-group"><strong>Blueprints</strong>';
        bpSet.forEach(function(v) {{
            html += '<label><input type="checkbox" class="filter-bp" value="' + v + '" checked onchange="applyFilters()"> ' + v + '</label>';
        }});
        html += '</div>';
    }}
    if (affSet.size > 0) {{
        html += '<div class="filter-group"><strong>Affixes</strong>';
        affSet.forEach(function(v) {{
            html += '<label><input type="checkbox" class="filter-aff" value="' + v + '" checked onchange="applyFilters()"> ' + v + '</label>';
        }});
        html += '</div>';
    }}
    if (attrSet.size > 0) {{
        html += '<div class="filter-group"><strong>Attributes</strong>';
        attrSet.forEach(function(v) {{
            html += '<label><input type="checkbox" class="filter-attr" value="' + v + '" checked onchange="applyFilters()"> ' + v + '</label>';
        }});
        html += '</div>';
    }}
    document.getElementById('filter-controls').innerHTML = html;
}}

function getFilteredData() {{
    var checkedBP = Array.from(document.querySelectorAll('.filter-bp:checked')).map(function(cb) {{ return Number(cb.value); }});
    var checkedAff = Array.from(document.querySelectorAll('.filter-aff:checked')).map(function(cb) {{ return Number(cb.value); }});
    var checkedAttr = Array.from(document.querySelectorAll('.filter-attr:checked')).map(function(cb) {{ return Number(cb.value); }});
    return BENCH_DATA.filter(function(r) {{
        return checkedBP.indexOf(r.blueprint_count) !== -1 &&
               checkedAff.indexOf(r.affix_count) !== -1 &&
               checkedAttr.indexOf(r.attribute_count) !== -1;
    }});
}}

var peakChart, latencyChart;

function applyFilters() {{
    var filtered = getFilteredData();
    updatePeakChart(filtered);
    updateLatencyChart(filtered);
    updateThroughputCharts(filtered);
}}

function getAttrColor(attrCount) {{
    if (attrCount <= 3) return 'rgba(54, 162, 235, 0.8)';
    if (attrCount <= 10) return 'rgba(255, 159, 64, 0.8)';
    return 'rgba(255, 99, 132, 0.8)';
}}

function updatePeakChart(filtered) {{
    var sorted = filtered.slice().sort(function(a, b) {{ return parseFloat(b.throughput) - parseFloat(a.throughput); }});
    var labels = sorted.map(function(r) {{ return 'bp' + r.blueprint_count + ' aff' + r.affix_count + ' attr' + r.attribute_count; }});
    var data = sorted.map(function(r) {{ return parseFloat(r.throughput); }});
    var bgColors = sorted.map(function(r) {{ return getAttrColor(r.attribute_count); }});

    if (peakChart) peakChart.destroy();
    peakChart = new Chart(document.getElementById('peakChart'), {{
        type: 'bar',
        data: {{
            labels: labels,
            datasets: [{{
                label: 'gen/s',
                data: data,
                backgroundColor: bgColors,
            }}],
        }},
        options: {{
            responsive: true,
            plugins: {{
                title: {{ display: true, text: 'Peak Throughput by Scenario' }},
                legend: {{ display: false }},
            }},
            scales: {{
                x: {{ title: {{ display: true, text: 'Scenario' }} }},
                y: {{ title: {{ display: true, text: 'gen/s' }}, beginAtZero: true }},
            }},
        }},
    }});
}}

function updateLatencyChart(filtered) {{
    var withLatency = filtered.filter(function(r) {{ return r.p50_ms !== ''; }});
    var labels = withLatency.map(function(r) {{ return 'bp' + r.blueprint_count + ' aff' + r.affix_count + ' attr' + r.attribute_count; }});
    var p50 = withLatency.map(function(r) {{ return parseFloat(r.p50_ms); }});
    var p95 = withLatency.map(function(r) {{ return parseFloat(r.p95_ms); }});
    var p99 = withLatency.map(function(r) {{ return parseFloat(r.p99_ms); }});

    if (latencyChart) latencyChart.destroy();
    latencyChart = new Chart(document.getElementById('latencyChart'), {{
        type: 'bar',
        data: {{
            labels: labels,
            datasets: [
                {{ label: 'p50 (ms)', data: p50, backgroundColor: 'rgba(54, 162, 235, 0.7)' }},
                {{ label: 'p95 (ms)', data: p95, backgroundColor: 'rgba(255, 159, 64, 0.7)' }},
                {{ label: 'p99 (ms)', data: p99, backgroundColor: 'rgba(255, 99, 132, 0.7)' }},
            ],
        }},
        options: {{
            responsive: true,
            plugins: {{
                title: {{ display: true, text: 'Latency Percentiles' }},
                legend: {{ position: 'bottom' }},
            }},
            scales: {{
                x: {{ title: {{ display: true, text: 'Scenario' }} }},
                y: {{ title: {{ display: true, text: 'ms' }}, beginAtZero: true }},
            }},
        }},
    }});
}}

var throughputCharts = {{}};

function updateThroughputCharts(filtered) {{
    var bpLevels = [...new Set(BENCH_DATA.map(function(r) {{ return r.blueprint_count; }}))].sort(function(a, b) {{ return a - b; }});
    bpLevels.forEach(function(bp) {{
        var bpData = filtered.filter(function(r) {{ return r.blueprint_count === bp; }});
        var seriesMap = {{}};
        bpData.forEach(function(r) {{
            var key = 'bp' + r.blueprint_count + ' aff' + r.affix_count + ' attr' + r.attribute_count;
            if (!seriesMap[key]) seriesMap[key] = [];
            seriesMap[key].push({{ x: r.concurrency, y: parseFloat(r.throughput) }});
        }});

        var datasets = [];
        var colorIdx = 0;
        for (var key in seriesMap) {{
            datasets.push({{
                label: key,
                data: seriesMap[key],
                borderColor: COLORS[colorIdx % COLORS.length],
                backgroundColor: COLORS[colorIdx % COLORS.length].replace('0.8', '0.2'),
                fill: false,
                tension: 0.1,
                pointRadius: 5,
            }});
            colorIdx++;
        }}

        if (throughputCharts[bp]) throughputCharts[bp].destroy();

        var canvas = document.getElementById('throughputChart-' + bp);
        if (canvas) {{
            throughputCharts[bp] = new Chart(canvas, {{
                type: 'line',
                data: {{ datasets: datasets }},
                options: {{
                    responsive: true,
                    plugins: {{
                        title: {{ display: true, text: 'Throughput vs Concurrency (bp=' + bp + ')' }},
                        legend: {{ position: 'bottom' }},
                    }},
                    scales: {{
                        x: {{ title: {{ display: true, text: 'Concurrency' }}, type: 'linear' }},
                        y: {{ title: {{ display: true, text: 'gen/s' }}, beginAtZero: true }},
                    }},
                }},
            }});
        }}
    }});
}}

function switchBlueprint() {{
    applyFilters();
}}

buildFilterControls();
var initialFiltered = getFilteredData();
updatePeakChart(initialFiltered);
updateLatencyChart(initialFiltered);
updateThroughputCharts(initialFiltered);
</script>
</body>
</html>"""

    with open(html_path, "w") as f:
        f.write(html_content)


def _run_sweep(api_key, target_url, config_path, concurrency=1, duration=10,
               sample_duration=None):
    """Run a multi-scenario sweep from YAML config.

    For each scenario: create a named client, seed data with the synthetic
    generator, run the fixed-concurrency bench loop, collect latency sample,
    record results, then clean up all clients at the end.
    """
    try:
        scenarios = load_config(config_path)
    except ValueError as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)

    results = []
    client_cleanups = []

    for i, scenario in enumerate(scenarios):
        bp = scenario["blueprint_count"]
        af = scenario.get("affix_count", 0)
        at = scenario["attribute_count"]
        client_name = f"bench-bp{bp}-aff{af}-attr{at}"

        try:
            client_id = create_client(api_key, target_url, name=client_name)
            key_id, scoped_key = create_api_key(client_id, api_key, target_url)

            for j in range(bp):
                create_blueprint(
                    client_id, scoped_key, f"Blueprint-{j:04d}", target_url,
                    attribute_count=at, scenario=scenario, blueprint_idx=j,
                )

            total, elapsed, rate = _run_bench_loop(
                scoped_key, target_url, concurrency, duration,
            )

            stable_tracker = _collect_stable_sample(
                scoped_key, target_url, concurrency=concurrency,
                sample_duration=sample_duration,
            )

            result = {
                "blueprint_count": bp,
                "affix_count": af,
                "attribute_count": at,
                "concurrency": concurrency,
                "duration_s": f"{elapsed:.2f}",
                "total_requests": total,
                "throughput": f"{rate:.2f}",
                "p50_ms": "",
                "p95_ms": "",
                "p99_ms": "",
            }

            if stable_tracker.count >= 50:
                result["p50_ms"] = f"{stable_tracker.p50():.2f}"
                result["p95_ms"] = f"{stable_tracker.p95():.2f}"
                result["p99_ms"] = f"{stable_tracker.p99():.2f}"

            results.append(result)

            client_cleanups.append((client_id, key_id))
            print(f"[{i + 1}/{len(scenarios)}] bp={bp} aff={af} attr={at} ... {rate:.0f} gen/s")

        except HTTPError as e:
            _handle_request_error(e, target_url)
        except URLError as e:
            _handle_request_error(e, target_url)

    csv_path = "bench/results.csv"
    os.makedirs(os.path.dirname(csv_path), exist_ok=True)
    with open(csv_path, "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=[
            "blueprint_count", "affix_count", "attribute_count",
            "concurrency", "duration_s", "total_requests", "throughput",
            "p50_ms", "p95_ms", "p99_ms",
        ])
        writer.writeheader()
        writer.writerows(results)

    print(f"\nResults written to {csv_path}")

    html_path = "bench/report.html"
    _generate_html_report(results, html_path)
    print(f"Report written to {html_path}")

    for client_id, key_id in client_cleanups:
        try:
            _request_json(
                "DELETE", f"/api/clients/{client_id}/keys/{key_id}",
                None, api_key, target_url,
            )
            _request_json(
                "DELETE", f"/api/clients/{client_id}",
                None, api_key, target_url,
            )
        except (HTTPError, URLError):
            pass

    print(f"Cleaned up {len(client_cleanups)} test client(s)")


if __name__ == "__main__":
    main()
