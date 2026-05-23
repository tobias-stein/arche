# Glossary

## Core terms

- **Thing** — A generated entity (e.g. an item, creature) produced by the service. Instantiated from a blueprint with concrete attribute values and affixes.
- **Blueprint** — A genotype/template defining a specific variant of a thing. Has an archetype (a string label like "sword", "orc", "ring"), unique base name, weight, meta attributes (with concrete values or ranges), and an affix configuration. The affix count is rolled with a configurable distribution (default: uniform). Example: `{ "archetype": "sword", "name": "Longsword of the Ancients" }`.
- **Meta Attribute** — A named property with a value type. Five value types: single value (float), enum value, range value (min-max float), string constant, boolean flag. Can be defined globally (shared pool) or inline on a blueprint/affix. When both a global and inline definition share the same key name on a blueprint, the inline definition wins.
- **Global Meta Attribute** — A named attribute definition in the shared client pool. Blueprints and affixes reference it via `$ref_id` to reuse the definition. Global updates propagate automatically at resolution time (generation).
- **Affix** — A modifier that can be attached to a generated thing. Carries exactly one meta attribute with a concrete value. Divided into prefixes and suffixes. Each affix can appear at most once per thing. Selected via weighted random without replacement.
- **Rarity** — A user-defined meta attribute (enum type). No special meaning in the generation engine — treated identically to any other attribute.
- **Client** — A named project (game, simulation, or any consuming system) that owns a set of definitions. Enables multi-tenant isolation. Every resource is scoped to a client.

## Architecture

- **Database** — PostgreSQL as source of truth. Only hit on CRUD changes and auth lookups, not on the generation hot path.
- **Cache** — Each arche instance loads all definitions into memory at startup.
  - **Single-instance mode:** No Redis. Cache invalidation via direct DB notification or polling.
  - **Cluster mode:** Redis pub/sub for cross-instance cache invalidation. Redis is NOT used for data lookups on the hot path.
- **Service (API)** — Rust. Named **Arche** (from "archetype"). Handles generation (CPU-bound, in-memory), CRUD operations, and data consistency enforcement.
- **Web UI** — React + shadcn/ui. Modern, intuitive browser-based admin interface.
- **CLI** — Rust. Named `arche`. Shares types with the service. For automation and scripting.
- **Bootstrapping:** On first startup, arche auto-generates a super admin API key and prints it to stdout.
- **Deployment** — Configurable: single-instance (no Redis) or cluster (multiple instances + Redis). Same codebase.
- **Authentication:** Static API key via header.
  - **Super admin key:** Auto-generated at first startup, printed to stdout. Can create/delete clients and access any client's data.
  - **Client RBAC keys:** Multiple per client, scoped to a single client. Permissions: `read`, `write`, `delete`, `generate`, `admin` (manage keys).
- **Audit log:** PostgreSQL table recording all CRUD operations. Stores before/after JSON snapshots, actor (API key ID), resource type, resource ID, action, and timestamp. Generate operations are NOT logged. Auto-adjustments (e.g., affix cascade) are logged with action `adjusted`.
- **Attribute order:** Blueprint attributes are stored as a JSONB map (keyed by name) with a separate `TEXT[]` column listing display order. Uniqueness and completeness are enforced at write time.
- **API versioning:** None. Backward-compatible changes only (add fields, never remove).
- **Naming convention:** Resource names use `[a-zA-Z0-9_]` only. The `$` prefix is reserved for system keys.
- **Export/import:** File-based JSON export (ZIP), folder-per-client structure. Core tables always exported; API keys and audit log are opt-in toggles. Export offers an "inline global references" option (resolves `$ref_id` to inline values in the export). Import is phased: globals first (with reference replacement), then core tables. Entire import runs in a single DB transaction with a single cache invalidation at the end. Conflict resolution at per-attribute granularity with batch "keep old" / "keep new" escape hatch; name collisions prompt "import into existing" vs. "create new client". Matching uses UUIDs, never names.
- **Import/export in CLI:** Same format and conflict resolution model as the UI. Two modes: interactive (terminal prompts per conflict) and file-based (`arche import --dry-run` generates a conflicts JSON, user edits it, feeds back with `arche import --resolve-file`).
- **Cache invalidation — single mode:** Poll `updated_at` max across all tables every 3-5 seconds. Endpoint supports `X-Cache-Refresh: true` header to force a fresh DB load for that single request.
- **Batch import transaction:** Entire import runs in a single DB transaction, fires one cache invalidation at the end. Avoids N reloads for N imported resources.
- **Admin UI:** React + shadcn/ui. Responsive (mobile, tablet, desktop). Dark/light mode toggle. Global search (Cmd+K, grouped results by type). Per-client scoped to selected client.
- **Navigation Drawer** — Toggleable left-side drawer. Houses global actions: client switcher, API key management, login/logout, settings.
- **Activity Panel** — Toggleable right-side panel (default: floating overlay that hides on outside click; optional: docked/pinned to the viewport). Shows activity logs, change history, or audit log entries.
- **Warnings** — Misconfiguration states surfaced on the dashboard: blueprint min prefix/suffix > 0 with empty pool, zero-weight blueprint, zero-weight affix assignment, dangling `$ref_id`, invalid attribute payload (range min > max, empty enum), invalid distribution config.
- **UI pattern — Resource views:** Each entity (blueprints, affixes, global meta attributes, clients) has a list page (table), a detail page (read-only, shareable URL with tabs), and create/edit modals (overlays opened from any page).

## Generation rules

- **Constraint matching** uses overlap semantics. A blueprint matches a constraint if its attribute range overlaps with the constraint range. Example: blueprint `damage: [10, 23]` matches constraint `damage >= 15` because the ranges overlap.
- **Constraint syntax:** Explicit operators for all value types. `{ "gte": 15 }`, `{ "in": ["rare", "legendary"] }`, `{ "contains": "sword" }`, exact values for booleans. Simple values (e.g., `"legendary"`) are NOT supported — always use an operator object.
- **Attribute resolution:** At generation time, each attribute is resolved. If it has a `$ref_id`, the global definition is loaded. If the same key also exists as an inline definition, the inline definition wins. Global updates are automatically reflected since resolution happens at generation time.
- **Constraint matching:** Only blueprints that explicitly define the constrained attribute (inline or `$ref_id`) can match. A blueprint without a `damage` attribute won't match a `damage: { gte: 15 }` constraint.
- **Affix min/max are preferences, not requirements.** Generation ignores them if the pool is insufficient. However, if a client explicitly constrains an affix count in the request, matching blueprints must be able to satisfy it.
- **Affix constraints in generate request:** `min_prefixes`, `max_prefixes`, `min_suffixes`, `max_suffixes` become hard requirements. `require: [ids]` — only blueprints that include all required affixes in their pool match. `block: [ids]` — blueprints that include any blocked affix are excluded.
- **Range values** are rolled as floats. Default distribution is uniform; normal (mean, std_dev) and exponential (rate) are also available. Distribution is specified inline per attribute.
- **Affix count** is rolled with a configurable distribution (default: uniform). Affixes are selected via weighted random without replacement — each affix appears at most once per thing.
- **Name composition** returns both a composed full name (prefix + base name + suffix) and the individual components.
- **Output structure** separates blueprint attributes and affix attributes. Blueprint attributes first (in `attribute_order` order), then affix attributes (in `sort_order` order). The service does not combine or interpret attribute values — that is the consuming game's responsibility.
- **Seed:** A `u64` integer. Clients may pass it in the request; if omitted, the service generates one. The seed is always returned in the response.
- **Pagination:** Cursor-based for the API, offset-based for the admin UI. Default limit: 50.
- **Error format:** RFC 9457 (Problem JSON).

## Consistency

- **Default delete:** Rejected if the resource is currently referenced. The frontend must guide the user to resolve references first.
- **Force delete (affix):** Cascading delete — removes the affix from all blueprints that reference it. Auto-adjusts min/max affix counts to match remaining pool size. Logs the adjustment in the audit log with action `adjusted`.
- **Force delete (global meta attribute):** Cascading delete — removes the `$ref_id` from all blueprints and affixes that reference it.
- **Force delete (blueprint):** Deletes the blueprint and its affix references.
- **Cache invalidation:** On any CRUD change that affects generation data, the service broadcasts an invalidation event. In cluster mode via Redis pub/sub, in single mode as a local event.

## Tenancy

- The service is multi-tenant. Every definition (blueprint, affix, global meta attribute) is scoped to a **client**.
- The API key identifies the client. The key is passed in a header.
- Isolation is at the data level — clients cannot see or reference each other's definitions.

## Relationships

- A **Client** has many **Blueprints**, **Affixes**, and **Global Meta Attributes**.
- A **Blueprint** belongs to one **Client**, has an **archetype** (string label), has many **Meta Attributes** (inline or `$ref_id` to global), and defines which **Affixes** it can roll (with weights and min/max counts for prefixes and suffixes).
- An **Affix** belongs to one **Client** and has exactly one **Meta Attribute** (inline or `$ref_id` to global).
- A **Global Meta Attribute** belongs to one **Client** and can be referenced by many **Blueprints** and **Affixes**.
- A generated **Thing** is instantiated from one **Blueprint**, with rolled attribute values and selected affixes.
