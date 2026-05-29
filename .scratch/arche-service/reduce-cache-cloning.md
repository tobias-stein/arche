---
title: Reduce Vec cloning in cache read path
status: ready-for-agent
---

## What to build

Every generate request in `generate.rs:86-90` clones three `Vec<Arc<T>>` from the cache:

```rust
client_cache = ClientCache {
    blueprints: cc.blueprints.clone(), // clones Vec<Arc<Blueprint>>
    affixes: cc.affixes.clone(),       // clones Vec<Arc<Affix>>
    global_meta_attributes: cc.global_meta_attributes.clone(), // clones Vec<Arc<GlobalMetaAttribute>>
};
```

While Arc clones are cheap (ref-count increment), the Vec allocation itself happens per request. For 1000 blueprints, this is allocating ~24KB + 1000 ref-count bumps per request. Combined with the `bp_affixes_lookup` HashMap clone on line 92, this adds measurable allocation pressure.

**Design:**

Several approaches, from simplest to most impactful:
1. **Store `Arc<Vec<Arc<T>>>` in the cache** — the Vec itself is wrapped in an Arc, so cloning is a single ref-count bump.
2. **Use `Arc<ClientCache>` directly** — wrap the entire `ClientCache` in an Arc in the cache's `by_client` map. The generate handler clones the Arc (one ref bump) instead of cloning inner Vecs.
3. **Return `&ClientCache` under the read lock** — instead of cloning, keep the read lock held throughout generation. This avoids all allocation but holds the lock longer.

Approach 2 (Arc<ClientCache>) is the recommended balance: simple change, minimal allocation, no lock contention increase. Store `HashMap<Uuid, Arc<ClientCache>>` in the cache.

**Acceptance criteria:**

- [ ] `by_client` stores `Arc<ClientCache>` instead of `ClientCache`
- [ ] Generate handler clones the Arc instead of cloning inner Vecs
- [ ] All existing cache tests still pass
- [ ] Benchmark shows reduced allocation rate and improved throughput

## Blocked by

None — can start immediately
