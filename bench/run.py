"""
Benchmark suite for Arche generate endpoint.

Supports:
  - API connectivity check: send a single generate request
  - Config loading: read YAML dimension sweep config

Usage:
    python bench/run.py --api-key <key>
    python bench/run.py --api-key <key> --target-url http://other-host:8080
    python bench/run.py --config bench/scenarios.yaml --print-scenarios
"""

import argparse
import itertools
import re
import sys
import time
import urllib.request
from urllib.error import HTTPError, URLError


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


def main():
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
    args = parser.parse_args()

    if args.api_key:
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
        print(f"Error: HTTP {e.code} {e.reason}", file=sys.stderr)
        try:
            error_body = e.read().decode("utf-8", errors="replace")
            print(f"Response body: {error_body}", file=sys.stderr)
        except Exception:
            pass
        sys.exit(1)
    except URLError as e:
        print(f"Error: Could not reach server at {target_url}", file=sys.stderr)
        print(f"Reason: {e.reason}", file=sys.stderr)
        sys.exit(1)

    print(f"Status: {status}")
    print(f"Body: {body}")
    print(f"Elapsed: {elapsed_ms:.2f} ms")


if __name__ == "__main__":
    main()
