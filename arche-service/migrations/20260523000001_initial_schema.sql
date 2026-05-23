CREATE TYPE value_type AS ENUM ('single', 'enum', 'range', 'string', 'boolean');

CREATE TYPE affix_location AS ENUM ('prefix', 'suffix');

CREATE TYPE audit_action AS ENUM ('created', 'updated', 'deleted', 'force_deleted', 'adjusted');

CREATE TABLE clients (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE api_keys (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id    UUID REFERENCES clients(id) ON DELETE CASCADE,
    name         TEXT NOT NULL,
    key_hash     TEXT NOT NULL,
    permissions  TEXT[] NOT NULL,
    is_super     BOOLEAN NOT NULL DEFAULT false,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at   TIMESTAMPTZ
);

CREATE INDEX idx_api_keys_client ON api_keys(client_id);
CREATE UNIQUE INDEX idx_api_keys_hash ON api_keys(key_hash);

CREATE TABLE global_meta_attributes (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id    UUID NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    name         TEXT NOT NULL,
    description  TEXT,
    value_type   value_type NOT NULL,
    payload      JSONB NOT NULL DEFAULT '{}',
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(client_id, name)
);

CREATE INDEX idx_gma_client ON global_meta_attributes(client_id);

CREATE TABLE blueprints (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id      UUID NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    name           TEXT NOT NULL,
    archetype      TEXT NOT NULL,
    weight         DOUBLE PRECISION NOT NULL DEFAULT 1.0,
    description    TEXT,
    attributes     JSONB NOT NULL DEFAULT '{}',
    attribute_order TEXT[] NOT NULL DEFAULT '{}',
    min_prefixes   INTEGER NOT NULL DEFAULT 0,
    max_prefixes   INTEGER NOT NULL DEFAULT 0,
    min_suffixes   INTEGER NOT NULL DEFAULT 0,
    max_suffixes   INTEGER NOT NULL DEFAULT 0,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(client_id, name)
);

CREATE INDEX idx_blueprints_client ON blueprints(client_id);
CREATE INDEX idx_blueprints_archetype ON blueprints(client_id, archetype);
CREATE INDEX idx_blueprints_gin ON blueprints USING GIN (attributes);

CREATE TABLE affixes (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id    UUID NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    name         TEXT NOT NULL,
    type         affix_location NOT NULL,
    description  TEXT,
    attribute    JSONB NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(client_id, name)
);

CREATE INDEX idx_affixes_client ON affixes(client_id);

CREATE TABLE blueprint_affixes (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    blueprint_id UUID NOT NULL REFERENCES blueprints(id) ON DELETE CASCADE,
    affix_id     UUID NOT NULL REFERENCES affixes(id) ON DELETE CASCADE,
    weight       DOUBLE PRECISION NOT NULL DEFAULT 1.0,
    location     affix_location NOT NULL,
    sort_order   INTEGER NOT NULL DEFAULT 0,
    UNIQUE(blueprint_id, affix_id)
);

CREATE INDEX idx_ba_blueprint ON blueprint_affixes(blueprint_id);
CREATE INDEX idx_ba_affix ON blueprint_affixes(affix_id);

CREATE TABLE audit_log (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    timestamp       TIMESTAMPTZ NOT NULL DEFAULT now(),
    actor_key_id    UUID NOT NULL REFERENCES api_keys(id),
    actor_key_name  TEXT NOT NULL,
    client_id       UUID REFERENCES clients(id),
    resource_type   TEXT NOT NULL,
    resource_id     UUID NOT NULL,
    action          audit_action NOT NULL,
    before          JSONB,
    after           JSONB
);

CREATE INDEX idx_audit_timestamp ON audit_log(timestamp DESC);
CREATE INDEX idx_audit_client ON audit_log(client_id);
CREATE INDEX idx_audit_resource ON audit_log(resource_type, resource_id);
CREATE INDEX idx_audit_actor ON audit_log(actor_key_id);

-- migrate:rollback

DROP TABLE IF EXISTS audit_log;

DROP TABLE IF EXISTS blueprint_affixes;

DROP TABLE IF EXISTS affixes;

DROP TABLE IF EXISTS blueprints;

DROP TABLE IF EXISTS global_meta_attributes;

DROP TABLE IF EXISTS api_keys;

DROP TABLE IF EXISTS clients;

DROP TYPE IF EXISTS audit_action;

DROP TYPE IF EXISTS affix_location;

DROP TYPE IF EXISTS value_type;
