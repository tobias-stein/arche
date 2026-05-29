---
title: Batch generate endpoint — POST /api/generate/batch
status: ready-for-agent
---

## What to build

Add a new endpoint `POST /api/generate/batch` that accepts an array of generation requests and returns an array of responses in a single HTTP round-trip. This amortizes the auth overhead, cache lock acquisition, JSON parsing, and response serialization across N generations.

For batch sizes of 100+, this can deliver the largest throughput multiplier — potentially 1000+ effective gen/s even with the current single-request throughput of ~2 gen/s.

**Design:**

- New request type in `arche-types/src/generate.rs`: `BatchGenerateRequest { requests: Vec<GenerateRequest> }`
- New response type: `BatchGenerateResponse { results: Vec<GenerateResponse> }`
- New handler `generate_batch_handler` in `generate.rs`:
  - Authenticate once (shared across all N requests)
  - Read cache once (shared across all N requests)
  - Loop through requests, running the full generation pipeline for each
  - Use a fresh `StdRng` per request (seeded individually)
- Handle partial failures: if one request fails (e.g., no matching blueprint), include an error entry in the response array rather than failing the whole batch
- Error response format for individual failures can be a null result with an error message
- Add caching of the resolved attribute data across the batch (reuse the resolved `HashMap` for the same blueprint)

**Acceptance criteria:**

- [ ] New `POST /api/generate/batch` endpoint exists
- [ ] Accepts array of generation requests, returns array of responses
- [ ] Auth and cache lock acquired once per batch, not per request
- [ ] Individual request failures produce error entries instead of failing the whole batch
- [ ] Deterministic output for same seeds (same as single generate)
- [ ] Benchmark at batch size 100 shows 500+ effective gen/s
- [ ] Schema endpoint updated to include batch generate schema

## Blocked by

None — can start immediately
