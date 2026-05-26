---
title: Import — parse & conflict detection
status: completed
---

## Parent

Arche API Service PRD (see `.scratch/arche-service/prd.md`)

## What to build

Implement `POST /api/import` for importing a ZIP archive. Super admin only.

Request: Binary ZIP upload (`multipart/form-data`).

Steps:
1. Parse and validate ZIP structure (folder-per-client, expected JSON files)
2. Process globals first — create/update global meta attributes, resolve `$ref_id` references as inline for later matching
3. Process blueprints, affixes, blueprint_affixes — match resources by UUID (not by name)
4. Detect conflicts at per-attribute granularity: compare each existing resource's fields with the import version
5. For client name collisions, report as a conflict requiring "import into existing" vs "create new client" decision

If no conflicts: execute in a single DB transaction, return `{ "status": "imported", "clients_created": N, "resources_imported": N }`.

If conflicts detected: return 409 with conflict details and an import token. The conflict report lists each conflicting resource (type, id, name) with per-attribute diffs.

## Acceptance criteria

- [ ] Valid ZIP with no conflicts: imported successfully, resources created
- [ ] Conflict detected: returns 409 with per-attribute diff and import token
- [ ] Client name collision: reported as conflict
- [ ] Globals processed first (so `$ref_id` references resolve for blueprints/affixes)
- [ ] Matching by UUID, never by name
- [ ] Single transaction on success
- [ ] Invalid ZIP structure: 400 with validation details
- [ ] Integration test: round-trip export → modify → import → verify

## Blocked by

- arche-types/export-import-format.md
- arche-types/domain-structs-enums.md
- arche-service/auth-api-key-extractor.md
- arche-service/permission-matrix.md
