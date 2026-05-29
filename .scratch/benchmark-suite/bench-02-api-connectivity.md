---
title: "Benchmark: API connectivity + single generate"
status: ready-for-agent
---

## Parent

PRD: `.scratch/benchmark-suite/prd.md`

## What to build

A `bench/run.py` Python script that:

1. Accepts `--api-key` and `--target-url` arguments (defaults: `http://localhost:8080`)
2. Sends a single `POST /api/generate` with an empty body `{}` and the `X-API-Key` header
3. Prints the full response (status code, body, elapsed time in ms)
4. Exits with non-zero code if the request fails or returns an error

## Acceptance criteria

- [ ] `python bench/run.py --api-key <key>` sends one generate request and prints the response
- [ ] `python bench/run.py --api-key <key> --target-url http://other-host:8080` works
- [ ] Exits with code 1 on any error (connection refused, auth failure, server error)
- [ ] Prints elapsed time in milliseconds

## Blocked by

- `bench-01-stack-orchestration` (needs a running stack)
