---
title: Arche API Service
status: completed
---

## Problem Statement

Game developers need a backend service that can generate game items/things procedurally with configurable blueprints, affixes, attribute distributions, and constraint matching. The system must support multi-tenant isolation (multiple games), high-throughput generation (100–1M req/s), and a full CRUD API for managing definitions. It must be deployable in both single-instance mode (small games, no Redis) and cluster mode (large scale, Redis pub/sub for cache coherency).

No existing service combines weighted-random blueprint selection, overlap-semantics constraint matching, configurable distributions (uniform, normal, exponential), affix selection without replacement, and `$ref_id`-based global attribute sharing — all scoped to per-client tenancy with audit logging and import/export.

## Solution

A Rust API service named **Arche** that provides:

1. **Generation engine** — Blueprint selection via weighted random (respecting constraints with overlap semantics), attribute value rolling with configurable distributions, affix selection via weighted random without replacement, name composition (prefix + base name + suffix).
2. **REST API** — Full CRUD for blueprints, affixes, global meta attributes, clients, and API keys. Batch operations. JSON Schema introspection endpoints. Cursor-based pagination.
3. **In-memory cache** — All definitions loaded at startup. Cache invalidation via polling (single mode) or Redis pub/sub (cluster mode). The `/generate` endpoint never touches the database.
4. **Authentication & authorization** — Two-tier API key model (super admin + per-client RBAC keys). bcrypt hashing. Permission scoping per endpoint.
5. **Audit logging** — All CRUD mutations recorded with before/after JSON snapshots, actor identity, and timestamp.
6. **Import/export** — ZIP-based archive with folder-per-client structure. Conflict resolution at per-attribute granularity. Single-transaction import with deferred cache invalidation.
7. **Consistency enforcement** — Cascade-delete handling, auto-adjustment of affix counts on affix removal, `$ref_id` resolution at generation time, attribute order completeness validation.

## User Stories

### Generation

1. As a game developer calling the API, I want to generate a random thing by calling `POST /api/generate` with an empty body, so that I get a random result from any blueprint across the current client.
2. As a game developer, I want to constrain generation by archetype (e.g., `"archetype": "sword"`), so that I only get results from a specific category of blueprints.
3. As a game developer, I want to constrain generation by attribute values with overlap semantics (e.g., `damage >= 15`), so that I can filter blueprints that can produce values in my desired range.
4. As a game developer, I want to constrain by multiple attributes simultaneously, so that I can find a blueprint matching all my criteria.
5. As a game developer, I want to control affix counts (min/max prefixes and suffixes) in the generate request, so that I can specify how many modifiers the generated thing should have.
6. As a game developer, I want to require specific affixes (`require: [id]`), so that certain modifiers are guaranteed in the output.
7. As a game developer, I want to block specific affixes (`block: [id]`), so that certain modifiers are excluded from the output.
8. As a game developer, I want to pass a seed (`seed: 12345`) for reproducible generation, so that I can debug specific outputs deterministically.
9. As a game developer, I want the service to return a random seed if I don't provide one, so that each call produces fresh results by default.
10. As a game developer, I want the generate response to include the blueprint attributes (in `attribute_order`), the selected affix attributes (in `sort_order`), and the composed name with its parts, so that I have everything needed to construct the final thing.
11. As a game developer, I want the service to return RFC 9457 Problem JSON with status 404 when no blueprints match my constraints, so that I can distinguish "no match" from "server error."
12. As a game developer, I want the `X-Cache-Refresh: true` header to force a fresh DB load for a single generate request, so that I can bypass stale cache data when needed.

### Blueprints CRUD

13. As an admin, I want to create a blueprint with inline attributes and `$ref_id` references to global meta attributes, so that I can reuse common definitions.
14. As an admin, I want to define attribute value types as single, enum, range, string, or boolean, so that I can model any game attribute.
15. As an admin, I want range values to support configurable distributions (uniform, normal with mean/std_dev, exponential with rate), so that rolled values follow realistic patterns.
16. As an admin, I want to define an affix pool per blueprint with per-affix weights and separate min/max counts for prefixes and suffixes, so that I control affix availability.
17. As an admin, I want `attribute_order` to be validated at write time (all keys present, no extras, no duplicates), so that the display order is always consistent.
18. As an admin, I want to update any blueprint field, so that I can iterate on designs.
19. As an admin, I want to delete a blueprint (rejected by default if referenced), so that I can clean up unused definitions.
20. As an admin, I want to force-delete a blueprint (cascading deletion of affix references), so that I can remove it even when it has dependencies.
21. As an admin, I want to list blueprints with cursor-based pagination, filtering by archetype and name search, so that I can browse efficiently via API.

### Affixes CRUD

22. As an admin, I want to create an affix as a prefix or suffix with exactly one meta attribute (inline or `$ref_id`), so that I can define modifiers.
23. As an admin, I want to update an affix's attribute or name, so that I can correct mistakes.
24. As an admin, I want to delete an affix (rejected by default if referenced by any blueprint), so that I can remove unused modifiers.
25. As an admin, I want to force-delete an affix (cascading: remove from all blueprint pools, auto-adjust min/max counts), so that I can delete it even when it's in use.
26. As an admin, I want auto-adjustment of min/max affix counts to be logged as an `adjusted` audit action, so that I know my blueprints were modified.

### Global Meta Attributes CRUD

27. As an admin, I want to create global meta attributes with a value type and type-specific payload, so that I have a shared library of attribute definitions.
28. As an admin, I want to update a global meta attribute, with changes reflected automatically at generation time (resolution happens at generation, not write), so that updates are immediate.
29. As an admin, I want to delete a global meta attribute (rejected by default if referenced), so that I can clean up unused definitions.
30. As an admin, I want to force-delete a global meta attribute (cascading: remove all `$ref_id` references from blueprints and affixes), so that I can delete it even when referenced.

### Client & Key Management

31. As a super admin, I want to create and delete clients, so that I can onboard and remove games.
32. As a super admin, I want to list all clients, so that I can see what tenants exist.
33. As a super admin, I want to create per-client API keys with a permission set (read, write, delete, generate, admin), so that game servers and tools get scoped access.
34. As a super admin, I want the raw API key returned exactly once on creation (bcrypt-hashed thereafter), so that the caller can capture it securely.
35. As a super admin, I want to list and revoke API keys per client, so that I can rotate credentials.
36. As a super admin, I want client deletion to be rejected if the client has active API keys, so that I don't accidentally orphan keys.

### Audit Log

37. As a super admin, I want all CRUD mutations (except generate) to be recorded in the audit log with before/after JSON snapshots, actor key ID, resource type/ID, action, and timestamp, so that I have a full change history.
38. As a super admin, I want cascade adjustments (e.g., affix force-delete) to be logged as action `adjusted`, so that automated changes are distinguishable from user-initiated ones.
39. As a super admin, I want to query the audit log with filters (client, resource type, action, actor, cursor/limit), so that I can find specific changes.

### Import / Export

40. As a super admin, I want to export client data as a ZIP archive with folder-per-client structure containing JSON files for each resource type, so that I can back up or migrate data.
41. As a super admin, I want export options to include/exclude API keys and audit log, and to optionally inline `$ref_id` references, so that I control the export scope.
42. As a super admin, I want to import a ZIP archive in a single DB transaction with one cache invalidation at the end, so that imports are atomic and don't trigger N reloads.
43. As a super admin, I want the import to process globals first (with reference replacement), then core tables, so that `$ref_id` references remain valid throughout.
44. As a super admin, I want import conflicts to be reported at per-attribute granularity with per-resource and per-attribute resolution strategies (keep_old, keep_new, per_attribute), so that I have full control over conflict resolution.
45. As a super admin, I want name collisions during import to prompt "import into existing" vs. "create new client," so that I can merge or separate data.
46. As a super admin, I want import to match resources by UUID (not by name), so that the same resources from multiple exports don't duplicate.

### Batch Operations

47. As an admin, I want to batch-delete blueprints by ID list, so that I can clean up in bulk.
48. As an admin, I want to batch-assign affixes to blueprints (add N affixes to M blueprints' pools with a default weight), so that I can configure affix availability efficiently.
49. As an admin, I want to batch-edit attributes across multiple blueprints (only blueprints that already have the attribute key are updated; type mismatches return 422), so that I can update common properties in bulk.
50. As an admin, I want batch-assignment from the affix side too — assign N affixes to M blueprints — so that the operation is symmetric.

### Schema Introspection

51. As a tool developer (Admin UI / CLI generator), I want `GET /api/schema/blueprints`, `GET /api/schema/affixes`, and `GET /api/schema/generate` to return JSON Schema documents describing the request/response structures, so that I can auto-generate API clients.

### System / Bootstrap

52. As a devops engineer, I want Arche to auto-generate a super admin API key on first startup and print it to stdout, so that I don't need manual configuration steps for the initial admin.
53. As a devops engineer, I want Service to support single-instance mode (no Redis, cache polling) and cluster mode (Redis pub/sub for cache invalidation), so that I can choose the deployment model that fits my scale.
54. As a devops engineer, I want the service to expose a health check endpoint, so that load balancers and orchestrators can verify readiness.
55. As a devops engineer, I want all responses to include problem details in RFC 9457 format for errors, so that error handling is consistent across all clients.

## Implementation Decisions

### Crate Structure (Rust workspace)

The Rust workspace has three members:

```
arche-service/         — the HTTP server binary
  src/
    main.rs
    api/               — route handlers (axum)
    generation/        — generation engine (blueprint selection, attribute rolling, affix selection)
    db/                — PostgreSQL queries (sqlx)
    cache/             — in-memory cache, polling/pubsub invalidation
    auth/              — API key verification, permission checks
    audit/             — audit log writing
    export_import/     — ZIP-based export/import logic
    schema/            — JSON Schema generation
arche-types/           — shared types: domain structs, enums, validation, API contracts
arche-cli/             — CLI binary (separate PRD)
```

### Core data model

Defined in `arche-types` and mirrored in the PostgreSQL schema (see `docs/design/data-model.md`):

- **Client** — id (UUID), name, created_at, updated_at
- **ApiKey** — id (UUID), client_id (nullable for super admin), name, key_hash (bcrypt), permissions (TEXT[]), is_super, created_at, expires_at (optional)
- **GlobalMetaAttribute** — id (UUID), client_id, name, description, value_type (enum: single/enum/range/string/boolean), payload (JSONB)
- **Blueprint** — id (UUID), client_id, name, archetype, weight, description, attributes (JSONB), attribute_order (TEXT[]), min/max prefix/suffix counts
- **BlueprintAffix** — id (UUID), blueprint_id, affix_id, weight, location (prefix/suffix), sort_order
- **Affix** — id (UUID), client_id, name, type (prefix/suffix), description, attribute (JSONB)
- **AuditLogEntry** — id (UUID), timestamp, actor_key_id, actor_key_name, client_id, resource_type, resource_id, action, before (JSONB), after (JSONB)

### Generation Engine

The generation engine is the core deep module. It operates entirely in memory on preloaded data:

1. **Blueprint selection:** Filter blueprints by archetype (if specified), then by constraint overlap semantics (see ADR 0002). Select one blueprint via weighted random from the matching set.

2. **Attribute resolution:** For each attribute key on the selected blueprint, resolve `$ref_id` if present (load from global pool). Inline definitions override global ones of the same key.

3. **Attribute rolling:** For range values, roll a value according to the configured distribution:
   - Uniform: random value between min and max
   - Normal: clamp Box-Muller transform output to [min, max]
   - Exponential: sample from exponential distribution with given rate, clamp to [min, max]
   For enum values, select one value uniformly from the values array. For single/string/boolean, return the stated value.

4. **Affix selection:** From the blueprint's affix pool, separate into prefixes and suffixes. Roll affix count per type using the blueprint's distribution config (default: uniform between min and max). Select N affixes via weighted random without replacement. Blocked affixes are excluded; required affixes are pre-selected (counted against the roll).

5. **Name composition:** Concatenate selected affix names with the blueprint's base name: `[prefix1] [prefix2] [base_name] [suffix1] [suffix2]`.

6. **Output assembly:** Blueprint attributes ordered by `attribute_order`, then affix attributes ordered by `sort_order` on the `blueprint_affixes` join row.

### Attribute value type payloads

The payload JSONB schema for each value type (validated at write time, used at generation time):

```
single:  { "value": <float> }
enum:    { "values": [<string>, ...] }
range:   { "min": <float>, "max": <float>, "distribution": { "type": "uniform" | "normal", "mean": <float>, "std_dev": <float> } | { "type": "exponential", "rate": <float> } }
string:  { "value": <string> }   (value is optional — may be null for user-filled)
boolean: {}                       (no payload needed — value is always true when present)
```

Validation rules:
- `range.min` must be <= `range.max`
- `range.distribution.std_dev` must be > 0
- `range.distribution.rate` must be > 0
- `enum.values` must have at least 1 entry
- `single.value` must not be null

### In-Memory Cache

Each instance loads all definitions into memory at startup:

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

Cache invalidation:
- **Single mode:** A background task polls `SELECT max(updated_at) FROM ...` every 3–5 seconds. If changed, reload that client's data.
- **Cluster mode:** On any CRUD write, publish `{ "type": "invalidate", "client_id": "..." }` to Redis pub/sub. All instances listen and reload.
- **Force refresh:** The `X-Cache-Refresh: true` header causes a synchronous DB load for that request only, without updating the shared cache.

### API Layer (axum)

The API is built with `axum` and organized by resource:

- Route handlers are thin: parse request → call service layer → format response.
- All handlers accept an `AuthenticatedKey` extractor (see Auth).
- Pagination uses cursor-based (`?cursor=<uuid>&limit=50`) for API and offset-based (`?page=1&per_page=50`) for Admin UI convenience.
- Errors return RFC 9457 Problem JSON via a custom `axum::response::IntoResponse` impl.

### Authentication & Authorization

- A custom axum extractor `AuthenticatedKey` reads `X-API-Key` header, looks up the key hash, resolves permissions and client scope.
- Super admin keys have `client_id = None` and can access any endpoint across all clients.
- Per-client RBAC keys are checked against a permission matrix per endpoint:

| Endpoint | Required Permission |
|---|---|
| `GET /api/blueprints` (etc.) | `read` |
| `POST /api/blueprints` (etc.) | `write` |
| `PUT /api/blueprints/:id` | `write` |
| `DELETE /api/blueprints/:id` | `delete` |
| `POST /api/generate` | `generate` |
| `POST /api/clients/:id/keys` | `admin` |
| `GET /api/audit-log` | super admin only |

### Export / Import

Export:
- Query all requested clients' data.
- Build in-memory folder-per-client structure with JSON files.
- Optionally inline `$ref_id` references (resolve to inline definitions).
- Stream as ZIP via `async-zip` or similar.

Import:
- Parse ZIP archive, validate structure.
- Process globals first (resolve references as inline definitions for later matching).
- Process blueprints, affixes, blueprint_affixes.
- Detect conflicts by matching UUIDs. Report per-attribute differences.
- If no conflicts, execute in a single DB transaction, fire one cache invalidation at the end.
- If conflicts, return 409 with conflict details and an import token.
- `POST /api/import/resolve` accepts the token + resolutions, applies them, commits.

### Audit Log

- Written synchronously in the same DB transaction as the mutation (to guarantee at-most-once logging).
- Generate operations are NOT logged.
- Cascade adjustments (force-delete affix adjusting min/max counts) are logged with action `adjusted`.
- The `before` snapshot is the resource state before the mutation; `after` is the state after.

### Batch Operations

- Batch delete: `POST /api/blueprints/batch/delete` takes `{ "ids": [...] }`, deletes each in a loop (with individual reference checks), returns count.
- Batch assign: `POST /api/blueprints/batch/assign` takes `{ "blueprint_ids": [...], "affix_ids": [...], "weight": 1.0 }`, creates `blueprint_affixes` rows for each pair. Duplicates silently skipped. Symmetric endpoint from the affix side.
- Batch edit: `POST /api/blueprints/batch/edit` takes `{ "blueprint_ids": [...], "attributes": { "key": { ... } } }`. Only blueprints that already have the attribute key are updated. Type mismatch returns 422. Attributes not present on a blueprint are silently skipped.

### Schema Endpoints

- `/api/schema/blueprints`, `/api/schema/affixes`, `/api/schema/generate` return static (or lazily-generated) JSON Schema documents derived from the Rust types via `schemars` crate.
- These are stable across all clients and versions (backward-compatible changes only).

### Error Format (RFC 9457)

```json
{
  "type": "/errors/validation-error",
  "title": "Validation Error",
  "status": 400,
  "detail": "attribute 'damage': range min (25.0) exceeds max (10.0)",
  "instance": null,
  "errors": [
    { "path": "attributes.damage", "message": "min must be <= max" }
  ]
}
```

The `errors` array is optional and provides per-field error details. The `type` URI is relative to the API base (not a real document).

## Testing Decisions

### Testing Philosophy

- **Generation engine:** The core deep module. Test with deterministic seeds, assert exact output values. Property-based testing (`proptest`) for distribution sampling (assert statistical properties of many rolls).
- **API handlers:** Integration tests against a real or in-memory PostgreSQL database. Use `axum::test` helpers. Mock the cache layer and auth layer to focus on handler logic.
- **Cache layer:** Test invalidation logic with a mock notifier. Verify that stale data is not served after invalidation events.
- **Auth:** Unit-test key hashing/verification, permission matrix lookups, and edge cases (expired keys, revoked keys, malformed headers).
- **Export/import:** Integration tests with real ZIP files. Test round-trip: export → modify → import → verify DB state. Test conflict detection with known conflict scenarios. Test atomicity: inject a failure mid-import and assert no partial state.
- **Audit log:** Test that mutations produce correct before/after snapshots. Test that generate operations do not produce audit entries.

### Modules to Test

| Module | Test Type | What to Test |
|--------|-----------|--------------|
| **Generation engine** | Unit + property | Blueprint selection (weighted, constraints), attribute rolling (all distributions, clamping), affix selection (without replacement, require/block), name composition |
| **API handlers** | Integration | Each endpoint: success, validation error, auth error, not found, conflict |
| **Cache** | Unit + integration | Load, invalidation (poll + pubsub), force refresh, concurrent access |
| **Auth** | Unit | Key hashing/verification, permission checks, super admin vs. RBAC, expiry |
| **Audit log** | Integration | Mutation produces correct entry, generate does not, cascade adjustments logged |
| **Export/Import** | Integration | Round-trip, conflict detection, atomicity, per-attribute resolution |
| **Validation** | Unit | All value type payload rules, attribute_order completeness, distribution config ranges |
| **Batch operations** | Integration | Delete, assign, edit — success and error cases |

### Prior Art

No existing tests in this codebase. Rust test conventions:
- Unit tests: `#[cfg(test)] mod tests { ... }` co-located with the source
- Integration tests: `tests/` directory at crate root (one file per module: `api.rs`, `generation.rs`, `export_import.rs`)
- Database tests: use `sqlx::test` with testcontainers or a shared test database (`test_postgres` feature)
- Property-based: `proptest` for generation engine statistical assertions
- Mocking: `tower-test` for axum handler tests, `mockall` for trait mocking (cache, DB)

## Out of Scope

- **WebSocket / real-time updates** — no streaming endpoints or push notifications. The Admin UI polls or uses cache-refresh headers for freshness.
- **User accounts / SSO / OAuth** — the API-key-only model is kept (see ADR 0001).
- **Rate limiting** — not implemented in this iteration. Left to the reverse proxy / API gateway layer.
- **Metrics / OpenTelemetry** — the service does not export metrics endpoints in the initial version. Add in a follow-up.
- **Database migrations tool** — the schema is managed by the application (sqlx migrate) but the migration files are out of scope for this PRD. They are a prerequisite to the service implementation.
- **Horizontal sharding** — all data lives in a single PostgreSQL database. No read replicas or sharding.
- **Generation preview history** — storing recently generated things for the Admin UI dashboard is deferred.
- **CLI tool** — the `arche` CLI is a separate deliverable (separate PRD).
- **Admin UI** — the web interface is a separate deliverable (separate PRD).

## Further Notes

- The service uses `sqlx` for PostgreSQL access (compile-time checked queries via `sqlx::query!`). Database migrations are managed via `sqlx migrate`.
- `axum` is chosen for its ergonomic extractor system, tower middleware integration, and strong async ecosystem (tokio).
- The service listens on a configurable port (default 8080). Config is via environment variables (e.g., `ARCHE_DATABASE_URL`, `ARCHE_REDIS_URL`, `ARCHE_PORT`, `ARCHE_CACHE_POLL_INTERVAL_MS`).
- In cluster mode, Redis connection is established at startup and the pub/sub listener runs in a background tokio task. If Redis is unavailable, the service falls back to polling mode and logs a warning.
- The bootstrap mechanism: on first startup, the service checks if any super admin key exists in the `api_keys` table. If not, it generates one, inserts it (bcrypt-hashed), and prints the raw key to stdout. This key is returned only at startup — there is no API endpoint to retrieve it later.
- JSON Schema generation via `schemars` is done at compile time (build script) or lazily at first request. The schemas are static and do not vary by client.
