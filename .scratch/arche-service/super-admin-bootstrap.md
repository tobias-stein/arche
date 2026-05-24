---
title: Bootstrap — super admin key auto-generation
status: completed
---

## Parent

Arche API Service PRD (see `.scratch/arche-service/prd.md`)

## What to build

On first startup, the service checks if any super admin key exists in the `api_keys` table. If not, it generates a random API key (prefixed `arche_k_`), bcrypt-hashes it, inserts into `api_keys` with `is_super = true` and `client_id = NULL`, and prints the raw key to stdout. This key is returned only at startup — there is no API endpoint to retrieve it later.

If a super admin key already exists, the bootstrap phase is skipped silently.

The raw key output is clearly labeled and easy to spot in logs (e.g., a banner with `=== SUPER ADMIN API KEY ===`).

## Acceptance criteria

- [ ] Fresh database: key is generated, inserted (bcrypt-hashed), and printed to stdout
- [ ] Key format: starts with `arche_k_`
- [ ] Second startup: no new key generated, no duplicate insertion
- [ ] Key is retrievable only at startup — no API endpoint returns the raw key
- [ ] Integration test: wipe DB, start server, capture key from stdout, verify it authenticates
- [ ] `cargo test` passes

## Blocked by

- arche-types/database-migrations.md
- arche-types/domain-structs-enums.md
