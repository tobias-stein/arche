---
title: "Benchmark: YAML config + cross-product"
status: ready-for-agent
---

## Parent

PRD: `.scratch/benchmark-suite/prd.md`

## What to build

A `bench/scenarios.yaml` file and a config loader function in `bench/run.py` that:

1. Reads a YAML file with the following structure:
   ```yaml
   dimensions:
     blueprints: [10, 100, 1000]
     affixes: [0, 2, 4]
     attributes: [3, 10, 25]
   ```
2. Computes the cross-product of all dimension levels, producing a list of scenario descriptors
3. Each scenario descriptor is a dict: `{ "blueprint_count": 10, "affix_count": 0, "attribute_count": 3 }`
4. Supports being used both as a module import and via CLI: `python bench/run.py --config bench/scenarios.yaml --print-scenarios`
5. The config file path is configurable via `--config` argument

## Acceptance criteria

- [ ] `python bench/run.py --config bench/scenarios.yaml --print-scenarios` prints all 27 scenario combinations
- [ ] Loading a YAML with 2 levels per dimension produces 8 scenarios
- [ ] Loading a YAML with a single level per dimension produces 1 scenario
- [ ] Exits with a clear error on missing/malformed YAML

## Blocked by

None — this is a pure parser, testable without a running stack.
