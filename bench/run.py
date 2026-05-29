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


def _print_summary_table(results):
    """Print a terminal summary table at the end of a sweep.

    Columns: Scenario, Peak gen/s, Concurrency, p50(ms), p95(ms), p99(ms).
    Footer shows overall peak throughput and best scenario.
    Column widths auto-adjust to content.
    """
    if not results:
        return

    headers = ["Scenario", "Peak gen/s", "Concurrency", "p50(ms)", "p95(ms)", "p99(ms)"]

    rows = []
    for r in results:
        label = f"bp={r['blueprint_count']}   aff={r.get('affix_count', 0)}  attr={r['attribute_count']}"
        peak = r['throughput']
        conc = r['concurrency']
        p50_s = f"{r['p50']:.1f}" if r.get('p50') is not None else "\u2014"
        p95_s = f"{r['p95']:.1f}" if r.get('p95') is not None else "\u2014"
        p99_s = f"{r['p99']:.1f}" if r.get('p99') is not None else "\u2014"
        rows.append((label, peak, conc, p50_s, p95_s, p99_s))

    col_widths = [
        max(len(headers[0]), max(len(r[0]) for r in rows)),
        max(len(headers[1]), max(len(f"{r[1]:.0f}") for r in rows)),
        max(len(headers[2]), max(len(str(r[2])) for r in rows)),
        max(len(headers[3]), max(len(r[3]) for r in rows)),
        max(len(headers[4]), max(len(r[4]) for r in rows)),
        max(len(headers[5]), max(len(r[5]) for r in rows)),
    ]

    sep = "  "
    parts = []
    parts.append(f"{{:<{col_widths[0]}}}")
    for i in range(1, 6):
        parts.append(f"{{:>{col_widths[i]}}}")
    fmt = sep.join(parts)

    print()
    print(fmt.format(*headers))
    for label, peak, conc, p50_s, p95_s, p99_s in rows:
        print(fmt.format(label, int(round(peak)), conc, p50_s, p95_s, p99_s))

    best = max(results, key=lambda r: r['throughput'])
    best_label = f"bp={best['blueprint_count']} aff={best.get('affix_count', 0)} attr={best['attribute_count']}"
    print()
    print(f"Peak gen/s: {best['throughput']:.0f}  ({best_label})")
    print()


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


def _run_sweep(api_key, target_url, config_path, concurrency=1, duration=10):
    """Run a multi-scenario sweep from YAML config.

    For each scenario: create a named client, seed data with the synthetic
    generator, run the fixed-concurrency bench loop, record results, collect
    latency sample, then clean up all clients at the end.
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
            )
            p50 = round(stable_tracker.p50(), 1) if stable_tracker.count >= 50 else None
            p95 = round(stable_tracker.p95(), 1) if stable_tracker.count >= 50 else None
            p99 = round(stable_tracker.p99(), 1) if stable_tracker.count >= 50 else None

            results.append({
                "blueprint_count": bp,
                "affix_count": af,
                "attribute_count": at,
                "concurrency": concurrency,
                "duration_s": elapsed,
                "total_requests": total,
                "throughput": rate,
                "p50": p50,
                "p95": p95,
                "p99": p99,
            })

            client_cleanups.append((client_id, key_id))
            print(f"[{i + 1}/{len(scenarios)}] bp={bp} aff={af} attr={at} ... {rate:.0f} gen/s")

        except HTTPError as e:
            _handle_request_error(e, target_url)
        except URLError as e:
            _handle_request_error(e, target_url)

    csv_path = "bench/results.csv"
    os.makedirs(os.path.dirname(csv_path), exist_ok=True)
    csv_rows = [
        {
            "blueprint_count": r["blueprint_count"],
            "affix_count": r["affix_count"],
            "attribute_count": r["attribute_count"],
            "concurrency": r["concurrency"],
            "duration_s": f"{r['duration_s']:.2f}",
            "total_requests": r["total_requests"],
            "throughput": f"{r['throughput']:.2f}",
        }
        for r in results
    ]
    with open(csv_path, "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=[
            "blueprint_count", "affix_count", "attribute_count",
            "concurrency", "duration_s", "total_requests", "throughput",
        ])
        writer.writeheader()
        writer.writerows(csv_rows)

    print(f"\nResults written to {csv_path}")

    _print_summary_table(results)

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
