---
title: PostgreSQL schema + migrations
status: completed
---

## Parent

arche-types shared library (see `.scratch/arche-types/prd.md`)

## What to build

Create the initial `sqlx` migration files that define all six core tables plus the audit log table. Use the data model from `docs/design/data-model.md`.

Includes:
- `clients` table (id UUID, name TEXT, created_at/updated_at TIMESTAMPTZ)
- `api_keys` table (id UUID, client_id UUID nullable, name, key_hash TEXT, permissions TEXT[], is_super BOOLEAN, created_at, expires_at)
- `global_meta_attributes` table with `value_type` ENUM ('single', 'enum', 'range', 'string', 'boolean')
- `blueprints` table with JSONB `attributes`, TEXT[] `attribute_order`, affix min/max counts
- `blueprint_affixes` join table with weight, location ENUM, sort_order
- `affixes` table with JSONB `attribute`, type ENUM
- `audit_log` table with JSONB `before`/`after`, `audit_action` ENUM
- All indexes: foreign keys, GIN on blueprints.attributes, audit log timestamps

## Acceptance criteria

- [ ] Migration files in `arche-service/migrations/` directory
- [ ] `sqlx migrate run` applies cleanly to a fresh PostgreSQL database
- [ ] All tables, enums, indexes, and foreign key constraints exist after migration
- [ ] `sqlx migrate revert` rolls back cleanly
- [ ] Data model doc (`docs/design/data-model.md`) matches migration output

## Blocked by

None - can start immediately
