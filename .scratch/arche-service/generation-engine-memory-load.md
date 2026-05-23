---
title: Generation engine — in-memory data loading
status: ready-for-agent
---

## Parent

Arche API Service PRD (see `.scratch/arche-service/prd.md`)

## What to build

Implement the in-memory cache structs and the startup loading logic for the generation engine.

Define Cache structs:
```rust
struct Cache {
    clients: HashMap<Uuid, Client>,
    blueprints: HashMap<Uuid, Blueprint>,
    affixes: HashMap<Uuid, Affix>,
    global_meta_attributes: HashMap<Uuid, GlobalMetaAttribute>,
    blueprint_affixes: HashMap<Uuid, Vec<BlueprintAffix>>,
    by_client: HashMap<Uuid, ClientCache>,
}

struct ClientCache {
    blueprints: Vec<Arc<Blueprint>>,
    affixes: Vec<Arc<Affix>>,
    global_meta_attributes: Vec<Arc<GlobalMetaAttribute>>,
}
```

On startup, load ALL definitions from the database into memory. For each client, collect their blueprints (with resolved `blueprint_affixes`), affixes, and global meta attributes.

Provide a `Cache::get_client_data(client_id) -> Option<&ClientCache>` method used by the generation engine. The Cache is wrapped in `Arc<RwLock<>>` for concurrent access.

## Acceptance criteria

- [ ] Cache struct defined with all required maps
- [ ] `Cache::load(db_pool)` loads all data from DB
- [ ] ClientCache groups data per client for generation lookups
- [ ] `get_client_data` returns correct slice for a client
- [ ] `Cache` is thread-safe (Send + Sync)
- [ ] Unit tests with mock data verify correct loading
- [ ] Integration test: populate DB, start service, verify cache loaded correctly

## Blocked by

- arche-types/database-migrations.md
- arche-types/domain-structs-enums.md
