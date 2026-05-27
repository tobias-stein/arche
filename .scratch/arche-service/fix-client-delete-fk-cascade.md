---
title: Fix client delete FK cascade on audit_log
status: completed
---

## What to build

Deleting a client fails with a foreign key violation because `audit_log.client_id` and `audit_log.actor_key_id` reference `clients(id)` and `api_keys(id)` without `ON DELETE CASCADE`. All other child tables (`api_keys`, `global_meta_attributes`, `blueprints`, `affixes`) use `ON DELETE CASCADE`.

Add a migration that drops and re-creates the `audit_log` foreign key constraints with `ON DELETE CASCADE`:
- `audit_log.client_id REFERENCES clients(id) ON DELETE CASCADE`
- `audit_log.actor_key_id REFERENCES api_keys(id) ON DELETE CASCADE`

## Acceptance criteria

- [ ] New migration adds `ON DELETE CASCADE` to `audit_log.client_id` FK
- [ ] New migration adds `ON DELETE CASCADE` to `audit_log.actor_key_id` FK
- [ ] Deleting a client no longer fails when audit log entries reference it
- [ ] All existing tests pass

## Blocked by

None — can start immediately.
