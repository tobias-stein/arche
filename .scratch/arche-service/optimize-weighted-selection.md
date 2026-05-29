---
title: Optimize weighted selection without replacement algorithm
status: ready-for-agent
---

## What to build

The `weighted_select_without_replacement` function in `affix_selection.rs:181-209` uses an O(n²) algorithm:

1. On each pick, it rebuilds a `WeightedIndex` from scratch (line 198-199) — O(pool_size) each time
2. It removes the selected index via `indices.remove(pick)` (line 205) — O(remaining) due to element shifting

For blueprints with large affix pools, this quadratic behavior adds up. Replace it with a more efficient algorithm.

**Design:**

Replace the current loop-based approach with one of:
- **Weighted reservoir sampling** (Algorithm A-ES from Efraimidis & Spirakis) — single pass, O(n) total, no removal
- **Alias method** — build the alias table once in O(n), then each selection is O(1)
- **Floyd's algorithm** with weighted selection — O(k) for selecting k items

The alias method is the best fit: build once in O(pool_size), then each individual pick is O(1). Since we pick `rolled_count` items (typically small: 0-5), the overhead of building the alias table is amortized.

**Acceptance criteria:**

- [ ] Weighted selection algorithm is O(n) or better (not O(n²))
- [ ] Selection probabilities are preserved (same distribution as before)
- [ ] Determinism with same seed is preserved
- [ ] All affix selection tests still pass
- [ ] Benchmark shows improvement for scenarios with affixes > 0

## Blocked by

None — can start immediately
