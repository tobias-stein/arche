# Arche Data Model — PostgreSQL Schema Design

## Overview

Six core tables + one audit table. All relationships are foreign-key enforced. UUIDs everywhere.

---

## Tables

### `clients`
```sql
CREATE TABLE clients (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

### `api_keys`

Super admin key is stored with `client_id = NULL` and `is_super = true`.
```sql
CREATE TABLE api_keys (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id    UUID REFERENCES clients(id) ON DELETE CASCADE,  -- NULL for super admin
    name         TEXT NOT NULL,
    key_hash     TEXT NOT NULL,                -- bcrypt hash of the raw key
    permissions  TEXT[] NOT NULL,              -- e.g. {read, write, delete, generate, admin}
    is_super     BOOLEAN NOT NULL DEFAULT false,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at   TIMESTAMPTZ                   -- optional key expiry
);

CREATE INDEX idx_api_keys_client ON api_keys(client_id);
CREATE UNIQUE INDEX idx_api_keys_hash ON api_keys(key_hash);
```

### `global_meta_attributes`
```sql
CREATE TYPE value_type AS ENUM ('single', 'enum', 'range', 'string', 'boolean');

CREATE TABLE global_meta_attributes (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id    UUID NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    name         TEXT NOT NULL,
    description  TEXT,
    value_type   value_type NOT NULL,

    -- Type-specific payload (JSONB for flexibility; validated by application)
    -- single:  { "default": 150.0 }
    -- enum:    { "values": ["common", "uncommon", "rare", "legendary"] }
    -- range:   { "min": 0.0, "max": 100.0 }
    -- string:  { "default": "some text" }   (or null for no default)
    -- boolean: {}  (no extra payload)
    payload      JSONB NOT NULL DEFAULT '{}',

    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT now(),

    UNIQUE(client_id, name)
);

CREATE INDEX idx_gma_client ON global_meta_attributes(client_id);
```

### `blueprints`
```sql
CREATE TABLE blueprints (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id    UUID NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    name         TEXT NOT NULL,
    archetype    TEXT NOT NULL,               -- string label: "sword", "orc", etc.
    weight       DOUBLE PRECISION NOT NULL DEFAULT 1.0,
    description  TEXT,

    -- Inline meta attributes keyed by name. Each entry is either:
    --   { "value_type": "...", ... }           — inline definition
    --   { "$ref_id": "uuid-of-global-attr" }   — reference to global
    attributes      JSONB NOT NULL DEFAULT '{}',
    attribute_order TEXT[] NOT NULL DEFAULT '{}',   -- display order of attribute keys

    -- Affix configuration
    min_prefixes INTEGER NOT NULL DEFAULT 0,
    max_prefixes INTEGER NOT NULL DEFAULT 0,
    min_suffixes INTEGER NOT NULL DEFAULT 0,
    max_suffixes INTEGER NOT NULL DEFAULT 0,

    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT now(),

    UNIQUE(client_id, name)
);

CREATE INDEX idx_blueprints_client ON blueprints(client_id);
CREATE INDEX idx_blueprints_archetype ON blueprints(client_id, archetype);
CREATE INDEX idx_blueprints_gin ON blueprints USING GIN (attributes);
```

### `blueprint_affixes`
```sql
CREATE TYPE affix_location AS ENUM ('prefix', 'suffix');

CREATE TABLE blueprint_affixes (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    blueprint_id UUID NOT NULL REFERENCES blueprints(id) ON DELETE CASCADE,
    affix_id     UUID NOT NULL REFERENCES affixes(id) ON DELETE CASCADE,
    weight       DOUBLE PRECISION NOT NULL DEFAULT 1.0,
    location     affix_location NOT NULL,     -- denormalized from affix.type
    sort_order   INTEGER NOT NULL DEFAULT 0,

    UNIQUE(blueprint_id, affix_id)
);

CREATE INDEX idx_ba_blueprint ON blueprint_affixes(blueprint_id);
CREATE INDEX idx_ba_affix ON blueprint_affixes(affix_id);
```

### `affixes`
```sql
CREATE TABLE affixes (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id    UUID NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    name         TEXT NOT NULL,
    type         affix_location NOT NULL,     -- prefix or suffix
    description  TEXT,

    -- Exactly one meta attribute (inline or $ref_id)
    -- Same structure as blueprint.attributes entries
    attribute    JSONB NOT NULL,

    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT now(),

    UNIQUE(client_id, name)
);

CREATE INDEX idx_affixes_client ON affixes(client_id);
```

### `audit_log`
```sql
CREATE TYPE audit_action AS ENUM ('created', 'updated', 'deleted', 'force_deleted', 'adjusted');

CREATE TABLE audit_log (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    timestamp       TIMESTAMPTZ NOT NULL DEFAULT now(),
    actor_key_id    UUID NOT NULL REFERENCES api_keys(id),
    actor_key_name  TEXT NOT NULL,            -- denormalized for display
    client_id       UUID REFERENCES clients(id),
    resource_type   TEXT NOT NULL,            -- "blueprint", "affix", "global_meta_attribute", "client", "api_key"
    resource_id     UUID NOT NULL,
    action          audit_action NOT NULL,
    before          JSONB,                   -- NULL for created
    after           JSONB                    -- NULL for deleted
);

CREATE INDEX idx_audit_timestamp ON audit_log(timestamp DESC);
CREATE INDEX idx_audit_client ON audit_log(client_id);
CREATE INDEX idx_audit_resource ON audit_log(resource_type, resource_id);
CREATE INDEX idx_audit_actor ON audit_log(actor_key_id);
```

---

## Relationships Diagram

```
clients
  ├── api_keys (0..N, client_id NULL for super admin)
  ├── global_meta_attributes (0..N)
  ├── blueprints (0..N)
  │     └── blueprint_affixes (0..N)
  │           └── affixes (1)
  └── affixes (0..N)
```

## Cache Loading

On startup (and on invalidation), each arche instance loads into memory:

1. All `global_meta_attributes` (per client, keyed by ID)
2. All `blueprints` + their `blueprint_affixes` (per client, with resolved attributes)
3. All `affixes` (per client)

Cache invalidation strategies:
- **Single mode:** Poll `updated_at` max across all tables every N seconds
- **Cluster mode:** On any CRUD write, publish `{ "type": "invalidate", "client_id": "..." }` to Redis pub/sub. All instances reload that client's data.

## Notes

- **No deletion of clients that have active keys.** Check `api_keys` table before allowing client deletion.
- **Cascading deletes are handled at the application level** (consistency rules), not at the DB level. The DB uses `ON DELETE CASCADE` for structural cleanup (e.g., deleting a blueprint removes its blueprint_affixes).
- **JSONB for attributes** gives schema flexibility while keeping referential integrity via `$ref_id`.
- **bcrypt for key hashing.** Raw key is returned exactly once on creation.
- **Affix force delete cascade:** Application auto-adjusts `min_prefixes`/`max_prefixes` / `min_suffixes`/`max_suffixes` if the remaining pool cannot satisfy the configured counts. Only downsizes max first, then min. Logged as `adjusted` in audit log.
- **Import/export format:** ZIP archive. Each client gets a folder named by client ID. Inside: `blueprints.json`, `affixes.json`, `global-meta-attributes.json`, `api-keys.json` (opt-in), `audit-log.json` (opt-in). Export may inline `$ref_id` to resolved attribute values.
- **Batch import transactions:** Import runs in a single DB transaction. A single cache invalidation fires at the end, not per-resource.
