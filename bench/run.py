"""
Benchmark config loader.

Reads a YAML config file defining dimension sweeps and produces
cross-product scenario descriptors.

Usage:
    python bench/run.py --config bench/scenarios.yaml --print-scenarios
"""

import argparse
import itertools
import re
import sys


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
        description="Benchmark config loader for Arche throughput suite"
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

    try:
        scenarios = load_config(args.config)
    except ValueError as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)

    if args.print_scenarios:
        for s in scenarios:
            print(f"  {_format_scenario(s)}")
        print(f"\nTotal: {len(scenarios)} scenario(s)")


if __name__ == "__main__":
    main()
