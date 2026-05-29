---
title: Cache API keys in memory to eliminate DB + bcrypt on every request
status: completed
---

## What to build

Every authenticated request currently does a full `SELECT * FROM api_keys` scan and iterates every row calling `bcrypt::verify()` until a match is found. This is the #1 throughput bottleneck — at conc=100, the 5 connection pool slots are all stuck doing auth, and throughput drops to 0 gen/s.

Build an in-memory API key cache that loads all keys at startup (alongside the existing `Cache`) and is used by the auth middleware instead of hitting the database. The cache should be kept fresh via the existing polling mechanism or Redis pub/sub.

**Design:**

- Add a new `ApiKeyCache` struct holding `HashMap<String, AuthenticatedKey>` keyed by a fast hash (e.g., SHA-256 hex digest of the raw key string, or just the raw key itself since this is in-memory only).
- Load it in `main.rs` alongside the existing `Cache::load()` call.
- In `verify_api_key()` in `auth/mod.rs`, first try the cache lookup. Fall back to DB only on cache miss (and populate the cache).
- The cache poller should also refresh API keys — add a `load_api_keys()` query and `apply_api_key_data()` method.
- Handle expiry: if `expires_at` is in the past, treat as not found.
- Handle cache invalidation: key created/deleted → poller picks it up.
- The `blueprint_affixes` HashMap clone in `generate.rs:92` should also be avoided — pre-store it in the cache alongside the ClientCache data.

**Acceptance criteria:**

- [ ] Auth middleware does a HashMap lookup first, never queries DB on cache hit
- [ ] Cache is populated at startup with all API keys
- [ ] Cache poller picks up new/deleted/expired keys
- [ ] Benchmark at conc=100 shows dramatic throughput increase (from 0 gen/s to 50+ gen/s)
- [ ] All existing auth tests still pass
- [ ] Expired keys are rejected even if cached

## Blocked by

None — can start immediately
