"""
Benchmark suite for Arche generate endpoint.

Supports:
  - API connectivity check: send a single generate request
  - Config loading: read YAML dimension sweep config
  - Data seeding: create client, API key, and blueprints

Usage:
    python bench/run.py --api-key <key>
    python bench/run.py --api-key <key> --target-url http://other-host:8080
    python bench/run.py --api-key <key> --seed
    python bench/run.py --config bench/scenarios.yaml --print-scenarios
"""

import argparse
import itertools
import json
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


def create_client(api_key, target_url="http://localhost:8080"):
    """Create a client named 'bench-scratch' and return its id."""
    body = {"name": "bench-scratch"}
    resp = _request_json("POST", "/api/clients", body, api_key, target_url)
    return resp["id"]


def create_api_key(client_id, api_key, target_url="http://localhost:8080"):
    """Create a client-scoped API key with generate permission.

    Returns the raw API key string.
    """
    body = {"name": "benchmark-key", "permissions": ["generate"]}
    resp = _request_json(
        "POST", f"/api/clients/{client_id}/keys", body, api_key, target_url
    )
    return resp["key"]


def _build_blueprint_body(name):
    """Build the request body for creating a blueprint with 3 range attributes."""
    attributes = {}
    attribute_order = []
    for i in range(3):
        attr_name = f"attr_{i}"
        attributes[attr_name] = {
            "description": f"Attribute {attr_name}",
            "valueType": "range",
            "min": 0.0,
            "max": 100.0,
            "distribution": {"type": "uniform"},
        }
        attribute_order.append(attr_name)
    return {
        "name": name,
        "archetype": "item",
        "weight": 1.0,
        "attributes": attributes,
        "attributeOrder": attribute_order,
        "affixes": {
            "minPrefixes": 0,
            "maxPrefixes": 0,
            "minSuffixes": 0,
            "maxSuffixes": 0,
            "prefixes": [],
            "suffixes": [],
        },
    }


def create_blueprint(client_id, api_key, name, target_url="http://localhost:8080"):
    """Create a blueprint with 3 range attributes and return its id."""
    body = _build_blueprint_body(name)
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
    return parser.parse_args(argv)


def main():
    args = _parse_args()

    if args.api_key:
        if args.seed:
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


def _run_seed(api_key, target_url):
    """Seed test data: create client, API key, and 10 blueprints."""
    try:
        client_id = create_client(api_key, target_url)
        scoped_key = create_api_key(client_id, api_key, target_url)

        for i in range(10):
            name = f"Blueprint-{i:04d}"
            create_blueprint(client_id, scoped_key, name, target_url)

        print(f"Client ID: {client_id}")
        print(f"API Key: {scoped_key}")
    except HTTPError as e:
        _handle_request_error(e, target_url)
    except URLError as e:
        _handle_request_error(e, target_url)


if __name__ == "__main__":
    main()
