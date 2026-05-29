---
title: "Benchmark: Single-scenario data seeding"
status: completed
---

## Parent

PRD: `.scratch/benchmark-suite/prd.md`

## What to build

Extend `bench/run.py` with a `seed` subcommand (e.g. `--seed`) that:

1. Creates a client via `POST /api/clients` with a hardcoded name like `bench-scratch`
2. Creates a client-scoped API key with `generate` permission
3. Creates N blueprints with hardcoded data:
   - 10 blueprints, named `Blueprint-0000` through `Blueprint-0009`
   - 3 range-type attributes each (e.g. `attr_0` with min=0 max=100, uniform distribution)
   - 0 affixes
4. Prints the `client_id` and the new API key
5. The seeded data is deterministic (no randomness yet)

## Acceptance criteria

- [ ] `python bench/run.py --api-key <super-key> --seed` creates a client, blueprints, and prints client_id
- [ ] After seeding, `GET /api/seeded-client-id/blueprints` returns 10 blueprints
- [ ] Exits with code 1 on any error

## Blocked by

- `bench-02-api-connectivity` (needs HTTP client infrastructure)
